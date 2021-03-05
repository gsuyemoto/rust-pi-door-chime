use std::error::Error;
use std::vec::Vec;
use std::path::Path;
use rust_gpiozero::*;
use soloud::*;
use glob::glob;
use fastrand::*;

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

fn get_sound_files(list_snds: &mut Vec<Wav>, search: &str) {
    let files_path = format!("/home/pi/workspace/rust-pi-door-chime/media/*{}*.wav", search);
    
    for entry in glob(&files_path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path_buf)    => { 
                let mut new_sound = Wav::default();
                new_sound.load(&path_buf.as_path());

                list_snds.push(new_sound); 
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

    let mut snds_enter  = Vec::new();
    let mut snds_exit   = Vec::new();       	

    get_sound_files(&mut snds_enter, "enter");
    get_sound_files(&mut snds_exit, "exit");

	pir_led.off();
	mag_led.off();
	
	let sl = Soloud::default()?;

	loop {
        let current_state   = get_state(mag.is_active(), pir.is_active());
        let random_enter    = fastrand::usize(..snds_enter.len());
        let random_exit     = fastrand::usize(..snds_exit.len());

        println!("last state: {:?}", last_state);
        match (current_state, last_state) {
            (State::DoorOpenEntering, State::DoorClosed) => {
                println!("entering");
                sl.play(&snds_enter[random_enter]);
                mag_led.on();

			    while sl.voice_count() > 0 {	
			    	std::thread::sleep(std::time::Duration::from_millis(100));
			    }
            },
            (State::DoorOpenExiting, State::DoorClosed) => {
                println!("exiting");
                sl.play(&snds_exit[random_exit]);
                mag_led.on();
                pir_led.on();
			    
                while sl.voice_count() > 0 {	
			    	std::thread::sleep(std::time::Duration::from_millis(100));
			    }
            },
            (State::DoorClosed, _) => {
                println!("reset");
                mag_led.off();
                pir_led.off();
            },
            _ => println!("door never closed, do nothing"),
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
