extern crate hdrhist;
extern crate mergequeue_benchmarker;
extern crate amd64_timer;

use amd64_timer::ticks;

fn main() {

    let mut hist = hdrhist::HDRHist::new();
    mergequeue_benchmarker::utils::set_affinity(0);
    for _ in 0..1000000 {
        let t0 = ticks();
        let t1 = ticks();
        hist.add_value(t1 - t0);
    }
    println!("SUmmary of cost of tacking cycle count");
    mergequeue_benchmarker::utils::print_hist_summary(hist);
}
