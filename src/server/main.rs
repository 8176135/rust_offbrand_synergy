#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate winapi;
extern crate web_view;
extern crate shared_consts;
extern crate bincode;
#[macro_use]
extern crate simple_error;

#[macro_use]
extern crate lazy_static;

extern crate serde_json;

use simple_error::SimpleError;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use winapi::um::winuser;
use winapi::shared::windef;

use shared_consts::*;
//use std::time::{Duration, SystemTime};

static mut KEYBOARD_HOOK_ID: *mut winapi::shared::windef::HHOOK__ = std::ptr::null_mut();
static mut MOUSE_HOOK_ID: *mut winapi::shared::windef::HHOOK__ = std::ptr::null_mut();
static CURRENTLY_TRANSFERRING: AtomicBool = AtomicBool::new(false);
//static CACHED_HEIGHT: AtomicUsize = AtomicUsize::new(100);
//static MOUSE_POLL_LAST_TIME: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    static ref STREAM: UdpSocket = UdpSocket::bind("0.0.0.0:13111").expect("Failed to bind");
    static ref PREVIOUS_KEY_STATE: Mutex<[bool; 255]> = Mutex::new([false; 255]);
    static ref EDGE_LIST: Mutex<Vec<ConnectionInfo>> = Mutex::new(Vec::new());
//    static ref MONITOR_LENGTH: i32 = unsafe { winuser::GetSystemMetrics(winuser::SM_CXVIRTUALSCREEN) };
//    static ref SCREEN_RESOLUTION =
//    static ref CLIENT_INFO:
}

const HTML: &'static str = include_str!("html_frontend/connection_screen.html.embedded");

