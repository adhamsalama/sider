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
            println!("{:?}", req);
            match req.command {
                Command::SET => {
                    let value = req.value[0].clone();
                    cache.insert(req.key.clone(), DataType::String(value));
                    stream.write_all(b"+OK\r\n").unwrap();
                    stream.flush().unwrap();
                }
                Command::GET => {
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
                }
                Command::RPUSH => {
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
                }
                Command::LRANGE => {
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
                                    response =
                                        format!("{}${}\r\n{}\r\n", response, value.len(), value);
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
                Command::DEL => {
                    let key_to_delete = cache.get(&req.key);
                    match key_to_delete {
                        Some(_) => {
                            cache.remove(&req.key);
                            let response = ":1\r\n";
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }
                        None => {
                            let response = ":0\r\n";
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }
                    }
                }
                Command::INCR => {
                    let value = cache.get(&req.key);
                    match value {
                        Some(v) => match v {
                            DataType::String(v) => {
                                let v = v.parse::<i64>();
                                match v {
                                    Ok(i) => {
                                        let stringified_value = (i + 1).to_string();
                                        cache.insert(
                                            req.key,
                                            DataType::String(stringified_value.clone()),
                                        );
                                        let response = format!(":{}\r\n", stringified_value);
                                        stream.write_all(response.as_bytes()).unwrap();
                                        stream.flush().unwrap();
                                    }
                                    Err(_) => {
                                        let response = error_response.clone();
                                        stream.write_all(response.as_bytes()).unwrap();
                                        stream.flush().unwrap();
                                    }
                                }
                            }
                            _ => {
                                let response = error_response.clone();
                                stream.write_all(response.as_bytes()).unwrap();
                                stream.flush().unwrap();
                            }
                        },
                        None => {
                            cache.insert(req.key, DataType::String(String::from("1")));
                            let response = ":1\r\n";
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }
                    }
                }
                Command::INCRBY => {
                    let value = cache.get(&req.key);
                    let amount = req.value[0].parse::<i64>().unwrap();
                    match value {
                        Some(v) => match v {
                            DataType::String(v) => {
                                let v = v.parse::<i64>();
                                match v {
                                    Ok(i) => {
                                        let stringified_value = (i + amount).to_string();
                                        cache.insert(
                                            req.key,
                                            DataType::String(stringified_value.clone()),
                                        );
                                        let response = format!(":{}\r\n", stringified_value);
                                        stream.write_all(response.as_bytes()).unwrap();
                                        stream.flush().unwrap();
                                    }
                                    Err(_) => {
                                        let response = error_response.clone();
                                        stream.write_all(response.as_bytes()).unwrap();
                                        stream.flush().unwrap();
                                    }
                                }
                            }
                            _ => {
                                let response = error_response.clone();
                                stream.write_all(response.as_bytes()).unwrap();
                                stream.flush().unwrap();
                            }
                        },
                        None => {
                            cache.insert(req.key, DataType::String(amount.to_string()));
                            let response = format!(":{amount}\r\n");
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }
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
    DEL,
    // LLEN,
    // SADD,
    RPUSH,
    LRANGE,
    INCR,
    INCRBY,
    // DECR,
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
    let mut command: Option<Command> = None;
    let mut key = String::new();
    let mut value: Vec<String> = vec![];
    for i in 1..=size * 2 {
        if i % 2 == 1 {
            continue;
        }
        if i == 2 {
            match splitted[i].into() {
                "SET" => command = Some(Command::SET),
                "GET" => command = Some(Command::GET),
                "DEL" => command = Some(Command::DEL),
                // "LLEN" => command = Command::LLEN,
                // "SADD" => command = Command::SADD,
                "RPUSH" => command = Some(Command::RPUSH),
                "LRANGE" => command = Some(Command::LRANGE),
                "INCR" => command = Some(Command::INCR),
                "INCRBY" => command = Some(Command::INCRBY),
                other => {
                    panic!("Not {other} implemented!")
                }
            }
        } else if i == 4 {
            key = splitted[i].into();
        } else {
            value.push(splitted[i].into())
        }
    }
    match command {
        Some(c) => {
            let req = Request {
                command: c,
                key,
                value,
            };
            return req;
        }
        None => panic!("Something went wrong with parsing RESP"),
    }
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
