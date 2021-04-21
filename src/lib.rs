use std::
{
    env,
    fs,
    error::Error,
    thread::sleep,
    time::Duration,
    time::Instant,
    thread,
    process::Command,
    path::PathBuf,
};

use rust_gpiozero::*;
use glob::glob;

const DEFAULT_VOLUME: f32           = 1.0;
const FILE_CONFIG: &str             = "config.txt";
const FILE_PATH: &str               = ""; 
const FILE_SOUND_PATH: &str         = "/media"; 
const FILE_SOUND_TYPE: &str         = ".wav"; 
const PIN_MAG: u8                   = 13;
const PIN_LED1: u8                  = 5;
const PIN_LED2: u8                  = 6;

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

#[derive(Clone, Copy, Debug)]
enum State {
    DoorOpenEntering,
    DoorOpenExiting,
    DoorClosed,
}

fn get_state(mag_state: bool, person_detected: bool) -> State {
    match (mag_state, person_detected) {
        (true, true)    => State::DoorOpenExiting,
        (true, false)   => State::DoorOpenEntering,
        (false, false)  => State::DoorClosed,
        (false, true)   => State::DoorClosed,
    }
}

fn get_sound_files(list_snds: &mut Vec<PathBuf>) {
    let files_path = format!("{}{}/*{}", FILE_PATH, FILE_SOUND_PATH, FILE_SOUND_TYPE);
 
    for entry in glob(&files_path).expect("Failed to read files") {
        match entry {
            Ok(path_buf)    => { 
                println!("File: {}", path_buf.display());
                list_snds.push(path_buf); 
            },
            Err(e)      => println!("Error: {:?}", e),
        }
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let config          = Config::new();
	let mut last_state  = State::DoorClosed;
    let mag 		    = InputDevice::new(PIN_MAG);
	let mag_led 	    = LED::new(PIN_LED1);
	let ppl_led 	    = LED::new(PIN_LED2);

    let mut sounds: Vec<PathBuf>  = Vec::new();
    get_sound_files(&mut sounds);

    ppl_led.on();
    mag_led.on();
    
	sleep(Duration::from_secs(1));

	ppl_led.off();
	mag_led.off();
	
	loop {
        let current_state   = get_state(mag.is_active(), false);
        let random_sound    = fastrand::usize(..sounds.len());
        let sound_path      = sounds
                                .get_mut(random_sound)
                                .unwrap();

        let sound_path_str  = sound_path
                                .to_str()
                                .unwrap();

        match (current_state, last_state) {
            (State::DoorOpenEntering, State::DoorClosed) => {
                println!("entering: {:?} : {:?}", current_state, Command::new("aplay").arg(sound_path_str).status());
                
                mag_led.on();
			   	sleep(Duration::from_millis(100));
            },
            (State::DoorOpenExiting, State::DoorClosed) => {
                println!("entering: {:?} : {:?}", current_state, Command::new("aplay").arg(sound_path_str).status());

                mag_led.on();
                ppl_led.on();
			   	sleep(Duration::from_millis(100));
            },
            (State::DoorClosed, _) => {
                // println!("door closed");
                mag_led.off();
                ppl_led.off();
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
