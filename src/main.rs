use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
};

#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
}

impl From<String> for Request {
    fn from(buffer: String) -> Self {
        let request_parts: Vec<String> = buffer.split("\r\n").map(String::from).collect();

        let request_line = request_parts.first().unwrap_or(&String::new()).to_string();
        let request_line: Vec<String> = request_line.split(" ").map(String::from).collect();

        let method = request_line.first().unwrap_or(&String::new()).to_string();
        let path = request_line
            .get(1)
            .unwrap_or(&String::from("/"))
            .to_string();
        let version = request_line.get(2).unwrap_or(&String::new()).to_string();

        Self {
            method,
            path,
            version,
            headers: HashMap::new(),
        }
    }
}

fn handle_client(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];

    if let Err(err) = stream.read(&mut buffer) {
        println!("error: {}", err);
    }

    let req: Request = Request::from(String::from_utf8_lossy(&buffer).to_string());

    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") => {
            println!("Request OK");

            stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
        }
        _ => {
            println!("Request Not Found");

            stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
        }
    }
}

fn main() {
    // Uncomment this block to pass the first stage
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_client(&mut stream);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
