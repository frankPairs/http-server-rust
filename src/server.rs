use std::{collections::HashMap, net::TcpListener};

use crate::{
    request::Request,
    response::{ResponseBuilder, StatusCode},
};

pub type HandlerFn = fn(&Request, &mut ResponseBuilder);

#[derive(Debug, Default)]
pub struct ServerHTTP {
    handlers: HashMap<String, HandlerFn>,
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
                        let mut contain_matches = false;
                        let mut req = Request::new(&mut stream);
                        let mut res = ResponseBuilder::new(&mut stream);

                        res.with_compression_schemas(req.get_compression_schemas());
                        res.with_public_folder(public_folder);
                        res.with_version(req.version.clone());

                        for (k, h) in &handlers {
                            let (method, pattern) = k.split_once(":").unwrap();

                            if req.method_and_pattern_matches(method, pattern) {
                                h(&mut req, &mut res);

                                contain_matches = true;

                                break;
                            }
                        }

                        if !contain_matches {
                            res.send_status_code(StatusCode::NotFound);
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

    pub fn handle_fn(
        &mut self,
        method: String,
        pattern: String,
        handler_fn: fn(&Request, &mut ResponseBuilder),
    ) {
        let key = format!("{}:{}", method, pattern);
        let exists = self.handlers.get(&key);

        if exists.is_none() {
            self.handlers
                .insert(format!("{}:{}", method, pattern), handler_fn);
        }
    }

    pub fn set_public_folder(&mut self, public_folder: &str) {
        self.public_folder = Some(public_folder.to_string());
    }
}
