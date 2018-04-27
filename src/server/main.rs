use std::net::TcpStream;
use std::io::Write;
use std::io::Read;
fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:13111").expect("Can't connect to port: ");
    let mut stdin = std::io::stdin();
    let mut line_buffer = String::new();
    loop {
        line_buffer.clear();
        stdin.read_line(&mut line_buffer);
        stream.write( line_buffer.trim().as_bytes()).expect("Failed to write");
    }
}
