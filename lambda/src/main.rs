mod db;
mod game;
mod html;
mod scoring;
mod server;

use http::Method;
use lambda_http::{run, service_fn, Body, Error as LambdaError, Request, Response as HttpResponse};

use aws_sdk_dynamodb::Client;
use aws_config::SdkConfig;
use tokio::sync::OnceCell;

static AWS_CONFIG: OnceCell<SdkConfig> = OnceCell::const_new();
static DYNAMODB_CLIENT: OnceCell<Client> = OnceCell::const_new();

fn read_form_data(event: &Request) -> Result<Option<Vec<(String, String)>>, LambdaError> {
    if !matches!(event.method(), &Method::POST | &Method::PUT | &Method::PATCH) {
        return Ok(None);
    }

    let is_form_data = match get_header(event, "content-type") {
        Some(ct) => ct.starts_with("application/x-www-form-urlencoded"),
        None => false,
    };

    if !is_form_data {
        return Ok(None);
    }

    match event.body() {
        Body::Empty => Ok(Some(vec![])),
        Body::Text(t) => Ok(Some(serde_urlencoded::from_str(t)?)),
        Body::Binary(b) => {
            let s = String::from_utf8(b.to_vec())?;
            Ok(Some(serde_urlencoded::from_str(&s)?))
        }
    }
}

fn get_header<'a>(event: &'a Request, header_name: &str) -> Option<&'a str> {
    event.headers()
        .get(header_name)?
        .to_str()
        .ok()
}

async fn lambda_handler(event: Request) -> Result<HttpResponse<Body>, LambdaError> {
    let config = AWS_CONFIG.get_or_init(|| async { aws_config::load_from_env().await }).await;
    let client = DYNAMODB_CLIENT.get_or_init(|| async { Client::new(config) }).await;

    let host = get_header(&event, "x-forwarded-host")
        .or_else(|| get_header(&event, "host"))
        .unwrap_or("localhost:3000");

    let response = server::handler::handle(
        client,
        host,
        event.method(),
        event.uri().path(),
        &read_form_data(&event)?,
    ).await?;

    Ok(server::responses::render(response)?)
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(lambda_handler)).await
}
