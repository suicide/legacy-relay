use anyhow::Result;
use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router, Extension,
};
use mongodb::{options::{ClientOptions, FindOptions}, Client, bson::doc, Collection};
use serde::{Deserialize, Serialize};
use futures::stream::TryStreamExt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyUrl {
    content_id: i32,
    url: String,
    disabled: bool
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RedirectUrl {
    url: String
}


#[tokio::main]
async fn main() {
    println!("Starting...");

    let client_options = ClientOptions::parse("mongodb://admin:admin@localhost:27017")
        .await.unwrap();

    let client = Client::with_options(client_options).unwrap();

    let collection = client.database("legacyurl").collection::<LegacyUrl>("LegacyUrl");


    let app = Router::new().route("/*path", get(get_redirect))
        .layer(Extension(collection));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_redirect(headers: HeaderMap,
                      Path(path): Path<String>,
                      Extension(collection): Extension<Collection<LegacyUrl>>) -> impl IntoResponse {
    let host = headers.get("host").map(|s| s.to_str()).unwrap();
    let host_string = String::from(host.unwrap());

    let search = host_string + &path;

    let content_id = query_mongo_for_id(collection, search)
        .await.unwrap();


    let redirect = match content_id {
        Some(id) => query_redirect_url(id).await.unwrap(),
        None => None
    };

    let redirect = match redirect {
        Some(url) => url,
        None => "default url".to_string(),
    };

    (
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, redirect)],
    )
}

async fn query_redirect_url(content_id: i32) -> Result<Option<String>> {

    let request_url = format!("https://beepop.free.beeceptor.com/getUrl/{}",
                              content_id);
    let response = reqwest::get(request_url).await?;
    let redirect_url: RedirectUrl = response.json().await?;

    Ok(Some(redirect_url.url))
}

async fn query_mongo_for_id(collection: Collection<LegacyUrl>, search: String) -> Result<Option<i32>> {

    let filter = doc! { "url": search, "disabled": false };
    let find_opts = FindOptions::builder()
        .sort(doc! { "modified": -1 })
        .limit(1)
        .build();

    let mut cursor = collection.find(filter, find_opts).await?;

    if let Some(legacy_url) = cursor.try_next().await? {
        return Ok(Some(legacy_url.content_id))
    } else {
        return Ok(None)
    }
}
