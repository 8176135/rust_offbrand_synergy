extern crate libxdo_sys;

#[derive(Clone)]
pub struct Xdo {
    inst: *mut libxdo_sys::Struct_xdo
}

#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Xdo {
    pub fn new() -> Xdo {
        unsafe {
            Xdo {
                inst: libxdo_sys::xdo_new(std::ptr::null()),
            }
        }
    }

    /// Returns the cursor location as `(Point, screen_number)`
    ///
    pub fn get_mouse_location(&self) -> (Point, i32) {
        let (mut point, mut scrn_num) = (Point { x: 0, y: 0 }, 0);
        unsafe {
            libxdo_sys::xdo_get_mouse_location(self.inst, &mut point.x as *mut i32, &mut point.y as *mut i32, &mut scrn_num as *mut i32);
        }
        (point, scrn_num)
    }

}


#[cfg(test)]
mod tests {
    use ::Xdo;

    #[test]
    fn it_works() {
        let xdo = Xdo::new();
    }
}
