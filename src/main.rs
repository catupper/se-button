use midir::{Ignore, MidiInput};

use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex};

#[macro_use]
extern crate lazy_static;

struct SEPlayer {
    sound_name: String,
    cnt: i32,
}

impl SEPlayer {
    pub fn new(sound_name: String) -> Self {
        Self { sound_name, cnt: 0 }
    }

    pub fn play(&mut self) {
        println!("Play {} {}", self.sound_name, self.cnt);
        self.cnt += 1
    }

    pub fn stop(&mut self) {
        println!("Stop {} {}", self.sound_name, self.cnt);
        self.cnt += 1
    }
}

lazy_static! {
    static ref SE_DICT: HashMap<u8, Arc<Mutex<SEPlayer>>> = read_dict();
}

fn read_dict() -> HashMap<u8, Arc<Mutex<SEPlayer>>> {
    let mut dict = HashMap::new();
    dict.insert(60, Arc::new(Mutex::new(SEPlayer::new("Do".to_string()))));
    dict.insert(61, Arc::new(Mutex::new(SEPlayer::new("Re".to_string()))));
    dict
}

fn main() -> Result<(), Box<dyn Error>> {
    run()
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_, message, _| {
            let event = message[0];
            let key = message[1];
            match event {
                128 => {
                    //pressed
                    if let Some(se_player) = SE_DICT.get(&key) {
                        se_player.lock().unwrap().play();
                    }
                }
                144 => {
                    //released
                    if let Some(se_player) = SE_DICT.get(&key) {
                        se_player.lock().unwrap().stop();
                    }
                }
                _ => {}
            }
        },
        (),
    )?;

    println!(
        "Connection open, reading input from '{}' (press enter to exit) ...",
        in_port_name
    );

    let mut input = String::new();
    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}
