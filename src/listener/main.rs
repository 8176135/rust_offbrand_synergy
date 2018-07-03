extern crate enigo;
extern crate web_view;
extern crate libxdo_sys;

mod win_key_codes;

use win_key_codes::*;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
//use std::io::Read;

use enigo::{Enigo, KeyboardControllable, MouseControllable, Key, MouseButton};

const HTML: &'static str = include_str!("html_frontend/listener_screen.html.embedded");

fn main() {
    let all_main_threads = Arc::new(Mutex::new(Vec::new()));
    web_view::run("Listener", web_view::Content::Html(HTML), Some((200, 150)), false, true, move |_webview| {},
                  {
                      let all_main_threads = all_main_threads.clone();
                      move |webview, arg, _userdata| {
                          let arg: Vec<&str> = arg.split(' ').collect();
                          let port_num = arg[1];

                          match arg[0] {
                              "listen" => {
                                  let listener = UdpSocket::bind(format!("0.0.0.0:{}",port_num));
                                  if let Err(e) = listener {
                                      webview.eval(&format!("showConnectedMsg('Error: {}')", e.to_string()));
                                      println!("{:?}", e);
                                  } else if let Ok(listener) = listener {
                                      webview.eval(&format!("showConnectedMsg('Connected at port {}')", port_num));
                                      all_main_threads.lock().unwrap().push(std::thread::spawn(|| {
                                          listener_loop(listener);
                                      }));
                                      webview.terminate();
                                  }
                              }
                              _ => unimplemented!()
                          }
                      }
                  }, ());

    {
        let mut unlocked_thread_list = all_main_threads.lock().unwrap();

        while let Some(join_handle) = unlocked_thread_list.pop() {
            join_handle.join().unwrap();
        }
    }
}

fn listener_loop(listener: UdpSocket) {
    let mut enigo: Enigo = Enigo::new();
    let mut xdo = unsafe {
         libxdo_sys::xdo_new(std::ptr::null())
    };
    let mut data = [0u8; 3];
    loop {
        listener.recv(&mut data).unwrap();

        if data[0] <= 6 { // Mouse event
            let mouse_btn_to_press = match data[0] {
                VK_LBUTTON => MouseButton::Left,
                VK_RBUTTON => MouseButton::Right,
                VK_MBUTTON => MouseButton::Middle,
                0x07 if (data[1] as i8) > 0 => MouseButton::ScrollUp,
                0x07 if (data[1] as i8) < 0 => MouseButton::ScrollDown,
                _ => continue
            };
            if data[1] == 0 {
                println!("Down: {:?}", data[0]);
                enigo.mouse_down(mouse_btn_to_press);
            } else {
                println!("Up: {:?}", mouse_btn_to_press);
                enigo.mouse_up(mouse_btn_to_press);
            }
        } else if data[0] == 7 {
            if (data[1] as i8) > 0 {
                enigo.mouse_click(MouseButton::ScrollUp);
            } else {
                enigo.mouse_click(MouseButton::ScrollDown);
            }
        } else if data[0] == 0xFF {
//            let delta_x = (((data[1] & 0xF) as i8) << 4) >> 4;
//            let delta_y = ((data[1] as i8) >> 4);
            //println!("{}, {}", data[1] as i32,  data[2] as i32);
            let mut x = 0;
            let mut y = 0;
            let mut scrn_num = 0;
            unsafe {
                let temp = libxdo_sys::xdo_get_mouse_location(xdo, &mut x as *mut i32, &mut y as *mut i32, &mut scrn_num as *mut i32);
                println!("{} -- {}, {} -- {}", temp, x, y, scrn_num);
            }

            enigo.mouse_move_relative(data[1] as i8 as i32, data[2] as i8 as i32);
        } else { // or keyboard event
            let key_to_press = match data[0] {
                0x30...0x39 => Key::Layout(data[0] as char),
                0x41...0x5A => Key::Layout((data[0] + 0x20) as char),
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
                VK_LSHIFT | VK_SHIFT | VK_RSHIFT => Key::Shift,
                VK_LCONTROL | VK_CONTROL | VK_RCONTROL => Key::Control,
                VK_LMENU | VK_MENU | VK_RMENU => Key::Alt,
                VK_HOME => Key::Home,

                VK_DELETE => Key::Delete, // Linux only
                VK_PRIOR => Key::PageUp,
                VK_NEXT => Key::PageDown,
                //VK_OEM_1 => Key::Layout('['),
                VK_OEM_1 => Key::Layout(';'),
                VK_OEM_2 => Key::Layout('/'),
                VK_OEM_3 => Key::Layout('`'),
                VK_OEM_4 => Key::Layout('['),
                VK_OEM_5 => Key::Layout('\\'),
                VK_OEM_6 => Key::Layout(']'),
                VK_OEM_7 => Key::Layout('\''),
                VK_OEM_PLUS => Key::Layout('='),
                VK_OEM_MINUS => Key::Layout('-'),
                VK_OEM_COMMA => Key::Layout(','),
                VK_OEM_PERIOD => Key::Layout('.'),
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
