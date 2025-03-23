use codecrafters_http_server::{response::StatusCode, server::ServerHTTP};

fn main() {
    // Uncomment this block to pass the first stage
    let mut server = ServerHTTP::default();

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
            if let Some(user_agent) = req.headers.get("User-Agent") {
                res.text(Some(user_agent), None);
            } else {
                res.status_code(StatusCode::BadRequest);
            }
        },
    );

    server.listen("127.0.0.1:4221".to_string());
}
