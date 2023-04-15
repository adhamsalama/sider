use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write, Read},
    net::{TcpListener, TcpStream}, sync::{Arc, Mutex}, time::Duration,
};
mod threadpool;
use threadpool::ThreadPool;
pub struct Sider {
    listener: TcpListener,
}
impl Sider {
    pub fn new() -> Sider {
        let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
        // listener.set_nonblocking(true).expect("set_nonblocking call failed");
        Sider { listener }
    }

    pub fn start(&self) {
        let  cache: HashMap<String, DataType> = HashMap::new();
        let mut cache = Arc::new( Mutex::new(cache));
        let error_response =
            "-WRONGTYPE Operation against a key holding the wrong kind of value\r\n";
        let pool = ThreadPool::new(4);
        let mut i = 0;
        for stream in self.listener.incoming() {
            let mut stream = stream.unwrap();
            let cache = Arc::clone(&mut cache);
            pool.execute(move|| {
                let resp = construct_resp(&mut stream);
                println!("respoooooo {:?}", resp);
                // let req = parse_resp(&resp);
                let mut cache = cache.lock().expect("YOOOOOO");     
                // println!("{:?}", req);
                let reqs: Vec<Request> = resp.iter().map(|i| parse_resp(i)).collect();
                let responses : Vec<String>= reqs.iter().map(|req| 
                    match req.command {
                        Command::SET => {
                            let value = req.value[0].clone();
                            cache.insert(req.key.clone(), DataType::String(value));
                            let response ="+OK\r\n".to_string(); 
                            response
                            // stream.write_all().unwrap();
                            // stream.flush().unwrap();
                        }
                        Command::GET => {
                            let request_value = cache.get(&req.key);
                            match request_value {
                                Some(value) => match value {
                                    DataType::String(value) => {
                                        let response = format!("${}\r\n{}\r\n", value.len(), value);
                                        return response;
                                        // stream.write_all(response.as_bytes()).unwrap();
                                        // stream.flush().unwrap();
                                    }
        
                                    _ => {
                                        let response = error_response.clone().to_string();
                                        return response;
                                        // strkeam.write_all(response.as_bytes()).unwrap();
                                        // stream.flush().unwrap();
                                    }
                                },
                                None => {
                                    let response = "$-1\r\n".to_string();
                                    return response;
                                    // stream.write_all(b"$-1\r\n").unwrap();
                                    // stream.flush().unwrap();
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
                                        return response;
                                        // stream.write_all(response.as_bytes()).unwrap();
                                        // stream.flush().unwrap();
                                    }
                                    _ => {
                                        let response = error_response.to_string();
                                        return response;
                                        // stream.write_all(response.as_bytes()).unwrap();
                                        // stream.flush().unwrap();
                                    }
                                },
                                None => {
                                    let size = req.value.len();
                                    cache.insert(req.key.clone(), DataType::List(req.value.clone()));
                                    // println!("list now = {:?}", cache.get(&req.key).unwrap());
                                    let response = format!(":{}\r\n", size);
                                    return response;
                                    // stream.write_all(response.as_bytes()).unwrap();
                                    // stream.flush().unwrap();
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
                                        return response;
                                        // println!("{response}");
                                        // stream.write_all(response.as_bytes()).unwrap();
                                        // stream.flush().unwrap();
                                    }
                                    _ => {
                                        let response = error_response.clone().to_string();
                                        return response;
                                        // stream.write_all(response.as_bytes()).unwrap();
                                        // stream.flush().unwrap();
                                    }
                                },
        
                                None => {
                                    let response = "*0\r\n".to_string();
                                    return response;
                                    // stream.write_all(response.as_bytes()).unwrap();
                                    // stream.flush().unwrap();
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
                                    // stream.write_all(response.as_bytes()).unwrap();
                                    // stream.flush().unwrap();
                                }
                                None => {
                                    let response = ":0\r\n".to_string();
                                    return response;
                                    // stream.write_all(response.as_bytes()).unwrap();
                                    // stream.flush().unwrap();
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
                                                // stream.write_all(response.as_bytes()).unwrap();
                                                // stream.flush().unwrap();
                                            }
                                            Err(_) => {
                                                let response = error_response.clone().to_string();
                                                return response;
                                                // stream.write_all(response.as_bytes()).unwrap();
                                                // stream.flush().unwrap();
                                            }
                                        }
                                    }
                                    _ => {
                                        let response = error_response.clone().to_string();
                                        return response;
                                        // stream.write_all(response.as_bytes()).unwrap();
                                        // stream.flush().unwrap();
                                    }
                                },
                                None => {
                                    cache.insert(req.key.clone(), DataType::String(String::from("1")));
                                    let response = ":1\r\n".to_string();
                                    return response;
                                    // stream.write_all(response.as_bytes()).unwrap();
                                    // stream.flush().unwrap();
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
                                                // stream.write_all(response.as_bytes()).unwrap();
                                                // stream.flush().unwrap();
                                            }
                                            Err(_) => {
                                                let response = error_response.clone().to_string();
                                                return response;
                                                // stream.write_all(response.as_bytes()).unwrap();
                                                // stream.flush().unwrap();
                                            }
                                        }
                                    }
                                    _ => {
                                        let response = error_response.clone().to_string();
                                        return response;
                                        // stream.write_all(response.as_bytes()).unwrap();
                                        // stream.flush().unwrap();
                                    }
                                },
                                None => {
                                    cache.insert(req.key.clone(), DataType::String(amount.to_string()));
                                    let response = format!(":{amount}\r\n");
                                    return response;
                                    // stream.write_all(response.as_bytes()).unwrap();
                                    // stream.flush().unwrap();
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
                                                // stream.write_all(response.as_bytes()).unwrap();
                                                // stream.flush().unwrap();
                                            }
                                            Err(_) => {
                                                let response = error_response.clone().to_string();
                                                return response;
                                                // stream.write_all(response.as_bytes()).unwrap();
                                                // stream.flush().unwrap();
                                            }
                                        }
                                    }
                                    _ => {
                                        let response = error_response.clone().to_string();
                                        return response;
                                        // stream.write_all(response.as_bytes()).unwrap();
                                        // stream.flush().unwrap();
                                    }
                                },
                                None => {
                                    cache.insert(req.key.clone(), DataType::String(String::from("-1")));
                                    let response = ":-1\r\n".to_string();
                                    response
                                    // stream.write_all(response.as_bytes()).unwrap();
                                    // stream.flush().unwrap();
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
                                                // stream.write_all(response.as_bytes()).unwrap();
                                                // stream.flush().unwrap();
                                            }
                                            Err(_) => {
                                                let response = error_response.clone().to_string();
                                                return response;
                                                // stream.write_all(response.as_bytes()).unwrap();
                                                // stream.flush().unwrap();
                                            }
                                        }
                                    }
                                    _ => {
                                        let response = error_response.clone().to_string();
                                        return response;
                                        // stream.write_all(response.as_bytes()).unwrap();
                                        // stream.flush().unwrap();
                                    }
                                },
                                None => {
                                    cache.insert(req.key.clone(), DataType::String((-amount).to_string()));
                                    let response = format!(":{}\r\n", -amount);
                                    return response;
                                    // stream.write_all(response.as_bytes()).unwrap();
                                    // stream.flush().unwrap();
                                }
                            }
                        }
                        Command::CONFIG => {
                            let response = "*2\r\n$4\r\nSAVE\r\n$17\r\n3600 1 300 100 60 10000\r\n".to_string();
                            return response;
                            // println!("aaa");
                            // stream.write_all(response.as_bytes()).unwrap();
                            // stream.flush().unwrap();
                        }
                        Command::COMMAND => {
                            let response = "*9\r\n$3\r\nSET\r\n$3\r\nGET\r\n$3\r\nDEL\r\n$5\r\nRPUSH\r\n$6\r\nLRANGE\r\n$4\r\nINCR\r\n$6\r\nINCRBY\r\n$4\r\nDECR\r\n$6\r\nDECRBY\r\n".to_string();
                            return response;
                            // stream.write_all(response.as_byteks()).unwrap();
                            // stream.write_all(response.as_byteks()).unwrap();
                            // stream.flush().unwrap();
                        }
                    
                }).collect();
                stream.write_all(responses.join("").as_bytes()).unwrap();
            
            });
            i += 1;
            println!("i {i}");
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
pub fn parse_resp(s: &String) -> Request {
    let splitted: Vec<&str> = s.split("\r\n").collect();
    println!("s, {}, splitted {:?}", s, splitted);
    let size = splitted[0][1..].parse::<usize>().unwrap();
    let mut command: Option<Command> = None;
    let mut key = String::new();
    let mut value: Vec<String> = vec![];
    for i in 1..=size * 2 {
        if i % 2 == 1 {
            continue;
        }
        if i == 2 {
            match splitted[i-1].into() {
                "SET" => command = Some(Command::SET),
                "GET" => command = Some(Command::GET),
                "DEL" => command = Some(Command::DEL),
                // "LLEN" => command = Command::LLEN,
                // "SADD" => command = Command::SADD,
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

pub fn construct_resp(stream: &mut TcpStream) -> Vec<String> {
    let mut buf_reader = BufReader::new(stream);
    // let mut read_size = 1;
    // println!("new line {first_line}");
    let mut resps: Vec<String> = vec![];
    let mut q = buf_reader.take(512);
    let mut buf: [u8; 512] = [0; 512];
    q.read(&mut buf);
    println!("qqqqqq {:?}", buf);
    // loop  {
    //     let mut first_line = String::new();
    //     println!("first_line {first_line}");
    //     let read_size = buf_reader.read_line(&mut first_line).unwrap();
    //     println!("size: {read_size}");
    //     if read_size == 0 {
    //         break
    //     }
    //     let size = first_line.clone()[1..first_line.clone().len() - 2]
    //     .parse::<usize>()
    //     .unwrap();
    //     let mut resp = vec![first_line];
    //     for _ in 1..=size * 2 {
    //         let mut line = String::new();
    //         buf_reader.read_line(&mut line).unwrap();
    //         resp.push(line);
    //     }
    //     let resp = resp.join("");
    //     println!("resp {resp}");
    //     resps.push(resp);

    // }
    // // println!("{:?}", resps);
    let mut strings = Vec::new();
    let mut last_index = 0;

    for (index, &byte) in buf.iter().enumerate() {
        if byte == b'*' {
            continue;
        }
        if byte == 0 {
            break
        }
        if byte == b'\r' {
            let slice = &buf[last_index..index];
            if let Ok(s) = std::str::from_utf8(slice) {
                strings.push(s.to_string());
            }
            last_index = index +2;
        }
    }
    println!("strings {:?}", strings);
    strings
    
    // println!("before");
    // let request: Vec<String> = buf_reader
    //     .lines().map(|line| line.unwrap())
    //     .collect();
    // println!("req {:?}", request);
    // String::from("wewe")

}
