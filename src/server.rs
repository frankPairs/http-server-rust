use std::{collections::HashMap, net::TcpListener};

use crate::{
    handler::{HandlerFn, HandlerPattern},
    request::Request,
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
            let handlers = self.handlers.clone();
            let public_folder = self.public_folder.clone();

            match stream {
                Ok(mut stream) => {
                    std::thread::spawn(move || {
                        let mut req = Request::new(&mut stream);
                        let res = ResponseBuilder::new(&mut stream)
                            .with_compression_schemas(req.get_compression_schemas())
                            .with_public_folder(public_folder)
                            .with_version(req.version.clone())
                            .build();

                        let handler = handlers.iter().find(|h| {
                            let pattern = h.0;

                            pattern.contains_pattern(&req)
                        });

                        match handler {
                            Some(h) => {
                                let pattern = h.0;
                                let handle_fn = h.1;

                                req.set_path_params(&pattern.get_path());

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
