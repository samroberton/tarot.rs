mod scoring;
mod game;
mod db;
mod html;
mod server;

use http::Method;
use lambda_http::{run, service_fn, Body, Error as LambdaError, Request, Response as HttpResponse};

fn read_form_data(event: &Request) -> Result<Option<Vec<(String, String)>>, LambdaError> {
    match event.method() {
        &Method::POST | &Method::PUT | &Method::PATCH => {
            let is_form_data = event
                .headers()
                .get("content-type")
                .and_then(|ct| ct.to_str().ok())
                .map(|ct| ct.starts_with("application/x-www-form-urlencoded"))
                .unwrap_or(false);
        
            if !is_form_data {
                Ok(None)
            } else {
                match event.body() {
                    Body::Empty => Ok(Some(vec![])),
                    Body::Text(t) => Ok(Some(serde_urlencoded::from_str(&t)?)),
                    Body::Binary(b) => {
                        let s = String::from_utf8(b.to_vec())?;
                        Ok(Some(serde_urlencoded::from_str(&s)?))
                    }
                    
                }
            }
        },
        _ => Ok(None)
    }
}

async fn lambda_handler(event: Request) -> Result<HttpResponse<Body>, LambdaError> {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&config);

    let host = event.headers().get("host").unwrap().to_str().unwrap();
    let method = event.method();
    let path = event.uri().path();
    let form_data = read_form_data(&event)?;

    let response = server::handler::handle(&client, host, method, path, &form_data).await?;
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