use std::process;

fn main() {
    if let Err(e) = door_trigger::run() {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
<<<<<<< HEAD

/*
use soloud::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sl = Soloud::default()?;
    sl.set_global_volume(0.1);

    let mut speech = audio::Speech::default();

    speech.set_text("Welcome to Sunmerry")?;

    sl.play(&speech);
    while sl.active_voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let mut wav = audio::Wav::default();
    wav.load(&std::path::Path::new("exit_stars.wav"))?;

    sl.play(&wav);
    while sl.voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Ok(())
}
*/
=======
>>>>>>> 68fd1919d1ea1a835627c76cec15dd449cc25da9
