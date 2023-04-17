use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread::{self},
    time::Duration,
};
mod threadpool;
pub struct Sider {
    listener: TcpListener,
}
impl Sider {
    pub fn new() -> Sider {
        let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
        Sider { listener }
    }

    pub fn start(&self) {
        let cache: HashMap<String, DataType> = HashMap::new();
        let mut cache = Arc::new(Mutex::new(cache));
        for stream in self.listener.incoming() {
            let cache = Arc::clone(&mut cache);
            thread::spawn(move || {
                let stream = stream.unwrap();
                return construct_resp(stream, cache);
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

pub fn get_response(cache: Arc<Mutex<HashMap<String, DataType>>>, req: &Request) -> String {
    let error_response = "-WRONGTYPE Operation against a key holding the wrong kind of value\r\n";
    let mut cache = cache.lock().unwrap();
    match req.command {
        Command::SET => {
            let value = req.value[0].clone();
            cache.insert(req.key.clone(), DataType::String(value));
            let response = "+OK\r\n".to_string();
            response
        }
        Command::GET => {
            let request_value = cache.get(&req.key);
            match request_value {
                Some(value) => match value {
                    DataType::String(value) => {
                        let response = format!("${}\r\n{}\r\n", value.len(), value);
                        return response;
                    }

                    _ => {
                        let response = error_response.clone().to_string();
                        return response;
                    }
                },
                None => {
                    let response = "$-1\r\n".to_string();
                    return response;
                }
            }
        }
        Command::RPUSH => {
            let value = cache.get_mut(&req.key);
            match value {
                Some(existing) => match existing {
                    DataType::List(v) => {
                        for value in &req.value {
                            v.push(value.to_string());
                        }
                        let response = format!(":{}\r\n", v.len());
                        return response;
                    }
                    _ => {
                        let response = error_response.to_string();
                        return response;
                    }
                },
                None => {
                    let size = req.value.len();
                    cache.insert(req.key.clone(), DataType::List(req.value.clone()));
                    let response = format!(":{}\r\n", size);
                    return response;
                }
            }
        }
        Command::LRANGE => {
            let value = cache.get(&req.key);
            match value {
                Some(existing) => match existing {
                    DataType::List(value) => {
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
                        return response;
                    }
                    _ => {
                        let response = error_response.clone().to_string();
                        return response;
                    }
                },

                None => {
                    let response = "*0\r\n".to_string();
                    return response;
                }
            }
        }
        Command::DEL => {
            let key_to_delete = cache.get(&req.key);
            match key_to_delete {
                Some(_) => {
                    cache.remove(&req.key);
                    let response = ":1\r\n".to_string();
                    return response;
                }
                None => {
                    let response = ":0\r\n".to_string();
                    return response;
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
                                    req.key.clone(),
                                    DataType::String(stringified_value.clone()),
                                );
                                let response = format!(":{}\r\n", stringified_value);
                                return response;
                            }
                            Err(_) => {
                                let response = error_response.clone().to_string();
                                return response;
                            }
                        }
                    }
                    _ => {
                        let response = error_response.clone().to_string();
                        return response;
                    }
                },
                None => {
                    cache.insert(req.key.clone(), DataType::String(String::from("1")));
                    let response = ":1\r\n".to_string();
                    return response;
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
                                    req.key.clone(),
                                    DataType::String(stringified_value.clone()),
                                );
                                let response = format!(":{}\r\n", stringified_value);
                                return response;
                            }
                            Err(_) => {
                                let response = error_response.clone().to_string();
                                return response;
                            }
                        }
                    }
                    _ => {
                        let response = error_response.clone().to_string();
                        return response;
                    }
                },
                None => {
                    cache.insert(req.key.clone(), DataType::String(amount.to_string()));
                    let response = format!(":{amount}\r\n");
                    return response;
                }
            }
        }
        Command::DECR => {
            let value = cache.get(&req.key);
            match value {
                Some(v) => match v {
                    DataType::String(v) => {
                        let v = v.parse::<i64>();
                        match v {
                            Ok(i) => {
                                let stringified_value = (i - 1).to_string();
                                cache.insert(
                                    req.key.clone(),
                                    DataType::String(stringified_value.clone()),
                                );
                                let response = format!(":{}\r\n", stringified_value);
                                response
                            }
                            Err(_) => {
                                let response = error_response.clone().to_string();
                                return response;
                            }
                        }
                    }
                    _ => {
                        let response = error_response.clone().to_string();
                        return response;
                    }
                },
                None => {
                    cache.insert(req.key.clone(), DataType::String(String::from("-1")));
                    let response = ":-1\r\n".to_string();
                    response
                }
            }
        }
        Command::DECRBY => {
            let value = cache.get(&req.key);
            let amount = req.value[0].parse::<i64>().unwrap();
            match value {
                Some(v) => match v {
                    DataType::String(v) => {
                        let v = v.parse::<i64>();
                        match v {
                            Ok(i) => {
                                let stringified_value = (i - amount).to_string();
                                cache.insert(
                                    req.key.clone(),
                                    DataType::String(stringified_value.clone()),
                                );
                                let response = format!(":{}\r\n", stringified_value);
                                return response;
                            }
                            Err(_) => {
                                let response = error_response.clone().to_string();
                                return response;
                            }
                        }
                    }
                    _ => {
                        let response = error_response.clone().to_string();
                        return response;
                    }
                },
                None => {
                    cache.insert(req.key.clone(), DataType::String((-amount).to_string()));
                    let response = format!(":{}\r\n", -amount);
                    return response;
                }
            }
        }
        Command::CONFIG => {
            let response = "*2\r\n$4\r\nSAVE\r\n$17\r\n3600 1 300 100 60 10000\r\n".to_string();
            return response;
        }
        Command::COMMAND => {
            let response = ":0\r\n".to_string();
            return response;
        }
    }
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
    let mut splitted: Vec<&str> = s.split("\r\n").collect();
    splitted.remove(splitted.len() - 1);
    let size = splitted[0][1..].parse::<usize>().unwrap();
    let mut command: Option<Command> = None;
    let mut key = String::new();
    let mut value: Vec<String> = vec![];
    for i in 1..=size * 2 {
        if i % 2 == 1 {
            continue;
        }
        if i == 2 {
            match splitted[i].to_uppercase().as_str() {
                "SET" => command = Some(Command::SET),
                "GET" => command = Some(Command::GET),
                "DEL" => command = Some(Command::DEL),
                "RPUSH" => command = Some(Command::RPUSH),
                "LRANGE" => command = Some(Command::LRANGE),
                "INCR" => command = Some(Command::INCR),
                "INCRBY" => command = Some(Command::INCRBY),
                "DECR" => command = Some(Command::DECR),
                "DECRBY" => command = Some(Command::DECRBY),
                "CONFIG" => command = Some(Command::CONFIG),
                "COMMAND" => command = Some(Command::COMMAND),
                other => {
                    panic!("{other} command not implemented!");
                }
            }
        } else if i == 4 {
            key = splitted[i].into();
        } else {
            value.push(splitted[i].into())
        }
    }
    // println!("{key}, {:?}", value);
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

pub fn construct_resp(mut stream: TcpStream, cache: Arc<Mutex<HashMap<String, DataType>>>) {
    loop {
        let cache = cache.clone();
        stream
            .set_read_timeout(Some(Duration::from_millis(5000)))
            .unwrap();
        let mut buf_reader = BufReader::new(&mut stream);
        let mut first_line = String::new();
        let res = buf_reader.read_line(&mut first_line);
        match res {
            Err(_) => {
                stream.flush().unwrap();
                stream.shutdown(std::net::Shutdown::Read).unwrap();
                return;
            }
            Ok(s) => {
                if s == 0 {
                    stream.flush().unwrap();
                    stream.shutdown(std::net::Shutdown::Both).unwrap();
                    return;
                }
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
                let req = parse_resp(&resp);
                let cache = cache;
                let response = get_response(cache, &req);
                stream.write_all(response.as_bytes()).unwrap();
            }
        }
    }
}
