extern crate inline_assets;

use std::fs;

//const RESOURCES_PATH: &str = "src/resources/";
//macro_rules! return_none {
//    ($e:expr) => {
//        match $e {
//            Some(c) => c,
//            None => return
//        }
//    };
//}

fn main() {
    fs::write("src/server/html_frontend/connection_screen.html.embedded",
              inline_assets::inline_file("src/server/html_frontend/connection_screen.html", inline_assets::Config {inline_fonts: true, remove_new_lines: false}).expect("Path error").as_bytes()).expect("File write failed");
    fs::write("src/listener/html_frontend/listener_screen.html.embedded",
              inline_assets::inline_file("src/listener/html_frontend/listener_screen.html", inline_assets::Config {inline_fonts: true, remove_new_lines: false}).expect("Path error").as_bytes()).expect("File write failed");
}
