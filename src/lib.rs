use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    time::Duration,
};

pub struct Sider {
    listener: TcpListener,
}

impl Sider {
    pub fn new() -> Sider {
        Sider {
            listener: TcpListener::bind("127.0.0.1:6969").unwrap(),
        }
    }

    pub fn start(&self) {
        let mut cache: HashMap<String, String> = HashMap::new();
        for stream in self.listener.incoming() {
            let mut stream = stream.unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            stream.flush().unwrap();
            let mut buf_reader = BufReader::new(&mut stream);
            let mut first_line = String::new();
            buf_reader.read_line(&mut first_line).unwrap();
            let size = first_line[1..2].parse::<usize>().unwrap();
            let mut resp = vec![first_line];
            for _ in 1..=size * 2 {
                let mut line = String::new();
                buf_reader.read_line(&mut line).unwrap();
                resp.push(line);
            }
            let resp = resp.join("");
            let req = parse_resp(&resp);
            if let Command::SET = req.command {
                cache.insert(req.key.clone(), req.value[0].clone());
                stream.write_all(b"+OK\r\n").unwrap();
                stream.flush().unwrap();
            }
            if let Command::GET = req.command {
                let default = String::new();
                let request_value = cache.get(&req.key).unwrap_or_else(|| &default);
                let response = format!("${}\r\n{}\r\n", request_value.len(), request_value);
                stream.write_all(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
        }
    }
}
#[derive(Debug)]
pub enum Command {
    SET,
    GET,
    LLEN,
    SADD,
    EMPTY,
}
#[derive(Debug)]
pub struct Request {
    command: Command,
    key: String,
    value: Vec<String>,
}

pub fn parse_resp(s: &String) -> Request {
    let splitted: Vec<&str> = s.split("\r\n").collect();
    let size = splitted[0][1..].parse::<usize>().unwrap();
    let mut command: Command = Command::EMPTY;
    let mut key = String::new();
    let mut value: Vec<String> = vec![];
    for i in 1..=size * 2 {
        if i % 2 == 1 {
            continue;
        }
        if i == 2 {
            match splitted[i].into() {
                "SET" => command = Command::SET,
                "GET" => command = Command::GET,
                "LLEN" => command = Command::LLEN,
                "SADD" => command = Command::SADD,
                _ => panic!("Invalid command"),
            }
        } else if i == 4 {
            key = splitted[i].into();
        } else {
            value.push(splitted[i].into())
        }
    }
    Request {
        command,
        key,
        value,
    }
}
