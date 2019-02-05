use clap::{Arg, App};

pub struct Config {
    pub n_iterations: usize,
    pub frequency: u64,
    pub sender_pin: usize,
    pub receiver_pin: usize,
    pub clock_frequency: u64,
}

/// Extract the configuration from Command line
pub fn parse_config() -> Config {
    let matches = App::new("Config")
        .arg(Arg::with_name("iterations")
            .short("i")
            .long("iterations")
            .value_name("iterations")
            .help("number of rounds of transfer to perform")
            .takes_value(true)
            .default_value("100000")
        )
        .arg(Arg::with_name("frequency")
            .short("f")
            .long("frequency")
            .value_name("frequency")
            .help("frequency at which sending messages")
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
        .arg(Arg::with_name("clock")
            .short("c")
            .long("clock")
            .value_name("clock")
            .help("frequency of system clock")
            .takes_value(true)
            .default_value("2400000000")
        )
        .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let n_iterations = matches.value_of("iterations").unwrap().parse::<usize>().unwrap();
    let frequency = matches.value_of("frequency").unwrap().parse::<u64>().unwrap();
    let sender_pin = matches.value_of("sendpin").unwrap().parse::<usize>().unwrap();
    let receiver_pin = matches.value_of("recvpin").unwrap().parse::<usize>().unwrap();
    let clock_frequency = matches.value_of("clock").unwrap().parse::<u64>().unwrap();

    Config {
        n_iterations,
        frequency,
        sender_pin,
        receiver_pin,
        clock_frequency
    }
}
