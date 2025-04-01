use std::path::PathBuf;

use clap::Parser;
use codecrafters_http_server::{
    file_manager::{FileManager, FileManagerError},
    response::StatusCode,
    server::ServerHTTP,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Public server directory
    #[arg(short, long)]
    directory: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut server = ServerHTTP::default();

    if let Some(dir) = args.directory {
        server.set_public_folder(dir.as_str());
    }

    server.handle_fn("GET", "/", |_, res| {
        res.send();
    });

    server.handle_fn("GET", "/echo/{str}", |req, res| {
        let str_value = req.path_params.get("str");

        res.send_text(str_value.map(|value| value.as_str()).unwrap_or_default());
    });

    server.handle_fn("GET", "/user-agent", |req, res| {
        if let Some(user_agent) = req.headers.get("User-Agent") {
            res.send_text(user_agent);
        } else {
            res.status_code(StatusCode::BadRequest).send();
        }
    });

    server.handle_fn("GET", "/files/{filename}", |req, res| {
        if let Some(filename) = req.path_params.get("filename") {
            if res.public_folder.is_none() {
                res.status_code(StatusCode::InternalServer).send();

                return;
            }

            let mut path = PathBuf::new();

            path.push(res.public_folder.as_ref().unwrap());
            path.push(filename);

            let result = FileManager::read(path);

            match result {
                Ok(read_result) => {
                    res.send_file(&read_result.content);
                }
                Err(err) => match err {
                    FileManagerError::NotFound => {
                        res.status_code(StatusCode::NotFound).send();
                    }
                    _ => {
                        res.status_code(StatusCode::InternalServer).send();
                    }
                },
            }
        } else {
            res.status_code(StatusCode::BadRequest).send();
        }
    });

    server.handle_fn("POST", "/files/{filename}", |req, res| {
        if let Some(filename) = req.path_params.get("filename") {
            if res.public_folder.is_none() {
                eprintln!("Missing public folder.");

                res.status_code(StatusCode::InternalServer).send();

                return;
            }

            let public_folder = res.public_folder.as_ref().unwrap();

            let result = FileManager::write(public_folder, filename, req.body.as_str());

            match result {
                Ok(()) => {
                    res.status_code(StatusCode::Created).send();
                }
                Err(_) => {
                    res.status_code(StatusCode::InternalServer).send();
                }
            }
        } else {
            res.status_code(StatusCode::BadRequest).send();
        }
    });

    server.listen("127.0.0.1:4221".to_string());
}
