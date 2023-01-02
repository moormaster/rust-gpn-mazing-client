use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{Shutdown, TcpStream};

use gpn_mazing_client::{FieldEnvironment, Game, Position};

struct Config {
    ip: String,
    port: String,
    username: String,
    password: String,
    flag_continuous: bool
}

fn parse_arguments() -> Result<Config, String> {
    let mut result = Config {
        ip: "".to_string(),
        port: "".to_string(),
        username: "".to_string(),
        password: "".to_string(),
        flag_continuous: false
    };

    let mut positional_arguments: Vec<String> = vec![];
    for arg in env::args() {
        if arg.starts_with("-") {
            match arg.as_str() {
                "--continuous" | "-c" => { result.flag_continuous = true; },
                _ => {}
            }
        } else {
            positional_arguments.push(arg);
        }
    }

    if positional_arguments.len() < 5 {
        return Err(format!("Usage: {} [-c|--continuous] <ip> <port> <user> <password>", positional_arguments[0]));
    }

    result.ip = positional_arguments[1].clone();
    result.port = positional_arguments[2].clone();
    result.username = positional_arguments[3].clone();
    result.password = positional_arguments[4].clone();

    Ok(result)
}

fn main() {
    let config = {
        match parse_arguments() {
            Ok(config) => { config },
            Err(message) => {
                println!("{}", message);
                return;
            }
        }
    };

    println!("Connecting to {}:{}...", config.ip, config.port);

    let mut stream = TcpStream::connect(format!("{}:{}", config.ip, config.port)).expect("Couldn't connect to server");
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    let mut game: Option<Game> = None;
    let mut current_field_environment;

    println!("[II] Logging in as {}", config.username);
    stream.write_all( format!("join|{}|{}\n", config.username, config.password).as_bytes() ).expect("failed to transmit login info");

    let mut done = false;
    while !done {
        let mut buf = String::from("");
        reader.read_line(&mut buf).expect("Error while receiving");
        print!("[II] Received: {}", buf);

        if buf.starts_with("motd") {
            let parts: Vec<&str> = buf.split("|").collect();
            println!("[II] MOTD:\n{}", parts[1]);
        }

        if buf.starts_with("error") {
            let parts: Vec<&str> = buf.split("|").collect();
            println!("[EE] {}", parts[1]);
            done = true;
        }

        if buf.starts_with("game") {
            let parts: Vec<&str> = buf.split("|").collect();

            let width: usize = parts[1].parse().expect("Failed to parse width value");
            let height: usize = parts[2].parse().expect("Failed to parse height value");
            let xpos: usize = parts[3].parse().expect("Failed to parse x value");
            let ypos: usize = parts[4].split("\n").collect::<Vec<&str>>()[0].parse().expect("Failed to parse y value");

            game = Some(Game::new(width, height, Position::new(xpos, ypos)));
            println!("[II] Game started (width: {}, height: {}, target-position: ({}, {}))", width, height, xpos, ypos);
        }

        if buf.starts_with("lose") {
            let parts: Vec<&str> = buf.split("|").collect();
            let wins: usize = parts[1].parse().expect("Failed to parse win count value");
            let losses: usize = parts[2].split("\n").collect::<Vec<&str>>()[0].parse().expect("Failed to loss count value");
            println!("[II] We lost. ({} wins / {} losses)", wins, losses);

            if !config.flag_continuous {
                done = true;
            }
        }

        if buf.starts_with("win") {
            let parts: Vec<&str> = buf.split("|").collect();
            let wins: usize = parts[1].parse().expect("Failed to parse win count value");
            let losses: usize = parts[2].split("\n").collect::<Vec<&str>>()[0].parse().expect("Failed to loss count value");
            println!("[II] We have won. ({} wins / {} losses)", wins, losses);

            if !config.flag_continuous {
                done = true;
            }
        }

        if buf.starts_with("pos") {
            let parts: Vec<&str> = buf.split("|").collect();

            let xpos: usize = parts[1].parse().expect("Failed to parse x value");
            let ypos: usize = parts[2].parse().expect("Failed to parse y value");
            let has_upper_wall: bool = parts[3].parse::<u8>().expect("Failed to parse has_upper_wall value") > 0;
            let has_right_wall: bool = parts[4].parse::<u8>().expect("Failed to parse has_right_wall value") > 0;
            let has_lower_wall: bool = parts[5].parse::<u8>().expect("Failed to parse has_lower_wall value") > 0;
            let has_left_wall: bool = parts[6].split("\n").collect::<Vec<&str>>()[0].parse::<u8>().expect("Failed to parse has_left_wall value") > 0;

            println!("[II] Environment of current position:\n   {}\n {}   {}\n   {}\n", has_upper_wall, has_left_wall, has_right_wall, has_lower_wall);

            if let Some(game) = game.as_mut() {
                if !game.is_started() {
                    game.start(&Position::new(xpos, ypos)).unwrap();
                }

                current_field_environment = Some( FieldEnvironment::new(has_left_wall, has_right_wall, has_upper_wall, has_lower_wall) );


                if let Some(next_direction) = game.get_next_unvisited_direction(current_field_environment.unwrap()).unwrap() {
                    println!("[II] Moving {:?}", next_direction);
                    let direction = game.move_to(&next_direction).expect("Failed to move");

                    stream.write_all(format!("move|{}\n", direction).as_bytes() ).expect("Failed to transmit move command");
                } else {
                    print!("[II] Moving backward ");
                    let direction = game.move_backwards().expect("Failed to move");
                    println!(" ({:?})", direction);
                    stream.write_all(format!("move|{}\n", direction).as_bytes() ).expect("Failed to transmit move command");
                }
            }

            
        }
    }
    

    stream.shutdown(Shutdown::Both).unwrap();
}