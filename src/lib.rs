use std::error::Error;
use rust_gpiozero::*;
use soloud::*;

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

pub fn run() -> Result<(), Box<dyn Error>> {
	let mut last_state  = State::DoorClosed;
	let mag 		    = InputDevice::new(23);
	let pir 		    = InputDevice::new(24);
	let mut mag_led 	= LED::new(25);
	let pir_led 	    = LED::new(18);
	
	pir_led.off();
	mag_led.off();
	
	let mut enter = audio::Wav::default();
	enter.load(&std::path::Path::new("/home/pi/workspace/rust-pi-door-chime/media/enter.wav")).unwrap_or_else( |err| {
		mag_led.blink(2.0, 3.0);
		println!("Unable to load .wav file: {:?}", err);
		loop {}
	});

	let mut exit = audio::Wav::default();
	exit.load(&std::path::Path::new("/home/pi/workspace/rust-pi-door-chime/media/exit.wav")).unwrap_or_else( |err| {
		mag_led.blink(2.0, 3.0);
		println!("Unable to load .wav file: {:?}", err);
		loop {}
	});

	let sl = Soloud::default().unwrap_or_else( |err| {
		mag_led.blink(2.0, 3.0);
		println!("Unable to create soloud: {:?}", err);
		loop {}
	});

	loop {
        let current_state = get_state(mag.is_active(), pir.is_active());

        println!("last state: {:?}", last_state);
        match (current_state, last_state) {
            (State::DoorOpenEntering, State::DoorClosed) => {
                println!("entering");
                sl.play(&enter);
                mag_led.on();

			    while sl.voice_count() > 0 {	
			    	std::thread::sleep(std::time::Duration::from_millis(100));
			    }
            },
            (State::DoorOpenExiting, State::DoorClosed) => {
                println!("exiting");
                sl.play(&exit);
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
            _ => println!("Do nothing as door is already open"),
        }

        last_state = current_state;
	}
}
