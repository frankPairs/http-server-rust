use std::{collections::HashMap, io::Write, net::TcpStream};

use crate::encoding::{self, CompressionSchema, CompressionSchemaError};

pub enum StatusCode {
    Ok,
    NotFound,
    BadRequest,
    InternalServer,
    Created,
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusCode::Ok => {
                write!(f, "200 OK")
            }
            StatusCode::Created => {
                write!(f, "201 Created")
            }
            StatusCode::NotFound => {
                write!(f, "404 Not Found")
            }
            StatusCode::BadRequest => {
                write!(f, "400 Bad Request")
            }
            StatusCode::InternalServer => {
                write!(f, "500 Internal Server Error")
            }
        }
    }
}

pub struct Response<'a> {
    pub version: String,
    pub headers: HashMap<String, String>,
    pub public_folder: Option<String>,
    pub compression_schema: Option<CompressionSchema>,
    stream: &'a mut TcpStream,
}

pub struct ResponseOptions {
    status_code: StatusCode,
}

impl<'a> Response<'a> {
    pub fn new(
        stream: &'a mut TcpStream,
        version: &str,
        public_folder: Option<String>,
        compression_schema: Option<String>,
    ) -> Self {
        let compression_schema: Option<Result<CompressionSchema, CompressionSchemaError>> =
            compression_schema.map(|v| v.try_into());

        Response {
            stream,
            version: String::from(version),
            headers: HashMap::new(),
            public_folder,
            compression_schema: match compression_schema {
                Some(Ok(schema)) => Some(schema),
                Some(Err(_)) => None,
                None => None,
            },
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

        self.write_response(status_code, text);
    }

    pub fn file(&mut self, content: &str, bytes_read: usize, opts: Option<ResponseOptions>) {
        let status_code = opts.map(|opts| opts.status_code).unwrap_or(StatusCode::Ok);

        self.headers.insert(
            "Content-Type".to_string(),
            "application/octet-stream".to_string(),
        );

        self.headers
            .insert("Content-Length".to_string(), bytes_read.to_string());

        self.write_response(status_code, Some(content));
    }

    fn convert_headers_into_string(&self) -> String {
        let mut headers_strings: Vec<String> = vec![];

        for (k, v) in self.headers.iter() {
            headers_strings.push(format!("{}: {}", k, v));
        }

        headers_strings.join("\r\n")
    }

    fn write_response(&mut self, status_code: StatusCode, body: Option<&str>) {
        if let Some(v) = &self.compression_schema {
            self.headers
                .insert("Content-Encoding".to_string(), v.to_string());
        }

        let response_str = match body {
            Some(b) => {
                let headers_string = self.convert_headers_into_string();

                format!(
                    "{} {}\r\n{}\r\n\r\n{}",
                    self.version, status_code, headers_string, b
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
}
