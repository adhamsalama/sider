use sider::Sider;
use std::{env, time::Duration};

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
            port = args[index + 1]
                .clone()
                .parse()
                .expect("Port should be an integer");
            if port < 1 || port > 65535 {
                panic!("Port should be between 1 and 65535");
            }
        }
        if arg == "-t" {
            timeout = Some(Duration::from_millis(
                args[index + 1]
                    .clone()
                    .parse()
                    .expect("Timeout should be an integer"),
            ));
        }
    }
    (port, timeout)
}
