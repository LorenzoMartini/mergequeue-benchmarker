extern crate npnc;
extern crate timely;
extern crate hdrhist;
extern crate mergequeue_benchmarker;
extern crate amd64_timer;

use npnc::bounded::spsc;
use timely::bytes::arc::Bytes;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Barrier, atomic::AtomicBool, atomic::Ordering};
use mergequeue_benchmarker::config;
use amd64_timer::ticks;

fn main() {

    // Args extract
    let args = config::parse_config();
    let n_iterations = args.n_iterations;
    let affinity_send = args.sender_pin;
    let affinity_recv = args.receiver_pin;
    let clock_frequency = args.clock_frequency;

    // Compute how long to stop betweend sends
    let clock_break = clock_frequency / args.frequency;

    // MergeQueues init
    let (queue_send, queue_recv) = spsc::channel(512);

    // Build barrier to sync threads.
    // Sender will start a bit later to guarantee reader is already waiting
    let barrier = Arc::new(Barrier::new(2));
    let barrier_send = barrier.clone();
    let barrier_recv = barrier.clone();

    // Bool to determine when the receiver is done.
    let receiver_active = Arc::new(AtomicBool::new(true));
    let receiver_active_send = receiver_active.clone();

    // Collect the times when receiving
    let times_recv = thread::spawn(move || {

        mergequeue_benchmarker::utils::set_affinity(affinity_recv);

        // We will read messages into bytes_recv and clear it after every read.
        // We will store in times_recv the timestamp at recv time and the number of messages
        // received (we could read multiple messages from MergeQueue all together)
        let mut times_recv = Vec::with_capacity(n_iterations);
        let mut tot_n_messages = 0;

        barrier_recv.wait();

        while tot_n_messages < n_iterations {
            while match queue_recv.consume() {
                Ok(_item) => {false},
                Err(::npnc::ConsumeError::Disconnected) => { panic!("Disconnected"); },
                Err(::npnc::ConsumeError::Empty) => { true },
            } {}
            let t1 = ticks();

            // We may read more than one message from the queue => need to store how many messages
            // read for the same timestamp
            times_recv.push(t1);
            tot_n_messages += 1;
        }
        println!("Recv done");
        receiver_active.store(false, Ordering::Relaxed);
        times_recv
    });

    // Collect the times when sending
    let times_send = thread::spawn(move || {

        let mut buffer = Bytes::from(vec![0u8; n_iterations * 2]);

        mergequeue_benchmarker::utils::set_affinity(affinity_send);
        let mut times_send = Vec::with_capacity(n_iterations);
        barrier_send.wait();
        let mut tot_n_sent = 0;
        // Start later than recv to guarante recv is already polling
        thread::sleep(Duration::from_millis(1000));

        let mut prev_time = ticks();
        while tot_n_sent < n_iterations {
            let t0 = ticks();
            if t0 - prev_time >= clock_break {
                let to_send = Some(buffer.extract_to(1));
                match queue_send.produce(to_send) {
                    Ok(_item) => {
                        // Collect only n_measures measures
                        times_send.push(t0);
                        tot_n_sent += 1;
                        prev_time = prev_time + clock_break;
                    },
                    Err(::npnc::ProduceError::Full(_item)) => {},
                    Err(_err) => panic!("Can't push for some reason")
                }
            }
        }
        println!("Sender done");
        while receiver_active_send.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(1));
        }
        times_send
    });

    // Wait for completion and extract results
    let sends = times_send.join().unwrap_or_default();
    let recvs = times_recv.join().unwrap_or_default();

    // Collect and print measures using HDRHist
    println!("Collecting to HDRHist");
    let mut hist = hdrhist::HDRHist::new();
    for i in 0..n_iterations {
        hist.add_value(recvs[i] - sends[i]);
    }

    mergequeue_benchmarker::utils::print_hist_summary(hist);
}
