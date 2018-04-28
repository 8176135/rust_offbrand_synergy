extern crate user32;

use std::net::UdpSocket;
use std::io::Write;

fn main() {
    let mut stream = UdpSocket::bind("127.0.0.1:13112").expect("Can't connect to port: ");

    let mut previous_key_state: Vec<bool> = Vec::new();
    previous_key_state.resize(255, false);
    stream.connect("192.168.1.67:13111").expect("Failed to connect");
//    stream.connect("127.0.0.1:13111").expect("Failed to connect");
    loop {
        std::thread::sleep(std::time::Duration::from_millis(20));

        for i in 8..255u8 {
            let mut is_current_key_down: bool;
            unsafe {
                is_current_key_down = user32::GetAsyncKeyState(i as i32) & (1 << 15) != 0;
            }
            if previous_key_state[i as usize] != is_current_key_down {
                previous_key_state[i as usize] = is_current_key_down;
                match i {
                    0x30 ... 0x5A => {
                        stream.send(&[i, is_current_key_down as u8]).expect("Failed to write");
                    },
                    _ => {}
                }
            }
        }
    }

}
