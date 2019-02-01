extern crate timely;
extern crate streaming_harness_hdrhist;
extern crate core_affinity;
extern crate mergequeue_benchmarker;

use timely::communication::allocator::zero_copy::bytes_exchange::{MergeQueue, Signal, BytesPush, BytesPull};
use timely::bytes::arc::Bytes;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Barrier, atomic::AtomicBool, atomic::Ordering};
use std::ops::Add;
use mergequeue_benchmarker::config;

fn main() {

    // Args extract
    let args = config::parse_config();
    let n_iterations = args.n_iterations;
    let msg_size = args.n_bytes;
    let affinity_send = args.sender_pin;
    let affinity_recv = args.receiver_pin;
    let ns_break = Duration::from_nanos(1_000_000_000 / args.frequency);

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

        set_affinity(affinity_recv);

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
            let t1 = Instant::now();
            let n_messages = bytes_recv.len();
            tot_n_messages += n_messages;

            // We may read more than one message from the queue => need to store how many messages
            // read for the same timestamp
            times_recv.push((t1, n_messages));
            bytes_recv.clear();

            // Print progress
            if tot_n_messages * 100 % n_iterations == 0 {
                println!("Received {}%", tot_n_messages * 100 / n_iterations);
            }
        }
        println!("Recv done");
        receiver_active.store(false, Ordering::Relaxed);
        times_recv
    });

    // Collect the times when sending
    let times_send = thread::spawn(move || {

        set_affinity(affinity_send);
        let mut times_send = Vec::with_capacity(n_iterations);
        barrier_send.wait();
        thread::sleep(Duration::from_millis(1000));

        let mut prev_time = Instant::now();
        while receiver_active_send.load(Ordering::Relaxed) {
            let t0 = Instant::now();
            if t0.duration_since(prev_time).ge(&ns_break) {
                let to_send = vec![Bytes::from(vec![0; msg_size])].into_iter();
                queue_send.extend(to_send);

                // Collect only n_measures measures
                if times_send.len() < n_iterations {
                    times_send.push(t0);
                }
                prev_time = prev_time.add(ns_break);
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
            let duration = time_quantity_pair.0.duration_since(sends[i]);
            hist.add_value(duration.as_secs() * 1_000_000_000u64 + duration.subsec_nanos() as u64);
            n_messages_hist.add_value(time_quantity_pair.1 as u64);
            i += 1;
        }
    }

    print_summary(hist, n_messages_hist);
}

// / Pin thread to physical core using provided id
fn set_affinity(t_id: usize) {
    let core_ids = core_affinity::get_core_ids().unwrap();
    core_affinity::set_for_current(core_ids[t_id % core_ids.len()]);
}

fn print_summary(hist: streaming_harness_hdrhist::HDRHist, msg_hist: streaming_harness_hdrhist::HDRHist) {
    print_line();
    println!("VALUES HIST");
    print_hist_summary(hist);
    println!("MESSAGES QUANTITY HIST");
    print_hist_summary(msg_hist);
}
/// Nicely outputs summary of execution with stats and CDF points.
fn print_hist_summary(hist: streaming_harness_hdrhist::HDRHist) {
    print_line();
    println!("summary:\n{:#?}", hist.summary().collect::<Vec<_>>());
    print_line();
    println!("Summary_string:\n{}", hist.summary_string());
    print_line();
    println!("CDF summary:\n");
    for entry in hist.ccdf() {
        println!("{:?}", entry);
    }
    print_line();
}

/// Prints dashed line
fn print_line() {
    println!("\n-------------------------------------------------------------\n");
}
