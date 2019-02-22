extern crate timely;
extern crate hdrhist;
extern crate mergequeue_benchmarker;
extern crate amd64_timer;

use timely::communication::allocator::zero_copy::bytes_exchange::{MergeQueue, Signal, BytesPush, BytesPull};
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
    let queue = MergeQueue::new(Signal::new());
    let mut queue_send = queue.clone();
    let mut queue_recv = queue.clone();

    // Build barrier to sync threads.
    // Sender will start a bit later to guarantee reader is already waiting
    let barrier = Arc::new(Barrier::new(2));
    let barrier_send = barrier.clone();
    let barrier_recv = barrier.clone();

    // Bool to determine when the receiver is done.
    let receiver_active = Arc::new(AtomicBool::new(true));
    let receiver_active_send = receiver_active.clone();

    // Collect the times when receiving
    let recv = thread::spawn(move || {

        mergequeue_benchmarker::utils::set_affinity(affinity_recv);
        let mut hist = hdrhist::HDRHist::new();

        // We will read messages into bytes_recv and clear it after every read.
        // We will store in times_recv the timestamp at recv time and the number of messages
        // received (we could read multiple messages from MergeQueue all together)
        let mut bytes_recv = Vec::with_capacity(n_iterations);
        let mut tot_n_messages = 0;

        barrier_recv.wait();

        while tot_n_messages < n_iterations {
            assert_eq!(bytes_recv.is_empty(), true);
            let mut t0 = ticks();
            while bytes_recv.is_empty() {
                t0 = ticks();
                queue_recv.drain_into(&mut bytes_recv);
            }
            let t1 = ticks();
            let n_messages = bytes_recv.drain(..).map(|x| x.len()).sum();
            tot_n_messages += n_messages;

            // We may read more than one message from the queue => need to store how many messages
            // read for the same timestamp
            for _ in 0..n_messages {
                hist.add_value(t1 - t0);
            }
        }
        println!("Recv done");
        receiver_active.store(false, Ordering::Relaxed);
        hist
    });

    // Collect the times when sending
    let send = thread::spawn(move || {

        let mut buffer = Bytes::from(vec![0u8; n_iterations * 2]);

        mergequeue_benchmarker::utils::set_affinity(affinity_send);
        let mut hist = hdrhist::HDRHist::new();

        barrier_send.wait();
        // Start later than recv to guarante recv is already polling
        thread::sleep(Duration::from_millis(1000));

        let mut prev_time = ticks();
        while receiver_active_send.load(Ordering::Relaxed) {
            let t0 = ticks();
            if t0 - prev_time >= clock_break {
                let to_send = Some(buffer.extract_to(1));
                let send_t0 = ticks();
                queue_send.extend(to_send);
                let t1 = ticks();
                hist.add_value(t1 - send_t0);
                prev_time = prev_time + clock_break;
            }
        }
        println!("Sender done");
        hist
    });

    // Wait for completion and extract results
    let sends = send.join().unwrap();
    let recvs = recv.join().unwrap();

    println!("SEND LOCK");
    mergequeue_benchmarker::utils::print_hist_summary(sends);
    println!("RECV LOCK");
    mergequeue_benchmarker::utils::print_hist_summary(recvs);
}
