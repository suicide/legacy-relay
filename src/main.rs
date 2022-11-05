use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};

#[tokio::main]
async fn main() {
    println!("Starting...");

    let app = Router::new().route("/*path", get(get_redirect));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_redirect(headers: HeaderMap, Path(path): Path<String>) -> impl IntoResponse {
    let host = headers.get("host").map(|s| s.to_str()).unwrap();
    let host_string = String::from(host.unwrap());

    let search = host_string + &path;

    let redirect = format!("https://{}", &search);

    (
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, redirect)],
    )
}
