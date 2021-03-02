use rust_gpiozero::*;
use soloud::*;

fn main() {
	let mut play_once	= true;
	let mag 		    = InputDevice::new(23);
	let pir 		    = InputDevice::new(24);
	let mut mag_led 	= LED::new(25);
	let mut pir_led 	= LED::new(18);
	
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
		if mag.is_active() && !pir.is_active() && play_once {
			println!("entering");
			play_once = false;
			mag_led.on();
			sl.play(&enter);

			while sl.voice_count() > 0 {	
				std::thread::sleep(std::time::Duration::from_millis(100));
			}
		} else if mag.is_active() && pir.is_active() && play_once {
			println!("exiting");
			play_once = false;
			mag_led.on();
            pir_led.on();
			sl.play(&exit);

			while sl.voice_count() > 0 {	
				std::thread::sleep(std::time::Duration::from_millis(100));
			}
		} else if !mag.is_active() && !play_once {
			println!("resetting sound");
			play_once = true;
			mag_led.off();
            pir_led.off();
		}
	}
}
