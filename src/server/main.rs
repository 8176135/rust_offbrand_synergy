extern crate user32;
extern crate winapi;
//extern crate winit;
#[macro_use]
extern crate lazy_static;

use std::net::UdpSocket;
//use winit::os::windows::WindowExt;
//use std::io::Write;
//use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, SystemTime};

//use winit::{Event, DeviceEvent, WindowEvent, VirtualKeyCode, ElementState};

static mut KEYBOARD_HOOK_ID: *mut winapi::HHOOK__ = std::ptr::null_mut();
static mut MOUSE_HOOK_ID: *mut winapi::HHOOK__ = std::ptr::null_mut();
static CURRENTLY_TRANSFERRING: AtomicBool = AtomicBool::new(false);
static MOUSE_POLL_LAST_TIME: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    static ref STREAM: UdpSocket = UdpSocket::bind("0.0.0.0:13112").expect("Can't connect to port: ");
    static ref PREVIOUS_KEY_STATE: Mutex<[bool; 255]> = Mutex::new([false; 255]);
}

//static STREAM: Arc<Mutex<Option<UdpSocket>>> = Arc::new(Mutex::default());
//static PREVIOUS_KEY_STATE: Arc<Mutex<[bool; 255]>> = Arc::new(Mutex::new([false; 255]));

fn main() {
    unsafe {
        KEYBOARD_HOOK_ID = user32::SetWindowsHookExA(winapi::WH_KEYBOARD_LL, Some(keyboard_hook_callback), std::ptr::null_mut(), 0);
        MOUSE_HOOK_ID = user32::SetWindowsHookExA(winapi::WH_MOUSE_LL, Some(mouse_hook_callback), std::ptr::null_mut(), 0);
    }
    let mut msg: winapi::winuser::MSG = winapi::winuser::MSG {
        hwnd: 0 as winapi::HWND,
        message: 0 as winapi::UINT,
        wParam: 0 as winapi::WPARAM,
        lParam: 0 as winapi::LPARAM,
        time: 0 as winapi::DWORD,
        pt: winapi::POINT { x: 0, y: 0 },
    };
    STREAM.connect("192.168.1.54:13111").expect("Failed to connect");
//    let timer_thread_handle = {
//        let stream = UdpSocket::bind("127.0.0.1:13112").unwrap();
//        stream.connect("192.168.1.54:13111").expect("Failed to connect");
//        td::thread::spawn(move || {
//            loop {
//
//
//
//                std::thread::sleep(Duration::new(0, 20 * 1000000));
//            }
//        })
//    };


//    {
//        let mut stream = STREAM.lock().unwrap();
//        *stream = Some(UdpSocket::bind("0.0.0.0:13112").expect("Can't connect to port: "));
//        stream.as_ref().connect("192.168.1.54:13111").expect("Failed to connect");
//    }

    loop {
        unsafe {
            let pm = user32::GetMessageW(&mut msg, 0 as winapi::HWND, 0, 0);
            if pm == 0 {
                break;
            }
            println!("{:?}", msg);
            user32::TranslateMessage(&msg);
            user32::DispatchMessageA(&msg);
        }
    }
//    let mut events_loop = winit::EventsLoop::new();
//    let window = winit::WindowBuilder::new()
//        .build(&events_loop).unwrap();
//
//    window.set_title("A fantastic window!");
//    window.set_cursor_state(winit::CursorState::Hide).unwrap();

//    let mut previous_key_state: Vec<bool> = Vec::new();
//    previous_key_state.resize(255, false);

//    stream.connect("127.0.0.1:13111").expect("Failed to connect");

//    let is_in_another_window = std::sync::Arc::new(std::sync::Mutex::new(42));
//    let clone_thing = is_in_another_window.clone();
//    std::thread::spawn(move || other_thread(clone_thing));
    //events_loop.run_forever(|event| {
    //println!("{:?}", event);
//
//        {
//            let mut data = is_in_another_window.lock().unwrap();
//            *data += 1;
//        }

    //window.set_cursor_position(100, 100).is_ok(); // Don't care
//        for i in 1..255u8 {
//            let mut is_current_key_down: bool;
//            unsafe {
//                is_current_key_down = user32::GetAsyncKeyState(i as i32) & (1 << 15) != 0;
//            }
//            if previous_key_state[i as usize] != is_current_key_down {
//                previous_key_state[i as usize] = is_current_key_down;
//                stream.send(&[i, is_current_key_down as u8]).expect("Failed to write");
//            }
//        }
//        match event {
//            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => winit::ControlFlow::Break,
//            Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
//                if input.virtual_keycode.unwrap_or(VirtualKeyCode::A) == VirtualKeyCode::Key1 && input.modifiers.ctrl && input.modifiers.alt && input.modifiers.shift {
//                    println!("Disabled");
//                    CURRENTLY_TRANSFERING.store(false, Ordering::Relaxed);
//                } else {
//                    let key_code = if let Some(keycode) = input.virtual_keycode {
//                        keycode as u8 + 8
//                    } else {
//                        200
//                    };
//
//                    if previous_key_state[key_code as usize] != (input.state == ElementState::Pressed) {
//                        previous_key_state[key_code as usize] = input.state == ElementState::Pressed;
//                        stream.send(&[key_code, input.state as u8]).expect("Failed to write");
//
//                        println!("{:X}", key_code);
//                    }
//                }
//                winit::ControlFlow::Continue
//            }
//            Event::WindowEvent { event: WindowEvent::MouseInput { state, button, .. }, .. } => {
//                println!("{:?}, {}", state, btn_to_num(&button));
//                stream.send(&[btn_to_num(&button), state as u8]).expect("Failed to write");
//                winit::ControlFlow::Continue
//            }
//            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
////                println!("{:?}", delta);
////                println!("{:b}", ( delta.1.round() as i8) << 4);
//                //stream.send(&[0xFF, ((( delta.1.min(7.0).max(-7.0).round() as i8) << 4) | ((delta.0.min(7.0).max(-7.0).round() as i8) & 0xF)) as u8 ]).expect("Failed to write");
//                stream.send(&[0xFF, (delta.0.min(127.0).max(-127.0).round() as i8) as u8, (delta.1.min(127.0).max(-127.0).round() as i8) as u8]).expect("Failed to write");
//
//                winit::ControlFlow::Continue
//            }
//            _ => winit::ControlFlow::Continue,
//        }
//    });
}

