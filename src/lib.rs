use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
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
        let mut cache: HashMap<String, DataType> = HashMap::new();
        let error_response =
            "-WRONGTYPE Operation against a key holding the wrong kind of value\r\n";

        for stream in self.listener.incoming() {
            let mut stream = stream.unwrap();
            let resp = construct_resp(&mut stream);
            let req = parse_resp(&resp);
            // println!("{:?}", req);
            if let Command::SET = req.command {
                let value = req.value[0].clone();
                cache.insert(req.key.clone(), DataType::String(value));
                stream.write_all(b"+OK\r\n").unwrap();
                stream.flush().unwrap();
            } else if let Command::GET = req.command {
                let request_value = cache.get(&req.key);
                match request_value {
                    Some(value) => match value {
                        DataType::String(value) => {
                            let response = format!("${}\r\n{}\r\n", value.len(), value);
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }

                        _ => {
                            let response = error_response.clone();
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }
                    },
                    None => {
                        stream.write_all(b"$-1\r\n").unwrap();
                        stream.flush().unwrap();
                    }
                }
            } else if let Command::RPUSH = req.command {
                let value = cache.get_mut(&req.key);
                match value {
                    Some(existing) => match existing {
                        DataType::List(v) => {
                            // println!("{:?}", v);
                            for value in &req.value {
                                v.push(value.to_string());
                            }
                            let response = format!(":{}\r\n", v.len());
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }
                        _ => {
                            let response = error_response.to_string();
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }
                    },
                    None => {
                        let size = req.value.len();
                        cache.insert(req.key.clone(), DataType::List(req.value));
                        // println!("list now = {:?}", cache.get(&req.key).unwrap());
                        let response = format!(":{}\r\n", size);
                        stream.write_all(response.as_bytes()).unwrap();
                        stream.flush().unwrap();
                    }
                }
            } else if let Command::LRANGE = req.command {
                let value = cache.get(&req.key);
                match value {
                    Some(existing) => match existing {
                        DataType::List(value) => {
                            // println!("list value {:?}", value);
                            let start = req.value[0].parse::<usize>().unwrap();
                            let end = req.value[1].parse::<i64>().unwrap();
                            let slice: &[String];
                            if end == -1 {
                                slice = &value[start..];
                            } else {
                                let mut end = end + 1;
                                if end >= value.len().try_into().unwrap() {
                                    end = value.len().try_into().unwrap();
                                }
                                slice = &value[start..end.try_into().unwrap()];
                            }
                            let mut response = format!("*{}\r\n", slice.len());
                            for value in slice {
                                response = format!("{}${}\r\n{}\r\n", response, value.len(), value);
                            }
                            // println!("{response}");
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }
                        _ => {
                            let response = error_response.clone();
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }
                    },

                    None => {
                        let response = "*0\r\n";
                        stream.write_all(response.as_bytes()).unwrap();
                        stream.flush().unwrap();
                    }
                }
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
    RPUSH,
    LRANGE,
    EMPTY,
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
                // "LLEN" => command = Command::LLEN,
                // "SADD" => command = Command::SADD,
                "RPUSH" => command = Command::RPUSH,
                "LRANGE" => command = Command::LRANGE,
                _ => panic!("Not implemented!"),
            }
        } else if i == 4 {
            key = splitted[i].into();
        } else {
            value.push(splitted[i].into())
        }
    }
    let req = Request {
        command,
        key,
        value,
    };
    println!("{:?}", req);
    req
}

pub fn construct_resp(stream: &mut TcpStream) -> String {
    let mut buf_reader = BufReader::new(stream);
    let mut first_line = String::new();
    buf_reader.read_line(&mut first_line).unwrap();
    let size = first_line[1..first_line.len() - 2]
        .parse::<usize>()
        .unwrap();
    let mut resp = vec![first_line];
    for _ in 1..=size * 2 {
        let mut line = String::new();
        buf_reader.read_line(&mut line).unwrap();
        resp.push(line);
    }
    let resp = resp.join("");
    resp
}
