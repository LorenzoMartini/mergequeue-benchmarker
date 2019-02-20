extern crate timely;
extern crate streaming_harness_hdrhist;
extern crate core_affinity;
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
    let times_recv = thread::spawn(move || {

        mergequeue_benchmarker::utils::set_affinity(affinity_recv);

        // We will read messages into bytes_recv and clear it after every read.
        // We will store in times_recv the timestamp at recv time and the number of messages
        // received (we could read multiple messages from MergeQueue all together)
        let mut bytes_recv = Vec::with_capacity(n_iterations);
        let mut times_recv = Vec::with_capacity(n_iterations);
        let mut tot_n_messages = 0;

        barrier_recv.wait();

        while tot_n_messages < n_iterations {
            assert_eq!(bytes_recv.is_empty(), true);
            while bytes_recv.is_empty() {
                queue_recv.drain_into(&mut bytes_recv);
            }
            let t1 = ticks();
            let n_messages = bytes_recv.drain(..).map(|x| x.len()).sum();
            tot_n_messages += n_messages;

            // We may read more than one message from the queue => need to store how many messages
            // read for the same timestamp
            times_recv.push((t1, n_messages));
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
        // Start later than recv to guarante recv is already polling
        thread::sleep(Duration::from_millis(1000));

        let mut prev_time = ticks();
        while receiver_active_send.load(Ordering::Relaxed) {
            let t0 = ticks();
            if t0 - prev_time >= clock_break {
                let to_send = Some(buffer.extract_to(1));
                queue_send.extend(to_send);

                // Collect only n_measures measures
                if times_send.len() < n_iterations {
                    times_send.push(t0);
                }
                prev_time = prev_time + clock_break;
            }
        }
        println!("Sender done");
        times_send
    });

    // Wait for completion and extract results
    let sends = times_send.join().unwrap_or_default();
    let recvs = times_recv.join().unwrap_or_default();

    // Collect and print measures using HDRHist
    println!("Collecting to HDRHist");
    let mut hist = streaming_harness_hdrhist::HDRHist::new();
    let mut n_messages_hist = streaming_harness_hdrhist::HDRHist::new();
    let mut i = 0;
    'outer: for time_quantity_pair in recvs {
        'inner: for _ in 0..time_quantity_pair.1 {
            // Save only first n_iterations measures
            if i >= n_iterations {
                break 'outer;
            }
            let duration = time_quantity_pair.0 - sends[i];
            hist.add_value(duration);
            n_messages_hist.add_value(time_quantity_pair.1 as u64);
            i += 1;
        }
    }

    mergequeue_benchmarker::utils::print_summary(hist, n_messages_hist);
}
