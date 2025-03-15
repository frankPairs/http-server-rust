use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
};

const MAX_BYTES_STREAM_BUFFER: usize = 100;

enum StatusCode {
    Ok,
    NotFound,
    BadRequest,
    MethodNotAllowed,
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusCode::Ok => {
                write!(f, "200 OK")
            }
            StatusCode::NotFound => {
                write!(f, "404 Not Found")
            }
            StatusCode::MethodNotAllowed => {
                write!(f, "405 Method Not Allowed")
            }
            StatusCode::BadRequest => {
                write!(f, "400 Bad Request")
            }
        }
    }
}

type HandlerFn = fn(&Request, &mut Response);
#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    path_params: HashMap<String, String>,
}

impl Request {
    fn new(stream: &mut TcpStream) -> Self {
        let mut bytes_received: Vec<u8> = vec![];
        let mut buffer = [0u8; MAX_BYTES_STREAM_BUFFER];

        loop {
            // Read from the current data in the TcpStream
            let bytes_read = stream.read(&mut buffer).unwrap();

            // However many bytes we read, extend the `received` string bytes
            bytes_received.extend_from_slice(&buffer[..bytes_read]);

            // If we didn't fill the array
            // stop reading because there's no more data (we hope!)
            if bytes_read < MAX_BYTES_STREAM_BUFFER {
                break;
            }
        }

        let request_string = String::from_utf8_lossy(&bytes_received);
        let request_parts: Vec<String> = request_string
            .replace("\r\n\r\n", "")
            .split("\r\n")
            .map(String::from)
            .collect();

        let request_line = request_parts.first().unwrap_or(&String::new()).to_string();
        let request_line: Vec<String> = request_line.split(" ").map(String::from).collect();
        let request_headers: Option<Vec<String>> =
            request_parts.get(1..).map(|parts| parts.to_vec());

        let method = request_line.first().unwrap_or(&String::new()).to_string();
        let path = request_line
            .get(1)
            .unwrap_or(&String::from("/"))
            .to_string();

        let version = request_line.get(2).unwrap_or(&String::new()).to_string();

        let mut headers: HashMap<String, String> = HashMap::new();

        if let Some(request_headers) = request_headers {
            for header in request_headers {
                let (header_name, header_value) = header.split_once(":").unwrap();

                headers.insert(header_name.to_string(), header_value.trim().to_string());
            }
        }

        Self {
            method,
            path,
            version,
            headers,
            path_params: HashMap::new(),
        }
    }

    fn method_and_pattern_matches(&mut self, method: &str, pattern: &str) -> bool {
        if method != self.method {
            return false;
        }

        if !pattern.starts_with('/') {
            return false;
        }

        if pattern == "/" && self.path == "/" {
            return true;
        }

        let pattern_values: Vec<&str> = pattern.split('/').collect();
        let path_values: Vec<&str> = self.path.split('/').collect();

        if pattern_values.len() != path_values.len() {
            return false;
        }

        // It analyzes each fragment of the url and checks if it matches with the current pattern
        for (index, pattern_value) in pattern_values.into_iter().enumerate() {
            // Extracts the path parameters and insert it into the path_params map
            if let Some(param_name) = pattern_value
                .strip_prefix('{')
                .and_then(|word| word.strip_suffix('}'))
            {
                self.path_params.insert(
                    param_name.to_string(),
                    path_values.get(index).unwrap().to_string(),
                );

                continue;
            }

            if pattern_value != *path_values.get(index).unwrap() {
                return false;
            }
        }

        true
    }
}

struct Response<'a> {
    version: String,
    stream: &'a mut TcpStream,
    headers: HashMap<String, String>,
}

struct ResponseOptions {
    status_code: StatusCode,
}

impl<'a> Response<'a> {
    fn new(stream: &'a mut TcpStream, version: String) -> Self {
        Response {
            stream,
            version,
            headers: HashMap::new(),
        }
    }

    fn status_code(&mut self, status_code: StatusCode) {
        let response_str = format!("{} {}\r\n\r\n", self.version, status_code);

        self.stream.write_all(response_str.as_bytes()).unwrap();
    }

    fn text(&mut self, text: Option<&str>, opts: Option<ResponseOptions>) {
        let status_code = opts.map(|opts| opts.status_code).unwrap_or(StatusCode::Ok);

        self.headers
            .insert("Content-Type".to_string(), "text/plain".to_string());

        if let Some(t) = text {
            self.headers
                .insert("Content-Length".to_string(), t.len().to_string());
        }

        let response_str = match text {
            Some(body) => {
                let headers_string = self.convert_headers_into_string();

                format!(
                    "{} {}\r\n{}\r\n\r\n{}",
                    self.version, status_code, headers_string, body
                )
            }
            None => {
                format!("{} {}\r\n\r\n", self.version, status_code)
            }
        };

        self.stream
            .write_all(response_str.as_bytes())
            .expect("Could not write a response");
    }

    fn convert_headers_into_string(&self) -> String {
        let mut headers_strings: Vec<String> = vec![];

        for (k, v) in self.headers.iter() {
            headers_strings.push(format!("{}: {}", k, v));
        }

        headers_strings.join("\r\n")
    }
}

struct ServerHTTP {
    handlers: HashMap<String, HandlerFn>,
}

impl ServerHTTP {
    fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// "127.0.0.1:4221"
    fn listen(&self, host: String) {
        let listener = TcpListener::bind(host).expect("Error to connect with the host");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut contain_matches = false;
                    let mut req = Request::new(&mut stream);
                    let mut res = Response::new(&mut stream, req.version.clone());

                    for (k, h) in &self.handlers {
                        let (method, pattern) = k.split_once(":").unwrap();

                        if req.method_and_pattern_matches(method, pattern) {
                            h(&mut req, &mut res);

                            contain_matches = true
                        }
                    }

                    if !contain_matches {
                        res.status_code(StatusCode::NotFound);
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    panic!("Error when listening messages: {}", e);
                }
            }
        }
    }

    fn handle_fn(
        &mut self,
        method: String,
        pattern: String,
        handler_fn: fn(&Request, &mut Response),
    ) {
        let key = format!("{}:{}", method, pattern);
        let exists = self.handlers.get(&key);

        if exists.is_none() {
            self.handlers
                .insert(format!("{}:{}", method, pattern), handler_fn);
        }
    }
}

fn main() {
    // Uncomment this block to pass the first stage
    let mut server = ServerHTTP::new();

    server.handle_fn("GET".to_string(), "/".to_string(), |_, res| {
        res.status_code(StatusCode::Ok);
    });

    server.handle_fn("GET".to_string(), "/echo/{str}".to_string(), |req, res| {
        let str_value = req.path_params.get("str");

        res.text(str_value.map(|value| value.as_str()), None);
    });

    server.handle_fn("GET".to_string(), "/user-agent".to_string(), |req, res| {
        if let Some(user_agent) = req.headers.get("User-Agent") {
            res.text(Some(user_agent), None);
        } else {
            res.status_code(StatusCode::BadRequest);
        }
    });

    server.listen("127.0.0.1:4221".to_string());
}
