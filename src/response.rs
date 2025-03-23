use std::{collections::HashMap, io::Write, net::TcpStream};

pub enum StatusCode {
    Ok,
    NotFound,
    BadRequest,
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
            StatusCode::BadRequest => {
                write!(f, "400 Bad Request")
            }
        }
    }
}

pub struct Response<'a> {
    pub version: String,
    stream: &'a mut TcpStream,
    pub headers: HashMap<String, String>,
}

pub struct ResponseOptions {
    status_code: StatusCode,
}

impl<'a> Response<'a> {
    pub fn new(stream: &'a mut TcpStream, version: String) -> Self {
        Response {
            stream,
            version,
            headers: HashMap::new(),
        }
    }

    pub fn status_code(&mut self, status_code: StatusCode) {
        let response_str = format!("{} {}\r\n\r\n", self.version, status_code);

        self.stream.write_all(response_str.as_bytes()).unwrap();
    }

    pub fn text(&mut self, text: Option<&str>, opts: Option<ResponseOptions>) {
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
