extern crate enigo;

mod win_key_codes;

use win_key_codes::*;
use std::net::UdpSocket;
use std::io::Read;

use enigo::{Enigo, KeyboardControllable, Key, MouseButton};

fn main() {
    let listener = UdpSocket::bind("0.0.0.0:13111").expect("Error binding to port: ");
    let mut data = [0u8; 2];
    let enigo = Enigo::new();

    loop {
        listener.recv(&mut data).unwrap();
        println!("Code: {:X}", data[0]);

        if data[0] < 8 { // Mouse event
            let mouse_btn_to_press = match data[0] {
                VK_LBUTTON => MouseButton::Left,
                VK_RBUTTON => MouseButton::Right,
                VK_MBUTTON => MouseButton::Middle,
                _ => continue
            };
            if data[1] == 0 {
                println!("Click: {:?}", data[0]);
                //enigo.mouse_down(mouse_btn_to_press);
            } else {
                println!("Up: {:?}", mouse_btn_to_press);
                //enigo.mouse_up(mouse_btn_to_press);
            }
        } else { // or keyboard event
            let key_to_press = match data[0] {
                0x30...0x5A => Key::Layout(data[0] as char),
                VK_NUMPAD0...VK_NUMPAD9 => Key::Layout((data[0] - 0x30) as char),
                VK_SPACE => Key::Space,
                VK_BACK => Key::Backspace,
                VK_TAB => Key::Tab,
                VK_ESCAPE => Key::Escape,
                VK_RETURN => Key::Return,
                VK_LEFT => Key::LeftArrow,
                VK_RIGHT => Key::RightArrow,
                VK_UP => Key::UpArrow,
                VK_DOWN => Key::DownArrow,
                VK_SHIFT => Key::Shift,
                VK_CONTROL => Key::Control,
                VK_LMENU => Key::Alt,
                VK_HOME => Key::Home,

                VK_DELETE => Key::Raw(83), // Linux only
                VK_PRIOR => Key::PageUp,
                VK_NEXT => Key::PageDown,
                //VK_OEM_1 => Key::Layout('['),
                VK_OEM_2 => Key::Layout('/'),
                VK_OEM_3 => Key::Layout('`'),
                VK_OEM_4 => Key::Layout('['),
                VK_OEM_5 => Key::Layout('\\'),
                VK_OEM_6 => Key::Layout(']'),
                VK_OEM_7 => Key::Layout('\''),
                VK_OEM_PLUS => Key::Layout('+'),
                VK_OEM_MINUS => Key::Layout('-'),
                VK_OEM_COMMA => Key::Layout(','),
                VK_OEM_PERIOD => Key::Layout('.'),
                _ => {
                    println!("Key code {:X} not supported", data[0]);
                    continue;
                }
            };

//        println!("Data: {:?}", key_to_press);
            if data[1] == 0 {
//                enigo.key_click()
                println!("Up: {:?}", key_to_press);
//            enigo.key_up(key_to_press)
            } else {
                println!("Down: {:?}", key_to_press);
//            enigo.key_down(key_to_press)
            }
        }
    }
}
