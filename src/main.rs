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

    server.handle_fn("GET".to_string(), "/".to_string(), |_, res| {
        res.send();
    });

    server.handle_fn("GET".to_string(), "/echo/{str}".to_string(), |req, res| {
        let str_value = req.path_params.get("str");

        res.send_text(str_value.map(|value| value.as_str()).unwrap_or_default());
    });

    server.handle_fn("GET".to_string(), "/user-agent".to_string(), |req, res| {
        if let Some(user_agent) = req.headers.get("User-Agent") {
            res.send_text(user_agent);
        } else {
            res.send_status_code(StatusCode::BadRequest);
        }
    });

    server.handle_fn(
        "GET".to_string(),
        "/files/{filename}".to_string(),
        |req, res| {
            if let Some(filename) = req.path_params.get("filename") {
                if res.public_folder.is_none() {
                    res.send_status_code(StatusCode::InternalServer);
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
                            res.send_status_code(StatusCode::NotFound);
                        }
                        _ => {
                            res.send_status_code(StatusCode::InternalServer);
                        }
                    },
                }
            } else {
                res.send_status_code(StatusCode::BadRequest);
            }
        },
    );

    server.handle_fn(
        "POST".to_string(),
        "/files/{filename}".to_string(),
        |req, res| {
            if let Some(filename) = req.path_params.get("filename") {
                if res.public_folder.is_none() {
                    println!("Missing public folder.");

                    res.send_status_code(StatusCode::InternalServer);
                }

                let public_folder = res.public_folder.as_ref().unwrap();

                let result = FileManager::write(public_folder, filename, req.body.as_str());

                match result {
                    Ok(()) => {
                        res.send_status_code(StatusCode::Created);
                    }
                    Err(_) => {
                        res.send_status_code(StatusCode::InternalServer);
                    }
                }
            } else {
                res.send_status_code(StatusCode::BadRequest);
            }
        },
    );

    server.listen("127.0.0.1:4221".to_string());
}
