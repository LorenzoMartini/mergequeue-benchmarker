extern crate timely;
extern crate streaming_harness_hdrhist;
extern crate core_affinity;
extern crate mergequeue_benchmarker;

use timely::communication::allocator::zero_copy::bytes_exchange::{MergeQueue, Signal, BytesPush, BytesPull};
use timely::bytes::arc::Bytes;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Barrier};
use mergequeue_benchmarker::config;

fn main() {

    // Args extract
    let args = config::parse_config();
    let n_iterations = args.n_iterations;
    let msg_size = args.n_bytes;
    let affinity_send = args.sender_pin;
    let affinity_recv = args.receiver_pin;

    // MergeQueues init
    let queue = MergeQueue::new(Signal::new());
    let mut queue_send = queue.clone();
    let mut queue_recv = queue.clone();

    // Build barrier to sync threads.
    // Sender will start a bit later to guarantee reader is already waiting
    let barrier = Arc::new(Barrier::new(2));
    let barrier_send = barrier.clone();
    let barrier_recv = barrier.clone();

    let active_threads = Arc::new(0 as u8);
    let recv_active = active_threads.clone();

    // Collect the times when receiving
    let times_recv = thread::spawn(move || {

        set_affinity(affinity_recv);

        let mut bytes_recv = Vec::with_capacity(1);
        let mut times_recv = Vec::with_capacity(n_iterations);

        barrier_recv.wait();

        for i in 0..n_iterations {
            while bytes_recv.is_empty() {
                queue_recv.drain_into(&mut bytes_recv);
            }
            let t1 = Instant::now();

            // Verify everything as expected and print progress
            assert_eq!(bytes_recv.remove(0).len(), msg_size);
            if i * 10 % n_iterations == 0 {
                println!("Received {}%", i * 100 / n_iterations);
            }
            times_recv.push(t1);
        }
        println!("Recv done");
        let _drop_active = Arc::try_unwrap(recv_active);
        times_recv
    });

    // Collect the times when sending
    let times_send = thread::spawn(move || {

        set_affinity(affinity_send);
        let mut times_send = Vec::with_capacity(n_iterations);

        barrier_send.wait();
        thread::sleep(Duration::from_millis(1000));

        while Arc::strong_count(&active_threads) > 1 {
            let to_send = vec![Bytes::from(vec![0; msg_size])].into_iter();
            let t0 = Instant::now();
            queue_send.extend(to_send);
            if times_send.len() < n_iterations {
                times_send.push(t0);
            }
        }
        println!("Sender done");
        times_send
    });

    // Wait for completion and extract results
    let mut sends = times_send.join().unwrap_or_default();
    let mut recvs = times_recv.join().unwrap_or_default();
    assert_eq!(sends.len(), recvs.len());

    // Collect and print measures using HDRHist
    println!("Collecting to HDRHist");
    let mut hist = streaming_harness_hdrhist::HDRHist::new();
    for _ in 0..n_iterations {
        let duration = recvs.remove(0).duration_since(sends.remove(0));
        hist.add_value(duration.as_secs() * 1_000_000_000u64 + duration.subsec_nanos() as u64);
    }
    print_summary(hist);

}

// / Pin thread to physical core using provided id
fn set_affinity(t_id: usize) {
    let core_ids = core_affinity::get_core_ids().unwrap();
    core_affinity::set_for_current(core_ids[t_id % core_ids.len()]);
}

/// Nicely outputs summary of execution with stats and CDF points.
fn print_summary(hist: streaming_harness_hdrhist::HDRHist) {
    println!("Sent/received everything!");
    print_line();
    println!("HDRHIST summary, measure in ns");
    print_line();
    println!("summary:\n{:#?}", hist.summary().collect::<Vec<_>>());
    print_line();
    println!("Summary_string:\n{}", hist.summary_string());
    print_line();
    println!("CDF summary:\n");
    for entry in hist.ccdf() {
        println!("{:?}", entry);
    }
}

/// Prints dashed line
fn print_line() {
    println!("\n-------------------------------------------------------------\n");
}
