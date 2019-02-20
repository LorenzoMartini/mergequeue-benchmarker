extern crate timely;
extern crate hdrhist;
extern crate mergequeue_benchmarker;
extern crate amd64_timer;

use timely::communication::allocator::zero_copy::bytes_exchange::{MergeQueue, Signal, BytesPush};
use timely::bytes::arc::Bytes;
use mergequeue_benchmarker::config;
use amd64_timer::ticks;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

fn main() {

    // Args extract
    let args = config::parse_config();
    let n_iterations = args.n_iterations;
    let affinity_send = args.sender_pin;

    // MergeQueues init
    let queue = MergeQueue::new(Signal::new());
    let mut queue_send = queue.clone();
    let mut _queue_recv = queue.clone();

    let send = thread::spawn(move || {
        let mut hist = hdrhist::HDRHist::new();
        let mut buffer = Bytes::from(vec![0u8; n_iterations * 2]);
        mergequeue_benchmarker::utils::set_affinity(affinity_send);
        for _ in 0..n_iterations {
            let to_send = Some(buffer.extract_to(1));
            let t0 = ticks();
            queue_send.extend(to_send);
            let t1 = ticks();
            hist.add_value(t1 - t0);
        }
        // Collect and print measures using HDRHist
        println!("Clock cycles for sender without receiver summary");
        mergequeue_benchmarker::utils::print_hist_summary(hist);
    });
    thread::spawn(move || {
        sleep(Duration::from_millis(1));
        _queue_recv.is_complete();
    });
    send.join();
}
