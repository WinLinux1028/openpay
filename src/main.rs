use axum::{handler::Handler, http::StatusCode, response, routing};

#[tokio::main]
async fn main() {
    let base = routing::any(method_not_allowed);

    let frontend = routing::Router::new().route("/", base.clone().get(hello_world));

    let api = routing::Router::new();

    let app = routing::Router::new()
        .fallback(not_found.into_service())
        .route("/", routing::any(redirect_to_frontend))
        .nest("/api", api)
        .nest("/frontend", frontend);

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn hello_world() -> response::Html<&'static str> {
    response::Html("Hello World!")
}

async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "404 Not Found")
}

async fn method_not_allowed() -> (StatusCode, &'static str) {
    (StatusCode::METHOD_NOT_ALLOWED, "405 Method Not Allowed")
}

async fn redirect_to_frontend() -> response::Redirect {
    response::Redirect::permanent("/frontend")
}
