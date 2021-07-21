use std::
{
    env,
    fs,
    error::Error,
    thread::sleep,
    time::Duration,
    time::Instant,
    thread,
    path::PathBuf,
};

use std::io;
use std::io::prelude::*;

use soloud::*;
use rust_gpiozero::*;
use glob::glob;
use std::sync::atomic::{AtomicU8, Ordering};

const DEFAULT_VOLUME: f32           = 0.3;
const FILE_CONFIG: &str             = "config.txt";
const FILE_PATH: &str               = ""; 
const FILE_SOUND_PATH: &str         = "/home/pi/media"; 
const FILE_SOUND_TYPE: &str         = ".wav"; 

const PIN_PLAY: u8                  = 6;
const PIN_MAG: u8                   = 13;
const PIN_LED: u8                   = 5;
const PIN_VUP: u8                   = 16;
const PIN_VDOWN: u8                 = 26;

const WINNER_SOUND: &str            = "winner";
const WINNER_NUM: u32               = 50;
const TIME_MAX: u64                 = 120; // time since last door open

const BTN_TYPE_NOTHING: u8          = 0;
const BTN_TYPE_PLAY: u8             = 1;
const BTN_TYPE_VUP: u8              = 2;
const BTN_TYPE_VDOWN: u8            = 3;

static BTN_PRESSED: AtomicU8 = AtomicU8::new(BTN_TYPE_NOTHING);

pub struct Config {
    pub volume: f32,
}

impl Config {
    pub fn new() -> Config {
        let mut args_vec: Vec<String> = env::args().collect();

        if args_vec.len() < 2 {
            args_vec = match fs::read_to_string(format!("{}/{}", FILE_PATH, FILE_CONFIG)) {
                Ok(contents)    => contents.split_whitespace().map(str::to_string).collect(),
                Err(_)          => vec![DEFAULT_VOLUME.to_string()],
            };
        }
        else {
            // command line args starts with name of runtime
            args_vec.remove(0);
        }

        let volume: f32     = match args_vec[0].parse::<f32>() {
            Ok(arg)         => arg,
            Err(e)          => {
                                    println!("Error parsing volume: {}", e);
                                    DEFAULT_VOLUME
                            },
        };

        println!("Volume: {}", volume);

        Config {
            volume,
        }
    }
}

struct Sounds {
    player: Soloud,
    counter: u32,
    winner: Wav,
    loser: Wav,
    time: Instant,
}

impl Sounds {
    fn new(volume: f32) -> Self {
        let mut soloud      = Soloud::default().unwrap();
        soloud.set_global_volume(volume);

        let files_path      = format!("{}/*{}", FILE_SOUND_PATH, FILE_SOUND_TYPE);
        let mut wav_win     = audio::Wav::default();
        let mut wav_lose    = audio::Wav::default();

        for entry in glob(&files_path).expect("Error globbing path") {
            if let Ok(path) = entry {
                if path.to_str().unwrap().contains(WINNER_SOUND) {
                    println!("Loading winner sound... {}", path.display());
                    wav_win.load(&path
                        .as_path())
                        .expect("Error loading sound");
                }
                else {
                    println!("Loading loser sound... {}", path.display());
                    wav_lose.load(&path
                        .as_path())
                        .expect("Error loading sound");
                }
            }
            else {
                println!("Path not globbed correctly");
            }
        }

        Sounds { 
            player: soloud,
            counter: 0,
            winner: wav_win, 
            loser: wav_lose,
            time: Instant::now(),
        }
    }

    fn play(&mut self) {
        // check amount of time since last play
        // if over TIME_MAX then store just opened
        // therefore need to set counter to new
        // random counter to mix up contest
        let time_max        = Duration::from_secs(TIME_MAX);

        if self.time.elapsed() > time_max {
            self.counter    = fastrand::u32(1..WINNER_NUM);
            self.time       = Instant::now();
        }
        else {
            self.counter    += 1;
        }

        if self.counter == WINNER_NUM {
            println!("Found a winner!");

            self.counter = 0;
            self.player.play(&self.winner);
        }
        else {
            println!("Another loser... {}", self.counter);

            self.player.play(&self.loser);
        }

        while self.player.active_voice_count() > 0 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn volume_up(&mut self) {
        let current_volume = self.player.global_volume();
        self.player.set_global_volume(current_volume + 0.1);
    }

    fn volume_down(&mut self) {
        let current_volume = self.player.global_volume();
        self.player.set_global_volume(current_volume - 0.1);
    }
}

fn watch_btn_play() {
    // wait for key press to play music file manually
    let mut button = Button::new(PIN_PLAY);
    button.wait_for_press(None);
    BTN_PRESSED.store(BTN_TYPE_PLAY, Ordering::Relaxed);
}

fn watch_btn_vup() {
    // wait for key press to play music file manually
    let mut button = Button::new(PIN_VUP);
    button.wait_for_press(None);
    BTN_PRESSED.store(BTN_TYPE_VUP, Ordering::Relaxed);
}

fn watch_btn_vdown() {
    // wait for key press to play music file manually
    let mut button = Button::new(PIN_VDOWN);
    button.wait_for_press(None);
    BTN_PRESSED.store(BTN_TYPE_VDOWN, Ordering::Relaxed);
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let config          = Config::new();
	let mut last_state  = false;
    let mag 		    = InputDevice::new(PIN_MAG);
	let mag_led 	    = LED::new(PIN_LED);
    let mut sounds      = Sounds::new(config.volume);

    std::thread::spawn(watch_btn_play);
    std::thread::spawn(watch_btn_vup);
    std::thread::spawn(watch_btn_vdown);
	
    println!("Starting loop...");
	loop {
        match BTN_PRESSED.load(Ordering::Relaxed) {
            BTN_TYPE_PLAY   =>  {
                                   sounds.play();
                                   BTN_PRESSED.store(BTN_TYPE_NOTHING, Ordering::Relaxed);
                                },
            BTN_TYPE_VUP    =>  {
                                   sounds.volume_up();
                                   BTN_PRESSED.store(BTN_TYPE_NOTHING, Ordering::Relaxed);
                                },
            BTN_TYPE_VDOWN  =>  {
                                   sounds.volume_down();
                                   BTN_PRESSED.store(BTN_TYPE_NOTHING, Ordering::Relaxed);
                                },
            _               =>  (),                    
        }

        let current_state = mag.is_active();
        match (current_state, last_state) {
            (true, false) => {
                println!("Door opened");
                
                sounds.play();    
            },
            _ => continue,
        }

        last_state = current_state;
        sleep(Duration::from_secs(3));
	} 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_sound_empty_string() {
        
    }
}
