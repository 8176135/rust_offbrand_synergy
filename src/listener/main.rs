extern crate enigo;

use std::net::TcpListener;
use std::io::Read;

use enigo::{Enigo, KeyboardControllable, Key};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:13111").expect("Error binding to port: ");
    let mut data= [0u8];
    let mut enigo = Enigo::new();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        loop {
            stream.read(&mut data).unwrap();
            println!("Data: {}", data[0] as char);
            enigo.key_click(Key::Layout(data[0] as char))
        }
    }
}
