use std::error::Error;
use std::vec::Vec;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use rust_gpiozero::*;
use soloud::*;
use glob::glob;

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

fn detect_person(trig: &mut OutputDevice, echo: &InputDevice) -> bool {
    // send sonic
    trig.on();
    sleep(Duration::from_micros(10));
    trig.off();
    
    // measure
    let check_fail      = Instant::now();
    let mut did_fail    = false;
    while !echo.is_active() { 
        if check_fail.elapsed().as_micros() > 17000 {
            did_fail = true;
            break; 
        }                
    }
    
    if did_fail {
        println!("Failed...");
        sleep(Duration::from_millis(60));
        return false
    }
    
    let time_start      = Instant::now();
    
    while echo.is_active() {}
    let time_elapsed    = time_start.elapsed().as_micros();
    println!("Time elapsed: {:?}", time_elapsed);
    
    let distance        = time_elapsed / 148;
    println!("Distance: {:?}", distance);
    
    // wait 60 ms between measurements
    sleep(Duration::from_millis(60));
    
    if distance < 30 { return true }
    else { return false }
}

struct Sound( Option<Wav>, Option<Speech>);

fn get_sound_files(list_snds: &mut Vec<Sound>, search: &str) {
    let files_path = format!("/mnt/usbdrive/media/{}*.wav", search);
 
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
	let mut last_state  = State::DoorClosed;
	
    let mut trig        = OutputDevice::new(5);
    let echo            = InputDevice::new(6);
    let mag 		    = InputDevice::new(23);

	let mag_led 	    = LED::new(25);
	let ppl_led 	    = LED::new(18);

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
    
    ppl_led.on();
    mag_led.on();
    
	sleep(Duration::from_secs(1));

	ppl_led.off();
	mag_led.off();
	
	let sl = Soloud::default().expect("Unable to create Soloud");

	loop {
        let current_state   = get_state(mag.is_active(), detect_person(&mut trig, &echo));
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
