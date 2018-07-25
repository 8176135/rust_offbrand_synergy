extern crate enigo;
extern crate web_view;
extern crate xdo_io;
extern crate regex;
extern crate bincode;
extern crate serde_json;
extern crate shared_consts;
extern crate num_traits;

use shared_consts::win_key_codes::*;
use shared_consts::*;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};

use num_traits::FromPrimitive;
//use std::io::Read;

use enigo::{Enigo, KeyboardControllable, MouseControllable, Key, MouseButton};

const HTML: &'static str = include_str!("html_frontend/listener_screen.html.embedded");


fn main() {
    let display_output = build_monitor_layout();
    let all_main_threads = Arc::new(Mutex::new(Vec::new()));
    web_view::run("Listener", web_view::Content::Html(HTML), Some((600, 450)), false, true, move |_webview| {},
                  {
                      let all_main_threads = all_main_threads.clone();
                      move |webview, arg, _userdata| {
                          let arg: Vec<&str> = arg.split(' ').collect();
                          let port_num = "13111";

                          match arg[0] {
                              "listen" => {
//                                  println!("{:?}", serde_json::from_str::<ConnectionInfo>(arg[1]));
                                  let listener = UdpSocket::bind(format!("0.0.0.0:{}", port_num));
                                  if let Err(e) = listener {
                                      webview.eval(&format!("showConnectedMsg('Error: {}')", e.to_string()));
                                      println!("{:?}", e);
                                  } else if let Ok(listener) = listener {
                                      webview.eval(&format!("showConnectedMsg('Connected at port {}')", port_num));
                                      let display_output = display_output.clone();
                                      all_main_threads.lock().unwrap().push(std::thread::spawn(move || {
                                          listener_loop(listener, &display_output);
                                      }));
                                      webview.terminate();
                                  }
                              }
                              "loaded" => {
                                  let json_parsed = serde_json::to_string(&display_output).unwrap();
                                  webview.eval(&format!("showMonitorList({})", json_parsed));
                              }
                              "debug" => {
                                  println!("{}", arg[1]);
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

fn build_monitor_layout() -> Box<ScreenCollection> {
    let output = String::from_utf8(
        std::process::Command::new("xrandr").arg("--listmonitors").output().expect("failed to run xrandr").stdout).unwrap();

    Box::new(ScreenCollection(regex::Regex::new(r#"(\w+) (\d+?)/\d+?x(\d+?)/\d+?\+(\d+)\+(\d+)"#).unwrap()
        .captures_iter(&output).map(|c| {
        let data: Vec<i32> = c.iter().skip(2).map(|c| c.unwrap().as_str().parse::<i32>().unwrap()).collect();
        ScreenRect::new_from_dimensions(data[2], data[3], data[0], data[1])
    }).collect::<Vec<ScreenRect>>()))
}

fn listener_loop(listener: UdpSocket, monitors: &ScreenCollection) {
    let mut enigo: Enigo = Enigo::new();
    let xdo = xdo_io::Xdo::new();
    let mut data = [0u8; 3];

    let mut input_direction = Direction::Right;

    println!("Listening started");
    loop {
        let addr = listener.recv_from(&mut data).unwrap().1;

        match data[0] {
            0...6 => {
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
            }
            x if x == CmdCode::MouseScroll as u8 => {
                if (data[1] as i8) > 0 {
                    enigo.mouse_click(MouseButton::ScrollUp);
                } else {
                    enigo.mouse_click(MouseButton::ScrollDown);
                }
            }
            x if x == CmdCode::MouseMove as u8 => {
                let (mouse_pos, screen_num) = xdo.get_mouse_location();
                let side = monitors.0.iter().map(|c| c.sides.get(&input_direction).unwrap()).next().unwrap();
//                println!("{:?}", Vector2 { x:mouse_pos.x + data[1] as i8 as i32, y: mouse_pos.y});
//                println!("{:?}", side);
                if side.is_point_outside(Vector2 { x:mouse_pos.x + data[1] as i8 as i32, y: mouse_pos.y}) {
                    println!("rergegr");
                    listener.connect(addr).unwrap();

                    listener.send(&[RESP_CROSSBACK, (side.get_pos_percent(Vector2 { x: mouse_pos.x, y: mouse_pos.y }).max(0.0).min(1.0) * 255 as f32).round() as u8]).unwrap();
                }
//                println!("{}, {}", data[1] as i8 as i32, data[2] as i8 as i32);
                enigo.mouse_move_relative(data[1] as i8 as i32, data[2] as i8 as i32);
            }
            x if x == CmdCode::MousePos as u8 => {
                let scaled = data[1] as f32 / 255.0;
                let dir_outwards = Direction::from_u8(data[2] ^ 0b11).unwrap();
                input_direction = dir_outwards;
                let side = monitors.0.iter().map(|c| c.sides.get(&dir_outwards).unwrap()).next().unwrap();
                let pos = side.get_percent_pos(scaled);
                enigo.mouse_move_to(pos.x, pos.y);
            }
//            x if x == CmdCode::Init as u8 => {
//                println!("Init");
//                listener.connect(addr).unwrap();
//                listener.send(&bincode::serialize(&(RESP_INIT, ClientInfo { scrn_size })).expect("Binary seralize failed")).expect("Unable to send back info");
//            }
            _ => {
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
//        if data[0] <= 6 { // Mouse event
//        } else if data[0] == 7 {
//
//        } else if data[0] == 0xFF {
////            let delta_x = (((data[1] & 0xF) as i8) << 4) >> 4;
////            let delta_y = ((data[1] as i8) >> 4);
//            //println!("{}, {}", data[1] as i32,  data[2] as i32);
//        } else { // or keyboard event
//
//        }
    }
}