extern "system" fn keyboard_hook_callback(code: i32, w_param: u64, l_param: i64) -> i64 {
    if code >= 0 {
        let w_param = w_param as u32;

        let mut pks_guard: MutexGuard<[bool; 255]> = PREVIOUS_KEY_STATE.lock().unwrap();

        let alt_down: bool = unsafe { user32::GetAsyncKeyState(winapi::VK_MENU) & (1 << 15) != 0 };
        let control_down: bool = pks_guard[winapi::VK_LCONTROL as usize] || pks_guard[winapi::VK_RCONTROL as usize] || pks_guard[winapi::VK_CONTROL as usize];
        let shift_down: bool = unsafe { user32::GetAsyncKeyState(0x10) & (1 << 15) != 0 };

        let mut key_changed = false;

        let l_param = unsafe { *(l_param as *mut winapi::KBDLLHOOKSTRUCT) };

        match w_param {
            winapi::WM_KEYUP | winapi::WM_SYSKEYUP => {
                if l_param.vkCode == 49 && alt_down && control_down && shift_down {
                    CURRENTLY_TRANSFERRING.store(!CURRENTLY_TRANSFERRING.load(Ordering::Relaxed), Ordering::Relaxed);
                    // TODO: Change this, save the cursor position and subtract from it instead
                    unsafe {
                        user32::SetCursorPos(0, 0);
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
            winapi::WM_KEYDOWN | winapi::WM_SYSKEYDOWN => {
                if l_param.vkCode == 49 && alt_down && control_down && shift_down {
                    return 1;
                }
                //let mut pks_guard: MutexGuard<[bool; 255]> = PREVIOUS_KEY_STATE.lock().unwrap();
                // TODO: Performance improvements to get here
                if !pks_guard[l_param.vkCode as usize] {
                    key_changed = true;
                    pks_guard[l_param.vkCode as usize] = true;
                }
            }
            _ => { println!("Keyboard action not implemented: {}", w_param) }
        }

        if key_changed {
            println!("{:?}, {}, {}, {}", l_param, control_down, shift_down, alt_down);
        }

        if CURRENTLY_TRANSFERRING.load(Ordering::Relaxed) && key_changed {
            match w_param {
                winapi::WM_KEYUP | winapi::WM_SYSKEYUP => {
                    STREAM.send(&[l_param.vkCode as u8, 1]).expect("Failed to write");
                }
                winapi::WM_KEYDOWN | winapi::WM_SYSKEYDOWN => {
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
        user32::CallNextHookEx(KEYBOARD_HOOK_ID, code, w_param, l_param)
    }
}

extern "system" fn mouse_hook_callback(code: i32, w_param: u64, l_param: i64) -> i64 {
//    println!("Code: {} ,  wP: {}, lP: {}", code, w_param, l_param);
    if code >= 0 && CURRENTLY_TRANSFERRING.load(Ordering::Relaxed) {
        let l_param = unsafe { *(l_param as *mut winapi::MSLLHOOKSTRUCT) };
        match w_param as u32 {
            winapi::WM_MOUSEMOVE => {
                println!("{}, {}", (l_param.pt.x.min(127).max(-127) as i8), (l_param.pt.y.min(127).max(-127) as i8) );
                STREAM.send(&[0xFF, (l_param.pt.x.min(127).max(-127) as i8) as u8, (l_param.pt.y.min(127).max(-127) as i8) as u8]).expect("Failed to write");
//                let temp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos() as usize;
//                if temp > MOUSE_POLL_LAST_TIME.load(Ordering::Relaxed) {
//                    println!("{:?}", temp - MOUSE_POLL_LAST_TIME.load(Ordering::Relaxed));
//                }
//                MOUSE_POLL_LAST_TIME.store(temp, Ordering::Relaxed);
            }
            winapi::WM_LBUTTONDOWN => {
                STREAM.send(&[winapi::VK_LBUTTON as u8, 0]).expect("Failed to write");
            }
            winapi::WM_LBUTTONUP => {
                STREAM.send(&[winapi::VK_LBUTTON as u8, 1]).expect("Failed to write");
            }
            winapi::WM_RBUTTONDOWN => {
                STREAM.send(&[winapi::VK_RBUTTON as u8, 0]).expect("Failed to write");
            }
            winapi::WM_RBUTTONUP => {
                STREAM.send(&[winapi::VK_RBUTTON as u8, 1]).expect("Failed to write");
            }
            winapi::WM_MOUSEWHEEL => {
                //println!("{:?}", (l_param.mouseData >> 16) as i16);
                STREAM.send(&[0x07, ((l_param.mouseData >> 16) as i16 / 120) as i8 as u8]).expect("Failed to write");
            }
            winapi::WM_MBUTTONDOWN => {
                STREAM.send(&[winapi::VK_MBUTTON as u8, 0]).expect("Failed to write");
            }
            winapi::WM_MBUTTONUP => {
                STREAM.send(&[winapi::VK_MBUTTON as u8, 1]).expect("Failed to write");
            }
            _ => { println!("Mouse action not implemented yet") }
        }
        return 1;
    }
    unsafe {
        user32::CallNextHookEx(MOUSE_HOOK_ID, code, w_param, l_param)
    }
}


//fn btn_to_num(btn: &winit::MouseButton) -> u8 {
//    match *btn {
//        winit::MouseButton::Left => 0,
//        winit::MouseButton::Right => 1,
//        winit::MouseButton::Middle => 2,
//        winit::MouseButton::Other(i) => 2 + i,
//    }
//}

//fn other_thread(other_window: std::sync::Arc<std::sync::Mutex<i32>>) {
//    loop {
//        std::thread::sleep(std::time::Duration::from_millis(20));
//        let mut cursor_pos = winapi::windef::POINT { x: 0, y: 0 };
//        println!("Thread num: {}", other_window.lock().unwrap());
//        unsafe {
//            //let data = std::ptr::read(window.get_hwnd());
//
//            //std::ptr::write(temp, window.get_hwnd());
//            //user32::SetForegroundWindow(window.get_hwnd() as *mut _);
//            //user32::ShowWindow(window.get_hwnd() as *mut _, 6);
//        }
////        unsafe {
////            if user32::GetCursorPos(&mut cursor_pos as winapi::windef::LPPOINT) != 0 {
////                user32::SetCursorPos(500,500);
////                user32::ShowCursor(0);
////            }
////        }
////
////user32::SetWindowsHookExA()
////        for i in 1..255u8 {
////            let mut is_current_key_down: bool;
////            unsafe {
////                is_current_key_down = user32::GetAsyncKeyState(i as i32) & (1 << 15) != 0;
////            }
////            if previous_key_state[i as usize] != is_current_key_down {
////                previous_key_state[i as usize] = is_current_key_down;
////                stream.send(&[i, is_current_key_down as u8]).expect("Failed to write");
////            }
////        }
//    }
//}
