use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use crate::{parser::parse_resp, Command, DataType, Request};
static WRONG_TYPE_ERROR_RESPONSE: &str =
    "-WRONGTYPE Operation against a key holding the wrong kind of value\r\n";

pub fn process_request(
    mut stream: TcpStream,
    cache: Arc<RwLock<HashMap<String, DataType>>>,
    bus: Arc<RwLock<HashMap<String, Vec<TcpStream>>>>,
    timeout: Option<Duration>,
) {
    loop {
        let cache = cache.clone();
        let bus = bus.clone();
        if let Some(timeout) = timeout {
            stream.set_read_timeout(Some(timeout)).unwrap();
        }
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
                let response = get_response(cache, bus, &req, &mut stream);
                stream.write_all(response.as_bytes()).unwrap();
            }
        }
    }
}

pub fn get_response(
    cache: Arc<RwLock<HashMap<String, DataType>>>,
    bus: Arc<RwLock<HashMap<String, Vec<TcpStream>>>>,
    req: &Request,
    stream: &mut TcpStream,
) -> String {
    match req.command {
        Command::SET => handle_set(req, cache),
        Command::GET => handle_get(req, cache),
        Command::RPUSH => handle_rpush(req, cache),
        Command::LRANGE => handle_lrange(req, cache),
        Command::DEL => handle_del(req, cache),
        Command::INCR => handle_incr(req, cache),
        Command::INCRBY => handle_incrby(req, cache),
        Command::DECR => handle_decr(req, cache),
        Command::DECRBY => handle_decrby(req, cache),
        Command::EXPIRE => handle_expire(req, cache),
        Command::CONFIG => handle_config(),
        Command::COMMAND => handle_command(),
        // This Pub/Sub implementation doesn't remove subscribers from the vector after
        // they disconnect, so technically this is a memory leak.
        // However, as a minor fix I remove disconnected streams from the vector on a new PUBLISH.
        // I could fix this once and for all if I changed the return type of get_response
        // to know if the response is for a SUBSCRIBE and remove it when it closes in process_request.
        Command::PUBLISH => handle_publish(req, bus),
        Command::SUBSCRIBE => handle_subscribe(req, bus, stream.try_clone().unwrap()),
    }
}

pub fn handle_set(req: &Request, cache: Arc<RwLock<HashMap<String, DataType>>>) -> String {
    let mut cache = cache.write().unwrap();
    let value = req.value[0].clone();
    cache.insert(req.key.clone(), DataType::String(value));
    let response = "+OK\r\n".to_string();
    response
}

pub fn handle_get(req: &Request, cache: Arc<RwLock<HashMap<String, DataType>>>) -> String {
    let cache = cache.read().unwrap();
    let request_value = cache.get(&req.key);
    match request_value {
        Some(value) => match value {
            DataType::String(value) => {
                let response = format!("${}\r\n{}\r\n", value.len(), value);
                return response;
            }

            _ => {
                let response = WRONG_TYPE_ERROR_RESPONSE.clone().to_string();
                return response;
            }
        },
        None => {
            let response = "$-1\r\n".to_string();
            return response;
        }
    }
}

pub fn handle_rpush(req: &Request, cache: Arc<RwLock<HashMap<String, DataType>>>) -> String {
    let mut cache = cache.write().unwrap();
    let value = cache.get_mut(&req.key);
    match value {
        Some(existing) => match existing {
            DataType::List(v) => {
                for value in &req.value {
                    v.push(value.to_string());
                }
                let response = format!(":{}\r\n", v.len());
                response
            }
            _ => {
                let response = WRONG_TYPE_ERROR_RESPONSE.to_string();
                response
            }
        },
        None => {
            let size = req.value.len();
            cache.insert(req.key.clone(), DataType::List(req.value.clone()));
            let response = format!(":{}\r\n", size);
            response
        }
    }
}

pub fn handle_lrange(req: &Request, cache: Arc<RwLock<HashMap<String, DataType>>>) -> String {
    let cache = cache.read().unwrap();
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
                let response = WRONG_TYPE_ERROR_RESPONSE.clone().to_string();
                return response;
            }
        },

        None => {
            let response = "*0\r\n".to_string();
            return response;
        }
    }
}

pub fn handle_del(req: &Request, cache: Arc<RwLock<HashMap<String, DataType>>>) -> String {
    let mut cache = cache.write().unwrap();
    let key_to_delete = cache.get(&req.key);
    match key_to_delete {
        Some(_) => {
            cache.remove(&req.key);
            let response = ":1\r\n".to_string();
            response
        }
        None => {
            let response = ":0\r\n".to_string();
            response
        }
    }
}

pub fn handle_incr(req: &Request, cache: Arc<RwLock<HashMap<String, DataType>>>) -> String {
    let mut cache = cache.write().unwrap();
    let value = cache.get(&req.key);
    match value {
        Some(v) => match v {
            DataType::String(v) => {
                let v = v.parse::<i64>();
                match v {
                    Ok(i) => {
                        let stringified_value = (i + 1).to_string();
                        cache.insert(req.key.clone(), DataType::String(stringified_value.clone()));
                        let response = format!(":{}\r\n", stringified_value);
                        response
                    }
                    Err(_) => {
                        let response = WRONG_TYPE_ERROR_RESPONSE.clone().to_string();
                        response
                    }
                }
            }
            _ => {
                let response = WRONG_TYPE_ERROR_RESPONSE.clone().to_string();
                response
            }
        },
        None => {
            cache.insert(req.key.clone(), DataType::String(String::from("1")));
            let response = ":1\r\n".to_string();
            response
        }
    }
}

