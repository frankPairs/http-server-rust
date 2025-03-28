use std::{collections::HashMap, io::Write, net::TcpStream, vec};

use crate::encoding::CompressionSchema;

#[derive(Debug, Clone)]
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

#[derive(Debug)]
pub struct ResponseBuilder<'a> {
    pub public_folder: Option<String>,
    version: String,
    headers: HashMap<String, String>,
    compression_schemas: Vec<CompressionSchema>,
    stream: &'a mut TcpStream,
    status_code: StatusCode,
    body: Vec<u8>,
}

impl<'a> ResponseBuilder<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            public_folder: None,
            compression_schemas: vec![],
            stream,
            status_code: StatusCode::Ok,
            body: Vec::new(),
        }
    }

    pub fn with_version(&mut self, version: String) {
        self.version = version
    }

    pub fn with_public_folder(&mut self, public_folder: Option<String>) {
        self.public_folder = public_folder
    }

    pub fn with_compression_schemas(&mut self, compression_schemas: Vec<CompressionSchema>) {
        self.compression_schemas = compression_schemas
    }

    pub fn with_status_code(&mut self, status_code: StatusCode) {
        self.status_code = status_code;
    }

    pub fn send_status_code(&mut self, status_code: StatusCode) {
        self.status_code = status_code;

        self.send();
    }

    pub fn send_text(&mut self, text: &str) {
        self.headers
            .insert("Content-Type".to_string(), "text/plain".to_string());
        self.body = text.as_bytes().to_vec();

        self.send();
    }

    pub fn send_file(&mut self, content: &str) {
        self.headers.insert(
            "Content-Type".to_string(),
            "application/octet-stream".to_string(),
        );
        self.body = content.as_bytes().to_vec();

        self.send();
    }

    pub fn send(&mut self) {
        let body = &self.get_body();

        if !self.compression_schemas.is_empty() {
            self.insert_encoding_header();
        }

        if !body.is_empty() {
            self.headers
                .insert("Content-Length".to_string(), body.len().to_string());
        }

        let headers_string = self.convert_headers_into_string();

        let response = format!(
            "{} {}{}\r\n\r\n",
            self.version, self.status_code, headers_string
        );

        self.stream
            .write_all(response.as_bytes())
            .expect("Could not write a response");

        if !self.body.is_empty() {
            self.stream
                .write_all(&self.get_body())
                .expect("Could not write the body");
        }
    }

    fn get_body(&self) -> Vec<u8> {
        if self.body.is_empty() || self.compression_schemas.is_empty() {
            return self.body.clone();
        }

        let schema = self.compression_schemas.first().unwrap();

        match schema.compress(self.body.clone()) {
            Ok(compressed_body) => compressed_body,
            // If there is an error on the compression process, we return the body decompress
            Err(_) => self.body.clone(),
        }
    }

    fn insert_encoding_header(&mut self) {
        let compression_schemas_str: Vec<String> = self
            .compression_schemas
            .iter()
            .map(|v| v.to_string())
            .collect();

        self.headers.insert(
            "Content-Encoding".to_string(),
            compression_schemas_str.join(", "),
        );
    }

    fn convert_headers_into_string(&self) -> String {
        if self.headers.is_empty() {
            return "".to_string();
        }

        let mut headers_strings: Vec<String> = vec![];

        for (k, v) in self.headers.iter() {
            headers_strings.push(format!("{}: {}", k, v));
        }

        format!("\r\n{}", headers_strings.join("\r\n"))
    }
}
