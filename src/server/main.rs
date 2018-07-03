#![allow(unused_imports)]

extern crate winapi;
extern crate web_view;

#[macro_use]
extern crate lazy_static;

use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use winapi::um::winuser;
//use std::time::{Duration, SystemTime};


static mut KEYBOARD_HOOK_ID: *mut winapi::shared::windef::HHOOK__ = std::ptr::null_mut();
static mut MOUSE_HOOK_ID: *mut winapi::shared::windef::HHOOK__ = std::ptr::null_mut();
static CURRENTLY_TRANSFERRING: AtomicBool = AtomicBool::new(false);
//static MOUSE_POLL_LAST_TIME: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    static ref STREAM: UdpSocket = UdpSocket::bind("0.0.0.0:13112").expect("Can't connect to port: ");
    static ref PREVIOUS_KEY_STATE: Mutex<[bool; 255]> = Mutex::new([false; 255]);
}

const HTML: &'static str = include_str!("html_frontend/connection_screen.html.embedded") ;

fn main() {
    let all_main_threads = Arc::new(Mutex::new(Vec::new()));
    web_view::run("Server", web_view::Content::Html(HTML), Some((250, 150)), false, false, move |_webview| {},
                             {
                                 let all_main_threads = all_main_threads.clone();
                                 move |webview, arg, _userdata| {
                                     let arg: Vec<&str> = arg.split(' ').collect();
                                     let ip_addr = arg[1];
                                     let port_num = arg[2];

                                     match arg[0] {
                                         "connect" => {
                                             if let Err(e) = STREAM.connect(format!("{}:{}", ip_addr, port_num)) {
                                                 webview.eval(&format!("showConnectedMsg('Error: {}')", e.to_string()));
                                                 println!("{:?}", e);
                                             } else {
                                                 webview.eval(&format!("showConnectedMsg('Connected at port {}')", port_num));
                                                 all_main_threads.lock().unwrap().push(std::thread::spawn(windows_message_pump));
                                                 unsafe {
                                                     let win_handle = winuser::GetActiveWindow();
                                                     winuser::DestroyWindow(win_handle);
                                                 }
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

        let alt_down: bool = unsafe { winuser::GetAsyncKeyState(winuser::VK_MENU) & (1 << 15) != 0 };
        let control_down: bool = pks_guard[winuser::VK_LCONTROL as usize] || pks_guard[winuser::VK_RCONTROL as usize] || pks_guard[winuser::VK_CONTROL as usize];
        let shift_down: bool = unsafe { winuser::GetAsyncKeyState(0x10) & (1 << 15) != 0 };

        let mut key_changed = false;

        let l_param = unsafe { *(l_param as *mut winuser::KBDLLHOOKSTRUCT) };

        match w_param {
            winuser::WM_KEYUP | winuser::WM_SYSKEYUP => {
                if l_param.vkCode == 49 && alt_down && control_down && shift_down {
                    CURRENTLY_TRANSFERRING.store(!CURRENTLY_TRANSFERRING.load(Ordering::Relaxed), Ordering::Relaxed);
                    // TODO: Change this, save the cursor position and subtract from it instead
                    unsafe {
                        winuser::SetCursorPos(0, 0);
                    }
                    println!("Current state: {}", CURRENTLY_TRANSFERRING.load(Ordering::Relaxed));
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
                STREAM.send(&[0xFF, (l_param.pt.x.min(127).max(-127) as i8) as u8, (l_param.pt.y.min(127).max(-127) as i8) as u8]).expect("Failed to write");
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
                //println!("{:?}", (l_param.mouseData >> 16) as i16);
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
    }
    unsafe {
        winuser::CallNextHookEx(MOUSE_HOOK_ID, code, w_param, l_param)
    }
}