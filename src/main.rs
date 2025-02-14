use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

fn handle_client(stream: &mut TcpStream) {
    if let Err(err) = stream.write(b"HTTP/1.1 200 OK\r\n\r\n") {
        println!("error: {}", err);
    };

    if let Err(err) = stream.flush() {
        println!("error: {}", err);
    };
}

fn main() {
    // Uncomment this block to pass the first stage
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_client(&mut stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
