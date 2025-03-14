use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
};

enum StatusCode {
    Ok,
    NotFound,
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
        }
    }
}

trait Handler {
    fn serve(&self, req: &mut Request, res: &mut Response);
}

#[derive(Debug)]
struct HandlerFn {
    method: String,
    pattern: String,
    handler_fn: fn(&Request, &mut Response),
}

impl Handler for HandlerFn {
    fn serve(&self, req: &mut Request, res: &mut Response) {
        if self.method != req.method {
            return;
        }

        if !self.pattern.starts_with('/') {
            return;
        }

        if self.pattern == "/" && req.path == "/" {
            let handle_fn = self.handler_fn;

            handle_fn(req, res);

            return;
        }

        let pattern_values: Vec<&str> = self.pattern.split('/').collect();
        let path_values: Vec<&str> = req.path.split('/').collect();

        if pattern_values.len() != path_values.len() {
            return;
        }

        let mut path_params: HashMap<String, String> = HashMap::new();

        // It analyzes each fragment of the url and checks if it matches with the current pattern
        for (index, pattern_value) in pattern_values.into_iter().enumerate() {
            if let Some(param_name) = pattern_value
                .strip_prefix('{')
                .and_then(|word| word.strip_suffix('}'))
            {
                path_params.insert(
                    param_name.to_string(),
                    path_values.get(index).unwrap().to_string(),
                );

                continue;
            }

            if pattern_value != *path_values.get(index).unwrap() {
                return;
            }
        }

        req.set_path_params(path_params);

        let handle_fn = self.handler_fn;

        handle_fn(req, res);
    }
}

#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    version: String,
    headers: HashMap<String, String>,
    path_params: HashMap<String, String>,
}

impl Request {
    fn set_path_params(&mut self, new_path_params: HashMap<String, String>) {
        self.path_params = new_path_params;
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
        let response_str = format!("HTTP/1.1 {}\r\n\r\n", status_code);

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
    handlers: HashMap<String, Box<dyn Handler>>,
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
                    let mut req = Self::build_request(&mut stream);
                    let mut res = Response::new(&mut stream, req.version.clone());

                    for (k, h) in &self.handlers {
                        if (Self::pattern_matches(&req, k.to_string())) {
                            h.serve(&mut req, &mut res);
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
        let handler = HandlerFn {
            handler_fn,
            method: method.clone(),
            pattern: pattern.clone(),
        };

        self.handlers
            .insert(format!("{}:{}", method, pattern), Box::new(handler));
    }

    fn build_request(stream: &mut TcpStream) -> Request {
        let mut buffer = [0; 1024];

        if let Err(err) = stream.read(&mut buffer) {
            println!("error: {}", err);
        }

        let string_buffer = String::from_utf8_lossy(&buffer).to_string();
        let request_parts: Vec<String> = string_buffer.split("\r\n").map(String::from).collect();

        let request_line = request_parts.first().unwrap_or(&String::new()).to_string();
        let request_line: Vec<String> = request_line.split(" ").map(String::from).collect();

        let method = request_line.first().unwrap_or(&String::new()).to_string();
        let path = request_line
            .get(1)
            .unwrap_or(&String::from("/"))
            .to_string();

        let version = request_line.get(2).unwrap_or(&String::new()).to_string();

        Request {
            method,
            path,
            version,
            headers: HashMap::new(),
            path_params: HashMap::new(),
        }
    }

    fn pattern_matches(req: &Request, handler_key: String) -> bool {
        let (method, pattern) = handler_key.split_once(":").unwrap();

        if method != req.method {
            return false;
        }

        if !pattern.starts_with('/') {
            return false;
        }

        if pattern == "/" && req.path == "/" {
            return true;
        }

        let pattern_values: Vec<&str> = pattern.split('/').collect();
        let path_values: Vec<&str> = req.path.split('/').collect();

        if pattern_values.len() != path_values.len() {
            return false;
        }

        // It analyzes each fragment of the url and checks if it matches with the current pattern
        for (index, pattern_value) in pattern_values.into_iter().enumerate() {
            if let Some(param_name) = pattern_value
                .strip_prefix('{')
                .and_then(|word| word.strip_suffix('}'))
            {
                continue;
            }

            if pattern_value != *path_values.get(index).unwrap() {
                return false;
            }
        }

        true
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

    server.listen("127.0.0.1:4221".to_string());
}
