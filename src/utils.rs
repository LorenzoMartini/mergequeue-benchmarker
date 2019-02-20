// Util functions for benchmarkers

// / Pin thread to physical core using provided id
pub fn set_affinity(t_id: usize) {
    let core_ids = core_affinity::get_core_ids().unwrap();
    core_affinity::set_for_current(core_ids[t_id % core_ids.len()]);
}

pub fn print_summary(hist: hdrhist::HDRHist, msg_hist: hdrhist::HDRHist) {
    print_line();
    println!("VALUES HIST");
    print_hist_summary(hist);
    println!("MESSAGES QUANTITY HIST");
    print_hist_summary(msg_hist);
}
/// Nicely outputs summary of execution with stats and CDF points.
pub fn print_hist_summary(hist: hdrhist::HDRHist) {
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
