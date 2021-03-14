use std::process;

fn main() {
    if let Err(e) = door_trigger::run() {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
