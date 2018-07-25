#![allow(unused_variables)]
#![allow(dead_code)]

#[macro_use]
extern crate serde_derive;
//extern crate bincode;
extern crate serde;

#[macro_use]
extern crate enum_primitive;
extern crate num_traits;

#[cfg(target_os = "windows")]
extern crate winapi;

#[cfg(target_os = "windows")]
use winapi::shared::windef;

extern crate simple_error;

use std::collections::HashMap;
use simple_error::SimpleError;


#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq)]
#[derive(Copy, Clone)]
pub struct Vector2 {
    pub x: i32,
    pub y: i32,
}

impl Vector2 {
    pub fn new(x: i32, y: i32) -> Vector2 {
        Vector2 { x, y }
    }
}

impl std::ops::Add for Vector2 {
    type Output = Vector2;

    fn add(self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::AddAssign for Vector2 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

#[cfg(target_os = "windows")]
impl From<windef::POINT> for Vector2 {
    fn from(rect: windef::POINT) -> Self {
        Self { x: rect.x, y: rect.y }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[derive(Copy, Clone)]
pub struct RectSide {
    outward_direction: Direction,
    length: i32,
    position: Vector2,
}

impl RectSide {
    pub fn new_from_coords(a: Vector2, b: Vector2) -> Result<RectSide, SimpleError> {
        if a.x != b.x && a.y != b.y {
            return Err(SimpleError::new("Not straight"));
        } else if a == b {
            return Err(SimpleError::new("Same point?"));
        }
        let (direction, length) = if a.x == b.x {
            let len = a.y - b.y;
            if len > 0 {
                (Direction::Left, len)
            } else {
                (Direction::Right, len * -1)
            }
        } else {
            let len = a.x - b.x;
            if len > 0 {
                (Direction::Down, len)
            } else {
                (Direction::Up, len * -1)
            }
        };
        Ok(RectSide {
            position: a,
            length: (a.x - b.x).abs().max((a.y - b.y).abs()),
            outward_direction: direction,
        })
    }

    pub fn offset(&mut self, amount: Vector2) {
        self.position += amount;
    }

    pub fn a(&self) -> Vector2 {
        self.position
    }

    pub fn b(&self) -> Vector2 {
        self.position + match self.outward_direction {
            Direction::Left => Vector2 { x: 0, y: -self.length },
            Direction::Right => Vector2 { x: 0, y: self.length },
            Direction::Up => Vector2 { x: self.length, y: 0 },
            Direction::Down => Vector2 { x: -self.length, y: 0 }
        }
    }

    pub fn is_point_outside(&self, point: Vector2) -> bool {
        match self.outward_direction {
            Direction::Left => point.x < self.position.x && point.y < self.position.y && point.y > self.position.y - self.length,
            Direction::Right => point.x > self.position.x && point.y > self.position.y && point.y < self.position.y + self.length,
            Direction::Up => point.y < self.position.y && point.x > self.position.x && point.x < self.position.x + self.length,
            Direction::Down => point.y > self.position.y && point.x < self.position.x && point.x > self.position.x - self.length,
        }
    }
    pub fn get_pos_percent(&self, point: Vector2) -> f32 {
        match self.outward_direction {
            Direction::Left => inverse_lerp(point.y, self.position.y, self.position.y - self.length),
            Direction::Right => inverse_lerp(point.y, self.position.y, self.position.y + self.length),
            Direction::Up => inverse_lerp(point.x, self.position.x, self.position.x + self.length),
            Direction::Down => inverse_lerp(point.x, self.position.x, self.position.x - self.length),
        }
    }

    pub fn get_percent_pos(&self, percent: f32) -> Vector2 {
        match self.outward_direction {
            Direction::Left => Vector2 { y: lerp(percent, self.position.y, self.position.y - self.length).round() as i32, x: self.a().x },
            Direction::Right => Vector2 { y: lerp(percent, self.position.y, self.position.y + self.length).round() as i32, x: self.a().x },
            Direction::Up => Vector2 { x: lerp(percent, self.position.x, self.position.x + self.length).round() as i32, y: self.a().y },
            Direction::Down => Vector2 { x: lerp(percent, self.position.x, self.position.x - self.length).round() as i32, y: self.a().y },
        }
    }

    pub fn outward_direction(&self) -> Direction {
        self.outward_direction
    }
    pub fn length(&self) -> i32 {
        self.length
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ScreenRect {
    pub sides: HashMap<Direction, RectSide>,
}

impl ScreenRect {
    pub fn new_primary_from_dimensions(width: i32, height: i32) -> ScreenRect {
        Self::new_from_dimensions(0, 0, width, height)
    }

    pub fn new_from_points(a_x: i32, a_y: i32, b_x: i32, b_y: i32) -> ScreenRect {
        Self::new_from_dimensions(a_x, a_y, b_x - a_x, b_y - a_y)
    }

    pub fn new_from_dimensions(x: i32, y: i32, width: i32, height: i32) -> ScreenRect {
        let mut temp = ScreenRect { sides: HashMap::new() };
        temp.sides.insert(Direction::Left, RectSide { length: height, outward_direction: Direction::Left, position: Vector2 { x, y: y + height } });
        temp.sides.insert(Direction::Right, RectSide { length: height, outward_direction: Direction::Right, position: Vector2 { x: x + width, y } });
        temp.sides.insert(Direction::Up, RectSide { length: width, outward_direction: Direction::Up, position: Vector2 { x, y } });
        temp.sides.insert(Direction::Down, RectSide { length: width, outward_direction: Direction::Down, position: Vector2 { x: x + width, y: y + height } });
        temp
    }

    fn offset(&mut self, vec: Vector2) {
        self.sides.iter_mut().for_each(|side| side.1.offset(vec));
    }

    fn dimensions(&self) -> Vector2 {
        Vector2 { x: self.sides.get(&Direction::Up).unwrap().length, y: self.sides.get(&Direction::Right).unwrap().length }
    }
}

impl PartialEq for ScreenRect {
    /// Not actually tested, probably should do that
    fn eq(&self, other: &Self) -> bool {
        for item in other.sides.iter() {
            if let Some(t) = self.sides.get(item.0) {
                return t == item.1;
            }
            return false;
        }
        true
    }
}

fn inverse_lerp(x: i32, a: i32, b: i32) -> f32 {
    (x - a) as f32 / ((b - a) as f32)
}

fn lerp(x: f32, a: i32, b: i32) -> f32 {
    a as f32 + x * (b - a) as f32
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(Copy, Clone)]
pub struct ClientInfo {
    pub scrn_size: Vector2
}

impl ScreenCollection {}

#[derive(Clone, Debug, Serialize)]
pub struct ScreenCollection(pub Vec<ScreenRect>);

#[cfg(target_os = "windows")]
impl From<windef::RECT> for ScreenRect {
    fn from(rect: windef::RECT) -> Self {
        Self::new_from_points(rect.left, rect.top, rect.right, rect.bottom)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ConnectionInfo {
    pub ip_and_port: String,
    pub side: RectSide,
}

pub enum CmdCode {
    Init = 0xFF,
    MouseMove = 0xF0,
    MouseScroll = 0x7,
    MousePos = 0xF1,
}

enum_from_primitive! {
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
    Left = 0,
    Up,
    Down,
    Right,
}
}
//pub mod direction {
//    pub const LEFT: u8 = 0;
//    pub const RIGHT: u8 = 1;
//    pub const TOP: u8 = 2;
//    pub const BOTTOM: u8 = 3;
//}

pub const RESP_INIT: u8 = 0x0F;
pub const RESP_CROSSBACK: u8 = 0xF1;

pub mod win_key_codes {
    pub const VK_LBUTTON: u8 = 0x01;
    pub const VK_RBUTTON: u8 = 0x02;
    pub const VK_CANCEL: u8 = 0x03;
    pub const VK_MBUTTON: u8 = 0x04;
    pub const VK_XBUTTON1: u8 = 0x05;
    pub const VK_XBUTTON2: u8 = 0x06;
    pub const VK_BACK: u8 = 0x08;
    pub const VK_TAB: u8 = 0x09;
    pub const VK_CLEAR: u8 = 0x0C;
    pub const VK_RETURN: u8 = 0x0D;
    pub const VK_SHIFT: u8 = 0x10;
    pub const VK_CONTROL: u8 = 0x11;
    pub const VK_MENU: u8 = 0x12;
    pub const VK_PAUSE: u8 = 0x13;
    pub const VK_CAPITAL: u8 = 0x14;
    pub const VK_KANA: u8 = 0x15;
    pub const VK_HANGEUL: u8 = 0x15;
    pub const VK_HANGUL: u8 = 0x15;
    pub const VK_JUNJA: u8 = 0x17;
    pub const VK_FINAL: u8 = 0x18;
    pub const VK_HANJA: u8 = 0x19;
    pub const VK_KANJI: u8 = 0x19;
    pub const VK_ESCAPE: u8 = 0x1B;
    pub const VK_CONVERT: u8 = 0x1C;
    pub const VK_NONCONVERT: u8 = 0x1D;
    pub const VK_ACCEPT: u8 = 0x1E;
    pub const VK_MODECHANGE: u8 = 0x1F;
    pub const VK_SPACE: u8 = 0x20;
    pub const VK_PRIOR: u8 = 0x21;
    pub const VK_NEXT: u8 = 0x22;
    pub const VK_END: u8 = 0x23;
    pub const VK_HOME: u8 = 0x24;
    pub const VK_LEFT: u8 = 0x25;
    pub const VK_UP: u8 = 0x26;
    pub const VK_RIGHT: u8 = 0x27;
    pub const VK_DOWN: u8 = 0x28;
    pub const VK_SELECT: u8 = 0x29;
    pub const VK_PRINT: u8 = 0x2A;
    pub const VK_EXECUTE: u8 = 0x2B;
    pub const VK_SNAPSHOT: u8 = 0x2C;
    pub const VK_INSERT: u8 = 0x2D;
    pub const VK_DELETE: u8 = 0x2E;
    pub const VK_HELP: u8 = 0x2F;

    pub const VK_LWIN: u8 = 0x5B;
    pub const VK_RWIN: u8 = 0x5C;
    pub const VK_APPS: u8 = 0x5D;
    pub const VK_SLEEP: u8 = 0x5F;
    pub const VK_NUMPAD0: u8 = 0x60;
    pub const VK_NUMPAD1: u8 = 0x61;
    pub const VK_NUMPAD2: u8 = 0x62;
    pub const VK_NUMPAD3: u8 = 0x63;
    pub const VK_NUMPAD4: u8 = 0x64;
    pub const VK_NUMPAD5: u8 = 0x65;
    pub const VK_NUMPAD6: u8 = 0x66;
    pub const VK_NUMPAD7: u8 = 0x67;
    pub const VK_NUMPAD8: u8 = 0x68;
    pub const VK_NUMPAD9: u8 = 0x69;
    pub const VK_MULTIPLY: u8 = 0x6A;
    pub const VK_ADD: u8 = 0x6B;
    pub const VK_SEPARATOR: u8 = 0x6C;
    pub const VK_SUBTRACT: u8 = 0x6D;
    pub const VK_DECIMAL: u8 = 0x6E;
    pub const VK_DIVIDE: u8 = 0x6F;
    pub const VK_F1: u8 = 0x70;
    pub const VK_F2: u8 = 0x71;
    pub const VK_F3: u8 = 0x72;
    pub const VK_F4: u8 = 0x73;
    pub const VK_F5: u8 = 0x74;
    pub const VK_F6: u8 = 0x75;
    pub const VK_F7: u8 = 0x76;
    pub const VK_F8: u8 = 0x77;
    pub const VK_F9: u8 = 0x78;
    pub const VK_F10: u8 = 0x79;
    pub const VK_F11: u8 = 0x7A;
    pub const VK_F12: u8 = 0x7B;
    pub const VK_F13: u8 = 0x7C;
    pub const VK_F14: u8 = 0x7D;
    pub const VK_F15: u8 = 0x7E;
    pub const VK_F16: u8 = 0x7F;
    pub const VK_F17: u8 = 0x80;
    pub const VK_F18: u8 = 0x81;
    pub const VK_F19: u8 = 0x82;
    pub const VK_F20: u8 = 0x83;
    pub const VK_F21: u8 = 0x84;
    pub const VK_F22: u8 = 0x85;
    pub const VK_F23: u8 = 0x86;
    pub const VK_F24: u8 = 0x87;
    pub const VK_NUMLOCK: u8 = 0x90;
    pub const VK_SCROLL: u8 = 0x91;
    pub const VK_OEM_NEC_EQUAL: u8 = 0x92;
    pub const VK_OEM_FJ_JISHO: u8 = 0x92;
    pub const VK_OEM_FJ_MASSHOU: u8 = 0x93;
    pub const VK_OEM_FJ_TOUROKU: u8 = 0x94;
    pub const VK_OEM_FJ_LOYA: u8 = 0x95;
    pub const VK_OEM_FJ_ROYA: u8 = 0x96;
    pub const VK_LSHIFT: u8 = 0xA0;
    pub const VK_RSHIFT: u8 = 0xA1;
    pub const VK_LCONTROL: u8 = 0xA2;
    pub const VK_RCONTROL: u8 = 0xA3;
    pub const VK_LMENU: u8 = 0xA4;
    pub const VK_RMENU: u8 = 0xA5;
    pub const VK_BROWSER_BACK: u8 = 0xA6;
    pub const VK_BROWSER_FORWARD: u8 = 0xA7;
    pub const VK_BROWSER_REFRESH: u8 = 0xA8;
    pub const VK_BROWSER_STOP: u8 = 0xA9;
    pub const VK_BROWSER_SEARCH: u8 = 0xAA;
    pub const VK_BROWSER_FAVORITES: u8 = 0xAB;
    pub const VK_BROWSER_HOME: u8 = 0xAC;
    pub const VK_VOLUME_MUTE: u8 = 0xAD;
    pub const VK_VOLUME_DOWN: u8 = 0xAE;
    pub const VK_VOLUME_UP: u8 = 0xAF;
    pub const VK_MEDIA_NEXT_TRACK: u8 = 0xB0;
    pub const VK_MEDIA_PREV_TRACK: u8 = 0xB1;
    pub const VK_MEDIA_STOP: u8 = 0xB2;
    pub const VK_MEDIA_PLAY_PAUSE: u8 = 0xB3;
    pub const VK_LAUNCH_MAIL: u8 = 0xB4;
    pub const VK_LAUNCH_MEDIA_SELECT: u8 = 0xB5;
    pub const VK_LAUNCH_APP1: u8 = 0xB6;
    pub const VK_LAUNCH_APP2: u8 = 0xB7;
    pub const VK_OEM_1: u8 = 0xBA;
    pub const VK_OEM_PLUS: u8 = 0xBB;
    pub const VK_OEM_COMMA: u8 = 0xBC;
    pub const VK_OEM_MINUS: u8 = 0xBD;
    pub const VK_OEM_PERIOD: u8 = 0xBE;
    pub const VK_OEM_2: u8 = 0xBF;
    pub const VK_OEM_3: u8 = 0xC0;
    pub const VK_OEM_4: u8 = 0xDB;
    pub const VK_OEM_5: u8 = 0xDC;
    pub const VK_OEM_6: u8 = 0xDD;
    pub const VK_OEM_7: u8 = 0xDE;
    pub const VK_OEM_8: u8 = 0xDF;
    pub const VK_OEM_AX: u8 = 0xE1;
    pub const VK_OEM_102: u8 = 0xE2;
    pub const VK_ICO_HELP: u8 = 0xE3;
    pub const VK_ICO_00: u8 = 0xE4;
    pub const VK_PROCESSKEY: u8 = 0xE5;
    pub const VK_ICO_CLEAR: u8 = 0xE6;
    pub const VK_PACKET: u8 = 0xE7;
    pub const VK_OEM_RESET: u8 = 0xE9;
    pub const VK_OEM_JUMP: u8 = 0xEA;
    pub const VK_OEM_PA1: u8 = 0xEB;
    pub const VK_OEM_PA2: u8 = 0xEC;
    pub const VK_OEM_PA3: u8 = 0xED;
    pub const VK_OEM_WSCTRL: u8 = 0xEE;
    pub const VK_OEM_CUSEL: u8 = 0xEF;
    pub const VK_OEM_ATTN: u8 = 0xF0;
    pub const VK_OEM_FINISH: u8 = 0xF1;
    pub const VK_OEM_COPY: u8 = 0xF2;
    pub const VK_OEM_AUTO: u8 = 0xF3;
    pub const VK_OEM_ENLW: u8 = 0xF4;
    pub const VK_OEM_BACKTAB: u8 = 0xF5;
    pub const VK_ATTN: u8 = 0xF6;
    pub const VK_CRSEL: u8 = 0xF7;
    pub const VK_EXSEL: u8 = 0xF8;
    pub const VK_EREOF: u8 = 0xF9;
    pub const VK_PLAY: u8 = 0xFA;
    pub const VK_ZOOM: u8 = 0xFB;
    pub const VK_NONAME: u8 = 0xFC;
    pub const VK_PA1: u8 = 0xFD;
    pub const VK_OEM_CLEAR: u8 = 0xFE;
}