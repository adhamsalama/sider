use sider::Sider;
use std::{env, process::exit, time::Duration};

fn main() {
    let args: Vec<String> = env::args().collect();
    let (port, timeout) = parse_args(args);
    let sider = Sider::new(port, timeout);
    sider.start();
}

fn parse_args(args: Vec<String>) -> (u32, Option<Duration>) {
    let mut port: u32 = 6379;
    let mut timeout: Option<Duration> = None;
    for (index, arg) in args.iter().enumerate() {
        if arg == "-p" {
            let parsed_port = args[index + 1].parse();
            match parsed_port {
                Ok(p) => port = p,
                Err(_) => {
                    println!("Error: Port should be an integer");
                    exit(1);
                }
            }
            if port < 1 || port > 65535 {
                println!("Error: Port should be between 1 and 65535");
                exit(1);
            }
        }
        if arg == "-t" {
            let parsed_timeout = args[index + 1].parse();
            match parsed_timeout {
                Ok(t) => timeout = Some(Duration::from_millis(t)),
                Err(_) => {
                    println!("Error: Timeout should be an integer");
                    exit(1);
                }
            }
        }
    }
    (port, timeout)
}
