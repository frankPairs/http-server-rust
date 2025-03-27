use std::{collections::HashMap, io::prelude::*, net::TcpStream};

use crate::encoding::{CompressionSchema, CompressionSchemaError};

const MAX_BYTES_STREAM_BUFFER: usize = 256;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub path_params: HashMap<String, String>,
    pub body: String,
}

impl Request {
    pub fn new(stream: &mut TcpStream) -> Self {
        let mut bytes_received: Vec<u8> = vec![];
        let mut buffer = [0u8; MAX_BYTES_STREAM_BUFFER];

        loop {
            let bytes_read = stream.read(&mut buffer).unwrap();

            bytes_received.extend_from_slice(&buffer[..bytes_read]);

            if bytes_read < MAX_BYTES_STREAM_BUFFER {
                break;
            }
        }

        let request_string = String::from_utf8_lossy(&bytes_received);
        let (request_info, request_body) = request_string.split_once("\r\n\r\n").unwrap();

        let request_parts: Vec<String> = request_info.split("\r\n").map(String::from).collect();

        let request_line: Vec<String> = request_parts
            .first()
            .unwrap_or(&String::new())
            .split(" ")
            .map(String::from)
            .collect();

        let method = request_line.first().unwrap_or(&String::new()).to_string();
        let path = request_line
            .get(1)
            .unwrap_or(&String::from("/"))
            .to_string();

        let version = request_line.get(2).unwrap_or(&String::new()).to_string();

        let request_headers: Option<Vec<String>> =
            request_parts.get(1..).map(|parts| parts.to_vec());

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
            body: String::from(request_body),
        }
    }

    pub fn method_and_pattern_matches(&mut self, method: &str, pattern: &str) -> bool {
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
                // When pattern and path are differents, we clean the path_params variable as we do not need them
                // anymore
                self.path_params = HashMap::new();

                return false;
            }
        }

        true
    }

    pub fn get_compression_schemas(&self) -> Vec<CompressionSchema> {
        let mut compression_schemas: Vec<CompressionSchema> = vec![];
        let encoding = self.headers.get("Accept-Encoding");

        if let Some(v) = encoding {
            let schemas: Vec<String> = v.split(", ").map(|v| v.to_string()).collect();

            for schema in schemas {
                let compression_schema: Result<CompressionSchema, CompressionSchemaError> =
                    schema.try_into();

                if let Ok(cs) = compression_schema {
                    compression_schemas.push(cs);
                }
            }
        }

        compression_schemas
    }
}
