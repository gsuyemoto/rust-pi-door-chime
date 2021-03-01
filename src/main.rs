use rust_gpiozero::*;
use soloud::*;

fn main() {
	let mut play_once	= true;
	let door 	= InputDevice::new(23);
	let mut led 	= LED::new(25);
	led.off();
	
	let mut wav	= audio::Wav::default();
	wav.load(&std::path::Path::new("/home/pi/door_trigger/sunmerry01.wav")).unwrap_or_else( |err| {
		led.blink(2.0, 3.0);
		println!("Unable to load .wav file: {:?}", err);
		loop {}
	});

	let sl 		= Soloud::default().unwrap_or_else( |err| {
		led.blink(2.0, 3.0);
		println!("Unable to create soloud: {:?}", err);
		loop {}
	});

	// let mut speech	= audio::Speech::default();

	// speech.set_text("Welcome to Sunmerry!")?;

	loop {
		if door.is_active() && play_once {
			println!("playing sound");
			play_once = false;
			led.on();
			sl.play(&wav);

			// sl.play(&speech);
			// while sl.active_voice_count() > 0 {	

			while sl.voice_count() > 0 {	
				std::thread::sleep(std::time::Duration::from_millis(100));
			}
		} else if !door.is_active() && !play_once {
			println!("resetting sound");
			play_once = true;
			led.off();
		}
	}
}
