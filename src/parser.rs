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
