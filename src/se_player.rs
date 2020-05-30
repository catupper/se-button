use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;

use rodio::{Device, Sink};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BGMType {
    BGM,
    ShortSE,
    LongSE,
}

impl FromStr for BGMType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s {
            "BGM" => BGMType::BGM,
            "ShortSE" => BGMType::ShortSE,
            "LongSE" => BGMType::LongSE,
            _ => panic!(),
        };
        Ok(res)
    }
}

pub struct SEPlayer {
    pub sink: Sink,
    pub device: &'static Device,
    pub sound_path: String,
    pub title: String,
    pub volume: f32,
    pub bgm_type: BGMType,
}

impl SEPlayer {
    pub fn new(
        device: &'static Device,
        title: String,
        sound_path: String,
        volume: f32,
        bgm_type: BGMType,
    ) -> Self {
        let sink = Sink::new(device);
        sink.set_volume(volume);
        Self {
            sink,
            device,
            sound_path,
            title,
            volume,
            bgm_type,
        }
    }

    pub fn play(&mut self) {
        println!("Play '{}' Volume:{}'", self.title, self.sink.volume());
        self.sink = Sink::new(self.device);
        self.sink.set_volume(self.volume);
        let file = File::open(self.sound_path.to_string()).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        self.sink.play();
        self.sink.append(source);
    }

    pub fn stop(&mut self) {
        println!("Stop {}", self.title);
        self.sink.stop();
    }

    pub fn volume_up(&mut self) {
        if self.volume < 1.3 {
            self.volume += 0.05;
        }
        self.sink.set_volume(self.volume);
        println!("Volume Up! Set Volume {}", self.volume);
    }

    pub fn volume_down(&mut self) {
        if self.volume > 0.07 {
            self.volume -= 0.05;
        }
        self.sink.set_volume(self.volume);
        println!("Volume Down! Set Volume {}", self.volume);
    }
}
