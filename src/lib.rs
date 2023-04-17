use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};
mod handler;
mod parser;
use handler::process_request;
pub struct Sider {
    listener: TcpListener,
}
impl Sider {
    pub fn new() -> Sider {
        let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
        Sider { listener }
    }

    pub fn start(&self) {
        let mut cache = Arc::new(Mutex::new(HashMap::new()));
        for stream in self.listener.incoming() {
            let cache = Arc::clone(&mut cache);
            thread::spawn(move || {
                let stream = stream.unwrap();
                return process_request(stream, cache);
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
