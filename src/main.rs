use clap::Parser;
use codecrafters_http_server::{response::StatusCode, server::ServerHTTP};

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
        res.status_code(StatusCode::Ok);
    });

    server.handle_fn("GET".to_string(), "/echo/{str}".to_string(), |req, res| {
        let str_value = req.path_params.get("str");

        res.text(str_value.map(|value| value.as_str()), None);
    });

    server.handle_fn("GET".to_string(), "/user-agent".to_string(), |req, res| {
        if let Some(user_agent) = req.headers.get("User-Agent") {
            res.text(Some(user_agent), None);
        } else {
            res.status_code(StatusCode::BadRequest);
        }
    });

    server.handle_fn(
        "GET".to_string(),
        "/files/{filename}".to_string(),
        |req, res| {
            if let Some(filename) = req.path_params.get("filename") {
                res.file(filename, None);
            } else {
                res.status_code(StatusCode::BadRequest);
            }
        },
    );

    server.listen("127.0.0.1:4221".to_string());
}