fn main() {
    let output = build_monitor_layout();
//    println!("{:?}", output);

    let all_main_threads = Arc::new(Mutex::new(Vec::new()));
    web_view::run("Server", web_view::Content::Html(HTML), Some((600, 450)), false, false, move |_webview| {},
                  {
                      let all_main_threads = all_main_threads.clone();
                      move |webview, arg, _userdata| {
                          let arg: Vec<&str> = arg.split(' ').collect();

                          match arg[0] {
                              "connect" => {
                                  let data = parse_return_json(arg[1]);
                                  println!("{:?}", data);
                                  *EDGE_LIST.lock().unwrap() = data;
                                  let mut main_threads_lock = all_main_threads.lock().unwrap();
                                  main_threads_lock.push(std::thread::spawn(windows_message_pump));
                                  main_threads_lock.push(std::thread::spawn(client_response_manager));
                                  webview.terminate();

                                  unsafe {
                                      let win_handle = winuser::GetActiveWindow();
                                      winuser::DestroyWindow(win_handle);
                                  }
                              }
                              "loaded" => {
                                  let json_parsed = serde_json::to_string(&output).unwrap();
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

fn connect_to_ip(urn: &str) -> Result<(), SimpleError> {
    if let Err(e) = STREAM.connect(urn) {
//        webview.eval(&format!("showConnectedMsg('Error: {}')", e.to_string()));
        println!("{:?}", e);
        Err(SimpleError::from(e))
    } else {
//        webview.eval(&format!("showConnectedMsg('Connected at port {}')", port_num));
        println!("Connected to {}", urn);
//        let _client_info = init_connection().unwrap();

        Ok(())
    }
}

fn parse_return_json(input: &str) -> Vec<ConnectionInfo> {
    let map: HashMap<String, ConnectionInfo> = serde_json::from_str(input).unwrap();
    map.values().cloned().collect()
}

fn build_monitor_layout() -> Box<ScreenCollection> {
    let screen_collection = Box::new(ScreenCollection(Vec::with_capacity(20)));
    let rect = windef::RECT { top: -10000, left: -10000, right: 10000, bottom: 10000 };

    unsafe {
        let (virtual_x, virtual_y) = (winuser::GetSystemMetrics(winuser::SM_CXVIRTUALSCREEN), winuser::GetSystemMetrics(winuser::SM_CYVIRTUALSCREEN));
        winuser::EnumDisplayMonitors(std::ptr::null_mut(), &rect as *const windef::RECT, Some(monitor_enum_callback), &screen_collection as *const _ as isize);
    }
//    println!("{:?}", testing);
    screen_collection
}

extern "system" fn monitor_enum_callback(monitor: windef::HMONITOR, hdc: windef::HDC, rect: windef::LPRECT, l_param: isize) -> i32 {
    let l_param = unsafe { (l_param as *mut Box<ScreenCollection>).as_mut().unwrap() };
    let rect = unsafe { rect.as_ref().unwrap() };
    let screen_cords = ScreenRect::from(*rect);
//    println!("{:?}", screen_cords);
    (*l_param).0.push(screen_cords);
    1
}

fn client_response_manager() {
    let mut data = [0u8; 3];
    loop {
        STREAM.recv(&mut data).unwrap();
        if data[0] == RESP_CROSSBACK {
            set_transfer(false);
        }
    }
}

fn init_connection() -> Result<ClientInfo, SimpleError> {
    STREAM.send(&[0xFF, 0xFF, 0xFF]).unwrap();
    let mut data = [0u8; 16];
    let recv_size = STREAM.recv(&mut data).map_err(|e| SimpleError::with("Can't receive stuff", e))?;

    let (ack, client_info) = bincode::deserialize::<(u8, shared_consts::ClientInfo)>(&data[..recv_size]).map_err(|e| SimpleError::with("Message incorrect, Wrong ip/port?", e))?;
    if ack != RESP_INIT {
        return Err(SimpleError::new("Ack message incorrect, Wrong ip/port?"));
    }
    Ok(client_info)
}

fn windows_message_pump() {
    unsafe {
        KEYBOARD_HOOK_ID = winuser::SetWindowsHookExA(winuser::WH_KEYBOARD_LL, Some(keyboard_hook_callback), std::ptr::null_mut(), 0);
        MOUSE_HOOK_ID = winuser::SetWindowsHookExA(winuser::WH_MOUSE_LL, Some(mouse_hook_callback), std::ptr::null_mut(), 0);
    }
    let mut msg: winuser::MSG = winuser::MSG {
        hwnd: winuser::HWND_TOP,
        message: 0,
        wParam: 0,
        lParam: 0,
        time: 0,
        pt: winapi::shared::windef::POINT { x: 0, y: 0 },
    };
    //STREAM.set_read_timeout(Some(std::time::Duration::new(0, 1000000)));
    loop {
        unsafe {
            let _pm = winuser::GetMessageW(&mut msg, winuser::HWND_TOP, 0, 0);
            winuser::TranslateMessage(&msg);
            winuser::DispatchMessageA(&msg);
        }
    }
}

extern "system" fn keyboard_hook_callback(code: i32, w_param: usize, l_param: isize) -> isize {
    if code >= 0 {
        let w_param = w_param as u32;

        let mut pks_guard: MutexGuard<[bool; 255]> = PREVIOUS_KEY_STATE.lock().unwrap();

        let alt_down: bool = pks_guard[winuser::VK_LMENU as usize] || pks_guard[winuser::VK_RMENU as usize] || pks_guard[winuser::VK_MENU as usize];//unsafe { winuser::GetAsyncKeyState(winuser::VK_MENU) & (1 << 15) != 0 };
        let control_down: bool = pks_guard[winuser::VK_LCONTROL as usize] || pks_guard[winuser::VK_RCONTROL as usize] || pks_guard[winuser::VK_CONTROL as usize];
        let shift_down: bool = pks_guard[winuser::VK_LSHIFT as usize] || pks_guard[winuser::VK_RSHIFT as usize] || pks_guard[winuser::VK_SHIFT as usize];//unsafe { winuser::GetAsyncKeyState(0x10) & (1 << 15) != 0 };

        let mut key_changed = false;

        let l_param = unsafe { *(l_param as *mut winuser::KBDLLHOOKSTRUCT) };
//        println!("ctrl: {} shift: {} alt: {}, {}", control_down, shift_down, alt_down,w_param);
        match w_param {
            winuser::WM_KEYUP | winuser::WM_SYSKEYUP => {
                if l_param.vkCode == 49 && alt_down && control_down && shift_down {
                    set_transfer(!CURRENTLY_TRANSFERRING.load(Ordering::Relaxed));
                    return 1;
                }

                // TODO: Performance improvements to get here
                if pks_guard[l_param.vkCode as usize] {
                    key_changed = true;
                    pks_guard[l_param.vkCode as usize] = false;
                }
            }
            winuser::WM_KEYDOWN | winuser::WM_SYSKEYDOWN => {
                if l_param.vkCode == 49 && alt_down && control_down && shift_down {
                    return 1;
                }
                // TODO: Performance improvements to get here
                if !pks_guard[l_param.vkCode as usize] {
                    key_changed = true;
                    pks_guard[l_param.vkCode as usize] = true;
                }
            }
            _ => { println!("Keyboard action not implemented: {}", w_param) }
        }

//        if key_changed {
//            println!("{:?}, {}, {}, {}", l_param, control_down, shift_down, alt_down);
//        }

        if CURRENTLY_TRANSFERRING.load(Ordering::Relaxed) && key_changed {
            match w_param {
                winuser::WM_KEYUP | winuser::WM_SYSKEYUP => {
                    STREAM.send(&[l_param.vkCode as u8, 1]).expect("Failed to write");
                }
                winuser::WM_KEYDOWN | winuser::WM_SYSKEYDOWN => {
                    STREAM.send(&[l_param.vkCode as u8, 0]).expect("Failed to write");
                }
                _ => { println!("Keyboard action not implemented: {}", w_param) }
            }
            return 1;
//            if (l_param.flags & 0b100000) != 0 && l_param.vkCode == 9 {
//                return 1;
//            }
        }
    }
    unsafe {
        winuser::CallNextHookEx(KEYBOARD_HOOK_ID, code, w_param, l_param)
    }
}

extern "system" fn mouse_hook_callback(code: i32, w_param: usize, l_param: isize) -> isize {
//    println!("Code: {} ,  wP: {}, lP: {}", code, w_param, l_param);
    if code >= 0 && CURRENTLY_TRANSFERRING.load(Ordering::Relaxed) {
        let l_param = unsafe { *(l_param as *mut winuser::MSLLHOOKSTRUCT) };
        match w_param as u32 {
            winuser::WM_MOUSEMOVE => {
                //println!("{}, {}", (l_param.pt.x.min(127).max(-127) as i8), (l_param.pt.y.min(127).max(-127) as i8));
                STREAM.send(&[CmdCode::MouseMove as u8, (l_param.pt.x.min(127).max(-127) as i8) as u8, (l_param.pt.y.min(127).max(-127) as i8) as u8]).expect("Failed to write");
            }
            winuser::WM_LBUTTONDOWN => {
                STREAM.send(&[winuser::VK_LBUTTON as u8, 0]).expect("Failed to write");
            }
            winuser::WM_LBUTTONUP => {
                STREAM.send(&[winuser::VK_LBUTTON as u8, 1]).expect("Failed to write");
            }
            winuser::WM_RBUTTONDOWN => {
                STREAM.send(&[winuser::VK_RBUTTON as u8, 0]).expect("Failed to write");
            }
            winuser::WM_RBUTTONUP => {
                STREAM.send(&[winuser::VK_RBUTTON as u8, 1]).expect("Failed to write");
            }
            winuser::WM_MOUSEWHEEL => {
                STREAM.send(&[0x07, ((l_param.mouseData >> 16) as i16 / 120) as i8 as u8]).expect("Failed to write");
            }
            winuser::WM_MBUTTONDOWN => {
                STREAM.send(&[winuser::VK_MBUTTON as u8, 0]).expect("Failed to write");
            }
            winuser::WM_MBUTTONUP => {
                STREAM.send(&[winuser::VK_MBUTTON as u8, 1]).expect("Failed to write");
            }
            _ => { println!("Mouse action not implemented yet") }
        }
        return 1;
    } else if code >= 0 {
        let l_param = unsafe { *(l_param as *mut winuser::MSLLHOOKSTRUCT) };
        if w_param as u32 == winuser::WM_MOUSEMOVE {
            let edge_list_guard: MutexGuard<Vec<ConnectionInfo>> = EDGE_LIST.lock().unwrap();
            for edge in edge_list_guard.iter() {
                if !edge.side.is_point_outside(l_param.pt.into()) {
                    continue;
                }
                println!("SEND IT!: x: {}, y: {}", l_param.pt.x, l_param.pt.y);
                let pos_frac = (edge.side.get_pos_percent(l_param.pt.into()) * 255.0).round() as u8;
                connect_to_ip(&edge.ip_and_port).expect("Can't connect");
                STREAM.send(&[CmdCode::MousePos as u8, pos_frac, edge.side.outward_direction() as u8]).expect("Failed to write");
                set_transfer(true);
            }
        }
    }
    unsafe {
        winuser::CallNextHookEx(MOUSE_HOOK_ID, code, w_param, l_param)
    }
}

fn monitor_info_from_point(x: i32, y: i32) -> ScreenRect {
    let mut monitor_info = winuser::MONITORINFO {
        cbSize: std::mem::size_of::<winuser::MONITORINFO>() as u32,
        rcMonitor: winapi::shared::windef::RECT { left: 0, right: 0, top: 0, bottom: 0 },
        rcWork: winapi::shared::windef::RECT { left: 0, right: 0, top: 0, bottom: 0 },
        dwFlags: 0,
    };
    unsafe {
        winuser::GetMonitorInfoA(winuser::MonitorFromPoint(windef::POINT { x, y }, winuser::MONITOR_DEFAULTTONEAREST), &mut monitor_info as *mut winuser::MONITORINFO);
    }
    ScreenRect::from(monitor_info.rcMonitor)
}

fn set_transfer(set_state: bool) {
    if set_state {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            unsafe { println!("{}", winuser::SetCursorPos(0, 0)) }
            CURRENTLY_TRANSFERRING.store(set_state, Ordering::Relaxed); //TODO: might be problematic
            println!("Current state: {}", set_state);
        });
    } else {
        CURRENTLY_TRANSFERRING.store(set_state, Ordering::Relaxed);
        println!("Current state: {}", set_state);
    }
}

//fn get_monitor_info_at_point() {
//
//}

//fn lerp(x: f32, a: i32, b: i32) -> f32 {
//    a as f32 + x * (b - a) as f32
//}