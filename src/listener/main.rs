extern crate enigo;

use std::net::UdpSocket;
use std::io::Read;

use enigo::{Enigo, KeyboardControllable, Key};

fn main() {
    let listener = UdpSocket::bind("0.0.0.0:13111").expect("Error binding to port: ");
    let mut data = [0u8; 2];
    let mut enigo = Enigo::new();

    loop {
        listener.recv(&mut data).unwrap();
        println!("Data: {}", data[0] as char);
        if data[1] == 0 {
            enigo.key_up(Key::Layout(data[0] as char))
        } else {
            enigo.key_down(Key::Layout(data[0] as char))
        }
    }
}
