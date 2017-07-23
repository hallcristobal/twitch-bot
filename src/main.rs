pub mod connection;
pub mod client;

use client::Client;
use std::thread;
use std::sync::Arc;
use std::io::ErrorKind;

pub fn main() {
    let pass = Arc::new("OAUTH");
    let usr_name = Arc::new("USERNAME");

    let handle = thread::spawn(move || { run(&usr_name, &pass); });

    handle.join().expect("Couldn't join thread");
}

pub fn run(u: &str, p: &str) {
    let conn = Client::new(u, p, "127.0.0.1", 6667);
    conn.login().unwrap();
    for msg in conn.iter() {
        match msg {
            Ok(msg) => {
                println!("{}", msg);
            }
            Err(ref err) if err.kind() == ErrorKind::NotConnected => {
                break;
            }
            Err(_) => {
                println!("Error was caught");
            }
        }
    }
}
