use std::
{
    env,
    fs,
    error::Error,
    thread::sleep,
    time::Duration,
    time::Instant,
    thread,
    sync::mpsc,
};

use rust_gpiozero::*;
use soloud::*;
use glob::glob;

const DEFAULT_VOLUME: f32           = 1.0;
const DEFAULT_DISTANCE: u128        = 60;
const FILE_CONFIG: &str             = "config.txt";
const FILE_PATH: &str               = "/mnt/usbdrive"; 
const FILE_SOUND_PATH: &str         = "/media"; 
const FILE_SOUND_TYPE: &str         = ".wav"; 
const FILE_SOUND_ENTER: &str        = "enter"; 
const FILE_SOUND_EXIT: &str         = "exit"; 
const FILE_SOUND_GREETING: &str     = "You're entering Sunmerry!"; 
const FILE_SOUND_PARTING: &str      = "Have a great day!"; 

pub struct Config {
    pub volume: f32,
    pub distance: u128,
}

impl Config {
    pub fn new() -> Config {
        let mut args_vec: Vec<String> = env::args().collect();

        if args_vec.len() < 2 {
            args_vec = match fs::read_to_string(format!("{}/{}", FILE_PATH, FILE_CONFIG)) {
                Ok(contents)    => contents.split_whitespace().map(str::to_string).collect(),
                Err(_)          => vec![DEFAULT_VOLUME.to_string(), DEFAULT_DISTANCE.to_string()],
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

        let distance: u128  = match args_vec[1].parse::<u128>() {
            Ok(arg)         => arg,
            Err(e)          => {
                                    println!("Error parsing distance: {}", e);
                                    DEFAULT_DISTANCE
                            },
        };

        println!("Volume: {}", volume);
        println!("Distance: {}", distance);

        Config {
            volume,
            distance,
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

fn median(mut readings: Vec<u128>) -> u128 {
    readings.sort();
    let mid = readings.len() /2;
    readings[mid]
}

struct DetectPerson {
    worker:             Option<thread::JoinHandle<()>>,
    person_detected:    bool,
}

impl DetectPerson {
    fn new(min_distance: u128, pin_out: OutputDevice, pin_in: InputDevice, sender: mpsc::Sender<bool>) -> DetectPerson {
        let mut last_ten: Vec<u128> = Vec::with_capacity(10);

        let worker = thread::spawn(move || loop {
            let distance = Self::get_sensor_readings(pin_out, pin_in);
            last_ten.push(distance);

            if last_ten.len() == 10 {
                let median = median(last_ten.clone());
                sender.send(median < min_distance);
                last_ten.pop();
                println!("median: {}", median);
            }

            // wait 60 ms between measurements
            sleep(Duration::from_millis(60));
        });

        DetectPerson {
            worker: Some(worker),
            person_detected: false,
        }
    }

    fn get_sensor_readings(pin_out:OutputDevice, pin_in: InputDevice) -> u128 {
        // send sonic
        pin_out.on();
        sleep(Duration::from_micros(10));
        pin_out.off();
        
        // measure
        let check_fail      = Instant::now();
        let mut did_fail    = false;
        while !pin_in.is_active() { 
            if check_fail.elapsed().as_micros() > 1700 {
                did_fail = true;
                break; 
            }                
        }
        
        if did_fail {
            println!("Failed...");
            sleep(Duration::from_millis(60));
            return 0
        }
        
        let time_start      = Instant::now();
        
        while pin_in.is_active() {}
        let time_elapsed    = time_start.elapsed().as_micros();
        let distance        = time_elapsed / 148; // for inches
        // println!("Distance: {:?}", distance);
        
        distance        
    }
}


struct Sound( Option<Wav>, Option<Speech>);

fn get_sound_files(list_snds: &mut Vec<Sound>, search: &str) {
    let files_path = format!("{}{}/{}*{}", FILE_PATH, FILE_SOUND_PATH, search, FILE_SOUND_TYPE);
 
    for entry in glob(&files_path).expect("Failed to read files") {
        match entry {
            Ok(path_buf)    => { 
                println!("File: {}", path_buf.display());

                let mut new_sound = Wav::default();
                new_sound.load(&path_buf.as_path()).expect("Unable to load a sound file");

                list_snds.push(Sound(Some(new_sound), None)); 
            },
            Err(e)      => println!("Error: {:?}", e),
        }
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let config          = Config::new();
	let mut last_state  = State::DoorClosed;
	
    let trig            = OutputDevice::new(5);
    let echo            = InputDevice::new(6);
    let mag 		    = InputDevice::new(23);

	let mag_led 	    = LED::new(25);
	let ppl_led 	    = LED::new(18);

    let mut snds_enter: Vec<Sound>  = Vec::new();
    let mut snds_exit: Vec<Sound>   = Vec::new();       	

    get_sound_files(&mut snds_enter, FILE_SOUND_ENTER);
    get_sound_files(&mut snds_exit, FILE_SOUND_EXIT);

    if snds_enter.len() == 0 {
        let mut speech = Speech::default();
        speech.set_text(FILE_SOUND_GREETING)?;
        snds_enter.push(Sound(None, Some(speech)));
    }

    if snds_exit.len() == 0 {
        let mut speech = Speech::default();
        speech.set_text(FILE_SOUND_PARTING)?;
        snds_exit.push(Sound(None, Some(speech)));
    }
    
    ppl_led.on();
    mag_led.on();
    
	sleep(Duration::from_secs(1));

	ppl_led.off();
	mag_led.off();
	
	let mut sl = Soloud::default().expect("Unable to create Soloud");
    sl.set_global_volume(config.volume);

    let (tx, rx)    = mpsc::channel(); 
    let _           = DetectPerson::new(
                        config.distance,
                        trig,
                        echo,
                        tx );

	loop {
        let person_was_detected = match rx.try_recv() {
            Ok(msg)     => msg,
            Err(_)      => false,
        };

        let current_state   = get_state(mag.is_active(), person_was_detected);
        let random_enter    = fastrand::usize(..snds_enter.len());
        let random_exit     = fastrand::usize(..snds_exit.len());

        match (current_state, last_state) {
            (State::DoorOpenEntering, State::DoorClosed) => {
                // println!("entering: {:?} : {:?}", current_state, last_state);
                match &snds_enter[random_enter].0 {
                    Some(sound)     => sl.play(sound),
                    None            => sl.play(snds_enter[random_enter].1.as_ref().unwrap()),
                };
                
                mag_led.on();

			    while sl.voice_count() > 0 {	
			    	sleep(Duration::from_millis(100));
			    }
            },
            (State::DoorOpenExiting, State::DoorClosed) => {
                // println!("exiting: {:?} : {:?}", current_state, last_state);
                match &snds_exit[random_exit].0 {
                    Some(sound)     => sl.play(sound),
                    None            => sl.play(snds_exit[random_exit].1.as_ref().unwrap()),
                };

                mag_led.on();
                ppl_led.on();
			    
                while sl.voice_count() > 0 {	
			    	sleep(Duration::from_millis(100));
			    }
            },
            (State::DoorClosed, _) => {
                // println!("door closed");
                mag_led.off();
                ppl_led.off();
            },
            _ => continue,
        }

        last_state = current_state;
	} 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_sound_empty_string() {
        
    }
}