pub fn handle_incrby(req: &Request, cache: Arc<RwLock<HashMap<String, DataType>>>) -> String {
    let mut cache = cache.write().unwrap();
    let value = cache.get(&req.key);
    let amount = req.value[0].parse::<i64>().unwrap();
    match value {
        Some(v) => match v {
            DataType::String(v) => {
                let v = v.parse::<i64>();
                match v {
                    Ok(i) => {
                        let stringified_value = (i + amount).to_string();
                        cache.insert(req.key.clone(), DataType::String(stringified_value.clone()));
                        let response = format!(":{}\r\n", stringified_value);
                        response
                    }
                    Err(_) => {
                        let response = WRONG_TYPE_ERROR_RESPONSE.clone().to_string();
                        response
                    }
                }
            }
            _ => {
                let response = WRONG_TYPE_ERROR_RESPONSE.clone().to_string();
                response
            }
        },
        None => {
            cache.insert(req.key.clone(), DataType::String(amount.to_string()));
            let response = format!(":{amount}\r\n");
            response
        }
    }
}

pub fn handle_decr(req: &Request, cache: Arc<RwLock<HashMap<String, DataType>>>) -> String {
    let mut cache = cache.write().unwrap();
    let value = cache.get(&req.key);
    match value {
        Some(v) => match v {
            DataType::String(v) => {
                let v = v.parse::<i64>();
                match v {
                    Ok(i) => {
                        let stringified_value = (i - 1).to_string();
                        cache.insert(req.key.clone(), DataType::String(stringified_value.clone()));
                        let response = format!(":{}\r\n", stringified_value);
                        response
                    }
                    Err(_) => {
                        let response = WRONG_TYPE_ERROR_RESPONSE.clone().to_string();
                        response
                    }
                }
            }
            _ => {
                let response = WRONG_TYPE_ERROR_RESPONSE.clone().to_string();
                response
            }
        },
        None => {
            cache.insert(req.key.clone(), DataType::String(String::from("-1")));
            let response = ":-1\r\n".to_string();
            response
        }
    }
}

pub fn handle_decrby(req: &Request, cache: Arc<RwLock<HashMap<String, DataType>>>) -> String {
    let mut cache = cache.write().unwrap();
    let value = cache.get(&req.key);
    let amount = req.value[0].parse::<i64>().unwrap();
    match value {
        Some(v) => match v {
            DataType::String(v) => {
                let v = v.parse::<i64>();
                match v {
                    Ok(i) => {
                        let stringified_value = (i - amount).to_string();
                        cache.insert(req.key.clone(), DataType::String(stringified_value.clone()));
                        let response = format!(":{}\r\n", stringified_value);
                        response
                    }
                    Err(_) => {
                        let response = WRONG_TYPE_ERROR_RESPONSE.clone().to_string();
                        response
                    }
                }
            }
            _ => {
                let response = WRONG_TYPE_ERROR_RESPONSE.clone().to_string();
                response
            }
        },
        None => {
            cache.insert(req.key.clone(), DataType::String((-amount).to_string()));
            let response = format!(":{}\r\n", -amount);
            response
        }
    }
}

pub fn handle_config() -> String {
    let response = ":0\r\n".to_string();
    response
}

pub fn handle_command() -> String {
    let response = ":0\r\n".to_string();
    response
}

pub fn handle_expire(req: &Request, cache: Arc<RwLock<HashMap<String, DataType>>>) -> String {
    let key = req.key.clone();
    let seconds = req.value[0].parse::<u64>().unwrap();
    let unlocked_cache = cache.read().unwrap();
    let value = unlocked_cache.get(&req.key).clone();
    let cloned_cache = Arc::clone(&cache);
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(seconds));
        let mut cache = cloned_cache.write().unwrap();
        let value = cache.get(&key).clone();
        match value {
            Some(_) => {
                cache.remove(&key);
            }
            None => {}
        }
    });

    match value {
        Some(_) => ":1\r\n".to_string(),
        None => ":0\r\n".to_string(),
    }
}

pub fn handle_publish(req: &Request, bus: Arc<RwLock<HashMap<String, Vec<TcpStream>>>>) -> String {
    let mut bus = bus.write().unwrap();
    let streams = bus.get_mut(&req.key);
    match streams {
        Some(streams) => {
            let mut number_of_subs = streams.len();
            for i in 0..number_of_subs {
                let stream = &mut streams[i];
                let topic_name = req.key.clone();
                let topic_name_len = topic_name.len();
                let payload = req.value[0].clone();
                let payload_len = payload.len();
                let message = format!("*3\r\n$7\r\nmessage\r\n${topic_name_len}\r\n{topic_name}\r\n${payload_len}\r\n{payload}\r\n");
                let result = stream.write_all(message.as_bytes());
                match result {
                    Ok(_) => {}
                    Err(_) => {
                        streams.remove(i);
                        number_of_subs -= 1;
                    }
                }
            }
            let response = format!(":{number_of_subs}\r\n");
            response
        }
        None => ":0\r\n".to_string(),
    }
}

pub fn handle_subscribe(
    req: &Request,
    bus: Arc<RwLock<HashMap<String, Vec<TcpStream>>>>,
    stream: TcpStream,
) -> String {
    let mut bus = bus.write().unwrap();
    let value = bus.get_mut(&req.key);
    let topic_name = req.key.clone();
    let topic_name_len = topic_name.len();

    let response = format!("*3\r\n$9\r\nsubscribe\r\n${topic_name_len}\r\n{topic_name}\r\n:1\r\n");
    // let stream: Arc<&mut TcpStream> = Arc::clone(&stream);
    match value {
        Some(streams) => {
            streams.push(stream);
            response
        }
        None => {
            bus.insert(topic_name, vec![stream]);
            response
        }
    }
}
