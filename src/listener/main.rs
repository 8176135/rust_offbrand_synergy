extern crate enigo;

mod win_key_codes;

use win_key_codes::*;
use std::net::UdpSocket;
use std::io::Read;

use enigo::{Enigo, KeyboardControllable, MouseControllable, Key, MouseButton};

fn main() {
    let listener = UdpSocket::bind("0.0.0.0:13111").expect("Error binding to port: ");
    let mut data = [0u8; 3];
    //let mut extra_data = [0u8; 1];
    let mut enigo = Enigo::new();

    loop {
        listener.recv(&mut data).unwrap();
        println!("Code: {:X}", data[0]);

        if data[0] < 8 { // Mouse event
            let mouse_btn_to_press = match data[0] {
                0 => MouseButton::Left,
                1 => MouseButton::Right,
                2 => MouseButton::Middle,
                _ => continue
            };
            if data[1] == 0 {
                println!("Down: {:?}", data[0]);
                enigo.mouse_down(mouse_btn_to_press);
            } else {
                println!("Up: {:?}", mouse_btn_to_press);
                enigo.mouse_up(mouse_btn_to_press);
            }
        } else if data[0] == 0xFF {
//            let delta_x = (((data[1] & 0xF) as i8) << 4) >> 4;
//            let delta_y = ((data[1] as i8) >> 4);

            //println!("{}, {}", data[1] as i32,  data[2] as i32);
            enigo.mouse_move_relative(data[1] as i32,  data[2] as i32);
        } else { // or keyboard event
            let key_to_press = match data[0] {
                0x12...0x2B => Key::Layout((data[0] + 0x2F) as char),
                0x8...0x10 => Key::Layout((data[0] + 0x29) as char),
                0x11 => Key::Layout('0'),
                0x6E | 0x87 => Key::Control,
                0x6F | 0x88 => Key::Alt,
                0x70 | 0x89 => Key::Shift,
                0x91 => Key::Tab,
                0x4A => Key::Return,
                0x69 => Key::Layout('`'),
                0x64 => Key::Layout(','),
                0x81 => Key::Layout('.'),
                0x8C => Key::Layout('/'),
                0x8B => Key::Layout(';'),
                0x60 => Key::Layout('\\'),
                0x49 => Key::Backspace,
                0x2C => Key::Escape,
//                VK_NUMPAD0...VK_NUMPAD9 => Key::Layout((data[0] - 0x30) as char),
//                VK_SPACE => Key::Space,
//                VK_BACK => Key::Backspace,
//                VK_TAB => Key::Tab,
//                VK_ESCAPE => Key::Escape,
//                VK_RETURN => Key::Return,
//                VK_LEFT => Key::LeftArrow,
//                VK_RIGHT => Key::RightArrow,
//                VK_UP => Key::UpArrow,
//                VK_DOWN => Key::DownArrow,
//                VK_SHIFT => Key::Shift,
//                VK_CONTROL => Key::Control,
//                VK_LMENU => Key::Alt,
//                VK_HOME => Key::Home,
//
//                VK_DELETE => Key::Raw(83), // Linux only
//                VK_PRIOR => Key::PageUp,
//                VK_NEXT => Key::PageDown,
//                //VK_OEM_1 => Key::Layout('['),
//                VK_OEM_2 => Key::Layout('/'),
//                VK_OEM_3 => Key::Layout('`'),
//                VK_OEM_4 => Key::Layout('['),
//                VK_OEM_5 => Key::Layout('\\'),
//                VK_OEM_6 => Key::Layout(']'),
//                VK_OEM_7 => Key::Layout('\''),
//                VK_OEM_PLUS => Key::Layout('+'),
//                VK_OEM_MINUS => Key::Layout('-'),
//                VK_OEM_COMMA => Key::Layout(','),
//                VK_OEM_PERIOD => Key::Layout('.'),
                _ => {
                    println!("Key code {:X} not supported", data[0]);
                    continue;
                }
            };

//        println!("Data: {:?}", key_to_press);
            if data[1] == 1 {
                println!("Up: {:?}", key_to_press);
                enigo.key_up(key_to_press)
            } else {
                println!("Down: {:?}", key_to_press);
                enigo.key_down(key_to_press)
            }
        }
    }
}
