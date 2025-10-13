use std::{collections::HashMap, net::TcpListener};

use crate::{
    handler::{HandlerFn, HandlerPattern},
    request::{Request, RequestReader},
    response::{Response, ResponseBuilder, StatusCode},
};

#[derive(Debug, Default)]
pub struct ServerHTTP {
    handlers: HashMap<HandlerPattern, HandlerFn>,
    public_folder: Option<String>,
}

impl ServerHTTP {
    pub fn listen(&self, host: String) {
        let listener = TcpListener::bind(host).expect("Error to connect with the host");

        for stream in listener.incoming() {
            let handlers_cloned = self.handlers.clone();
            let public_folder_cloned = self.public_folder.clone();

            match stream {
                Ok(mut stream) => {
                    std::thread::spawn(move || loop {
                        let public_folder_cloned = public_folder_cloned.clone();
                        let handlers_cloned = handlers_cloned.clone();

                        let req_reader = RequestReader::new(&mut stream);
                        let req_message = req_reader.read_to_string().unwrap();

                        // In order to keep the connection alive, we are looping around this logic,
                        // so there are going to be times where the there is not any message to
                        // reply. We do not want to parse the request when message is empty, so we just
                        // continue the loop.
                        if req_message.is_empty() {
                            continue;
                        }

                        let mut req = Request::new(req_message);

                        let mut res = ResponseBuilder::new(&mut stream)
                            .with_compression_schemas(req.get_compression_schemas())
                            .with_public_folder(public_folder_cloned)
                            .with_version(req.version.clone())
                            .build();

                        let handler = handlers_cloned.iter().find(|h| {
                            let pattern = h.0;

                            pattern.contains_pattern(&req)
                        });

                        match handler {
                            Some(h) => {
                                let pattern = h.0;
                                let handle_fn = h.1;
                                let req_headers = req.headers.clone();

                                req.set_path_params(&pattern.get_path());

                                // When Connection header is present, and its value is 'close', then we
                                // jump out of the connection loop
                                if let Some(value) = req_headers.get("Connection") {
                                    if value.to_lowercase() == "close" {
                                        res.insert_header("Connection", "close");
                                        handle_fn(req, res);
                                        break;
                                    }
                                }

                                handle_fn(req, res);
                            }
                            None => {
                                res.status_code(StatusCode::NotFound).send();
                            }
                        }
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    panic!("Error when listening messages: {}", e);
                }
            }
        }
    }

    pub fn handle_fn(&mut self, method: &str, path: &str, handler_fn: fn(Request, Response)) {
        let handler_pattern = HandlerPattern(method.to_string(), path.to_string());

        self.handlers.entry(handler_pattern).or_insert(handler_fn);
    }

    pub fn set_public_folder(&mut self, public_folder: &str) {
        self.public_folder = Some(public_folder.to_string());
    }
}
