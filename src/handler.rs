use crate::{request::Request, response::Response};

pub type HandlerFn = fn(Request, Response);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct HandlerPattern(pub String, pub String);

impl HandlerPattern {
    pub fn get_method(&self) -> String {
        self.0.clone()
    }

    pub fn get_path(&self) -> String {
        self.1.clone()
    }
}

impl HandlerPattern {
    pub fn contains_pattern(&self, request: &Request) -> bool {
        let method = self.get_method();
        let path = self.get_path();

        if method != request.method {
            return false;
        }

        if !path.starts_with('/') {
            return false;
        }

        if path == "/" && request.path == "/" {
            return true;
        }

        let path_values: Vec<&str> = path.split('/').collect();
        let req_path_values: Vec<&str> = request.path.split('/').collect();

        if path_values.len() != req_path_values.len() {
            return false;
        }

        // It analyzes each fragment of the url and checks if it matches with the current pattern
        for (index, pattern_value) in path_values.into_iter().enumerate() {
            // It consider the pattern fragment equals to the request fragment because we know that
            // when a path fragment starts with '{' and end with '}' it's a path parameter.
            // e.g. /echo/{str} == /echo/abc
            if pattern_value
                .strip_prefix('{')
                .and_then(|word| word.strip_suffix('}'))
                .is_some()
            {
                continue;
            }

            if pattern_value != *req_path_values.get(index).unwrap() {
                return false;
            }
        }

        true
    }
}
