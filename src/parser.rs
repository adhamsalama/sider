use std::collections::HashSet;

use crate::{Command, Request};

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
                "EXPIRE" => command = Some(Command::EXPIRE),
                "CONFIG" => command = Some(Command::CONFIG),
                "COMMAND" => command = Some(Command::COMMAND),
                "PUBLISH" => command = Some(Command::PUBLISH),
                "SUBSCRIBE" => command = Some(Command::SUBSCRIBE),
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
    match command {
        Some(command) => {
            if let Command::SUBSCRIBE = command {
                value.insert(0, key);
                // Remove duplicates as it's the official Redis behavior
                let set: HashSet<_> = value.drain(..).collect();
                value.extend(set);
                key = String::new();
            }
            let req = Request {
                command,
                key,
                value,
            };
            return req;
        }
        None => panic!("Something went wrong with parsing RESP"),
    }
}
