use se_button::{BGMType, SEPlayer};

use midir::{Ignore, MidiInput};
use rodio::Device;

use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref DEVICE: Device = rodio::default_output_device().unwrap();
    static ref SE_DICT: HashMap<u8, Arc<Mutex<SEPlayer>>> = read_dict();
}

fn read_dict() -> HashMap<u8, Arc<Mutex<SEPlayer>>> {
    let mut dict = HashMap::new();
    let mut rdr = csv::Reader::from_path("/home/catupper/Documents/se-button/config.csv").unwrap();
    for result in rdr.records() {
        let record = result.unwrap();
        let key = &record[0];
        let title = &record[1];
        let path = &record[2];
        let volume = &record[3];
        let bgm_type = &record[4];
        dict.insert(
            key.parse().unwrap(),
            Arc::new(Mutex::new(SEPlayer::new(
                &DEVICE,
                title.to_string(),
                path.to_string(),
                volume.parse().unwrap(),
                bgm_type.parse().unwrap(),
            ))),
        );
    }
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

    let mut bgm_playing: Option<u8> = None;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_, message, _| {
            let event = message[0];
            match event {
                144 => {
                    //pressed
                    let key = message[1];
                    if let Some(se_player) = SE_DICT.get(&key) {
                        let mut se_player = se_player.lock().unwrap();
                        match se_player.bgm_type {
                            BGMType::BGM => {
                                if bgm_playing.is_none() {
                                    se_player.play();
                                    bgm_playing = Some(key);
                                }
                            }
                            _ => {
                                se_player.play();
                            }
                        }
                    }
                }
                128 => {
                    //released
                    let key = message[1];
                    if let Some(se_player) = SE_DICT.get(&key) {
                        let mut se_player = se_player.lock().unwrap();
                        if se_player.bgm_type == BGMType::LongSE {
                            se_player.stop();
                        }
                    }
                }
                176 => {
                    //toggle up_or_down
                    let dir = message[1];
                    let pos = message[2];
                    if pos != 127 {
                        return;
                    }
                    if let Some(key) = bgm_playing {
                        let mut se_player = SE_DICT.get(&key).unwrap().lock().unwrap();
                        if dir == 1 {
                            //Up
                            se_player.volume_up();
                        }
                        if dir == 2 {
                            //Down
                            se_player.volume_down();
                        }
                    }
                }
                224 => {
                    //toggle left_or_right
                    let pos = (message[1] as i32) * 128 + (message[2] as i32);
                    if pos != 0 {
                        return;
                    }
                    if let Some(key) = bgm_playing {
                        let mut se_player = SE_DICT.get(&key).unwrap().lock().unwrap();
                        se_player.stop();
                    }
                    bgm_playing = None;
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
