use clap::{Arg, App};

pub struct Config {
    pub n_bytes: usize,
    pub n_iterations: usize,
    pub sender_pin: usize,
    pub receiver_pin: usize,
}

/// Extract the configuration from Command line
pub fn parse_config() -> Config {
    let matches = App::new("Config")
        .arg(Arg::with_name("n_bytes")
            .short("b")
            .long("bytes")
            .value_name("n_bytes")
            .help("number of bytes to transfer every round")
            .takes_value(true)
            .default_value("1")
        )
        .arg(Arg::with_name("iterations")
            .short("i")
            .long("iterations")
            .value_name("iterations")
            .help("number of rounds of transfer to perform")
            .takes_value(true)
            .default_value("100000")
        )
        .arg(Arg::with_name("sendpin")
            .short("s")
            .long("sendpin")
            .value_name("sendpin")
            .help("id of process to pin sender thread to, -1 for no pinning")
            .takes_value(true)
            .default_value("0")
        )
        .arg(Arg::with_name("recvpin")
            .short("r")
            .long("recvpin")
            .value_name("recvpin")
            .help("id of process to pin receiver thread to, -1 for no pinning")
            .takes_value(true)
            .default_value("1")
        )
        .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let n_bytes = matches.value_of("n_bytes").unwrap().parse::<usize>().unwrap();
    let n_iterations = matches.value_of("iterations").unwrap().parse::<usize>().unwrap();
    let sender_pin = matches.value_of("sendpin").unwrap().parse::<usize>().unwrap();
    let receiver_pin = matches.value_of("recvpin").unwrap().parse::<usize>().unwrap();

    // Don't kill machines
    if n_bytes > 100_000_000 {
        panic!("More than 100 MB per round is probably too much data you wanna send, \
        you may kill one of the machines. Try with maybe 100MB but more rounds")
    }

    // Very improbable case error handling
    if (n_bytes * 1000000) as u128 * n_iterations as u128 > u64::max_value().into() {
        panic!("There's gonna be too much data. Make sure n_bytes * n_rounds is < u128::MAX")
    }

    Config {
        n_bytes,
        n_iterations,
        sender_pin,
        receiver_pin
    }
}