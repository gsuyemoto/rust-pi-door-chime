use std::error::Error;
use std::vec::Vec;
use rust_gpiozero::*;
use soloud::*;
use glob::glob;

#[derive(Clone, Copy, Debug)]
enum State {
    DoorOpenEntering,
    DoorOpenExiting,
    DoorClosed,
}

fn get_state(mag_state: bool, pir_state: bool) -> State {
    match (mag_state, pir_state) {
        (true, true)    => State::DoorOpenExiting,
        (true, false)   => State::DoorOpenEntering,
        (false, false)  => State::DoorClosed,
        (false, true)   => State::DoorClosed,
    }
}

struct Sound( Option<Wav>, Option<Speech>);

fn get_sound_files(list_snds: &mut Vec<Sound>, search: &str) {
    let files_path      = format!("/mnt/usbdrive/media/*{}*.wav", search);
    let all_files       = glob(&files_path).expect("Failed to read files");  
 
    if all_files.size_hint().1.is_none() { return; } 

    for entry in all_files {
        match entry {
            Ok(path_buf)    => { 
                let mut new_sound = Wav::default();
                new_sound.load(&path_buf.as_path()).expect("Unable to load a sound file");

                list_snds.push(Sound(Some(new_sound), None)); 
                println!("File: {:?}", path_buf.display()); 
            },
            Err(e)      => println!("Error: {:?}", e),
        }
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
	let mut last_state  = State::DoorClosed;
	
    let mag 		    = InputDevice::new(23);
	let pir 		    = InputDevice::new(12);
	let mag_led 	    = LED::new(25);
	let pir_led 	    = LED::new(18);

    let mut snds_enter: Vec<Sound>  = Vec::new();
    let mut snds_exit: Vec<Sound>   = Vec::new();       	

    get_sound_files(&mut snds_enter, "enter");
    get_sound_files(&mut snds_exit, "exit");

    if snds_enter.len() == 0 {
        let mut speech = Speech::default();
        speech.set_text("You're entering Sunmerry!")?;
        snds_enter.push(Sound(None, Some(speech)));
    }

    if snds_exit.len() == 0 {
        let mut speech = Speech::default();
        speech.set_text("Have a great day!")?;
        snds_exit.push(Sound(None, Some(speech)));
    }

	pir_led.off();
	mag_led.off();
	
	let sl = Soloud::default()?;

	loop {
        let current_state   = get_state(mag.is_active(), pir.is_active());
        let random_enter    = fastrand::usize(..snds_enter.len());
        let random_exit     = fastrand::usize(..snds_exit.len());

        match (current_state, last_state) {
            (State::DoorOpenEntering, State::DoorClosed) => {
                println!("entering");
                match &snds_enter[random_enter].0 {
                    Some(sound)     => sl.play(sound),
                    None            => sl.play(snds_enter[random_enter].1.as_ref().unwrap()),
                };
                
                mag_led.on();

			    while sl.voice_count() > 0 {	
			    	std::thread::sleep(std::time::Duration::from_millis(100));
			    }
            },
            (State::DoorOpenExiting, State::DoorClosed) => {
                println!("exiting: {:?} : {:?}", current_state, last_state);
                match &snds_exit[random_exit].0 {
                    Some(sound)     => sl.play(sound),
                    None            => sl.play(snds_exit[random_exit].1.as_ref().unwrap()),
                };

                mag_led.on();
                pir_led.on();
			    
                while sl.voice_count() > 0 {	
			    	std::thread::sleep(std::time::Duration::from_millis(100));
			    }
            },
            (State::DoorClosed, _) => {
                // println!("reset");
                mag_led.off();
                pir_led.off();
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
