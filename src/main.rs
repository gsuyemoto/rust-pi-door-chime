use std::process;
use door_trigger::run;

fn main() {
    if let Err(e) = run() {
        println!("Application error: {}", e);

        process::exit(1);
    }
}
