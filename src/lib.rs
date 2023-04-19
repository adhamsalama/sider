use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};
mod handler;
mod parser;
use handler::process_request;

pub struct Sider {
    listener: TcpListener,
    port: u32,
    timeout: Option<Duration>,
}
impl Sider {
    pub fn new(port: u32, timeout: Option<Duration>) -> Sider {
        let address = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(address).unwrap();
        Sider {
            port,
            listener,
            timeout,
        }
    }

    pub fn start(&self) {
        println!("Sider is running on port {}", self.port);
        let mut cache = Arc::new(RwLock::new(HashMap::new()));
        let timeout = self.timeout;
        for stream in self.listener.incoming() {
            let cache = Arc::clone(&mut cache);
            thread::spawn(move || {
                let stream = stream.unwrap();
                return process_request(stream, cache, timeout);
            });
        }
    }
}
#[derive(Debug)]
pub enum Command {
    SET,
    GET,
    DEL,
    RPUSH,
    LRANGE,
    INCR,
    INCRBY,
    DECR,
    DECRBY,
    EXPIRE,
    CONFIG,
    COMMAND,
}
#[derive(Debug)]
pub struct Request {
    command: Command,
    key: String,
    value: Vec<String>,
}

#[derive(Debug)]
pub enum DataType {
    String(String),
    List(Vec<String>),
    // Set,
    // Hash,
    // SortedSet,
}
