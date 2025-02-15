use std::collections::HashMap;

use http::{Error as HttpError, Method, Response as HttpResponse};
use image::{ImageEncoder, Luma};
use lambda_http::Body;
use maud::Markup;
use qrcode::QrCode;

use crate::game::{CompletedHand, Game};
use crate::html::{html_bad_request, html_game, html_index, html_not_found};
use crate::server::routes::{url_for, Route};

pub enum Response {
    CreateGamePage,
    RedirectToGame(Game),
    GamePage(
        Game,
        Vec<(CompletedHand, HashMap<String, i32>)>,
        HashMap<String, i32>,
    ),
    GameNotFound(String),
    NotFound,
    HttpMethodNotAllowed(Method, String),
    ValidationError(String),
    QRCode(String),
}

pub fn render(response: Response) -> Result<HttpResponse<Body>, HttpError> {
    match response {
        Response::CreateGamePage => http200(html_index()),
        Response::RedirectToGame(game) => http302(url_for(&Route::Game(game.game_id))),
        Response::GamePage(game, hands, totals) => http200(html_game(&game, &hands, &totals)),
        Response::NotFound => http404(html_not_found()),
        Response::GameNotFound(_game_id) => {
            // TODO specific response for game not found
            http404(html_not_found())
        }
        Response::HttpMethodNotAllowed(_method, _path) => {
            // TODO specific response for method not allowed
            http405(html_not_found())
        }
        Response::ValidationError(msg) => {
            // TODO specific response for validation error
            http400(html_bad_request(&msg))
        }
        Response::QRCode(url) => {
            let code = QrCode::new(url.as_bytes()).unwrap();
            let image = code
                .render::<Luma<u8>>()
                .quiet_zone(true)
                .module_dimensions(6, 6)
                .build();
            
            let mut png_bytes = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
            encoder.write_image(
                image.as_raw(),
                image.width(),
                image.height(),
                image::ExtendedColorType::L8
            ).unwrap();

            HttpResponse::builder()
                .status(200)
                .header("Content-Type", "image/png")
                .body(png_bytes.into())
        }
    }
}

fn http200(html: Markup) -> Result<HttpResponse<Body>, HttpError> {
    HttpResponse::builder()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(html.into_string().into())
}

fn http302(location: String) -> Result<HttpResponse<Body>, HttpError> {
    HttpResponse::builder()
        .status(302)
        .header("Location", location)
        .body(Body::Empty)
}

fn http400(html: Markup) -> Result<HttpResponse<Body>, HttpError> {
    HttpResponse::builder()
        .status(400)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(html.into_string().into())
}

fn http404(html: Markup) -> Result<HttpResponse<Body>, HttpError> {
    HttpResponse::builder()
        .status(404)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(html.into_string().into())
}

fn http405(html: Markup) -> Result<HttpResponse<Body>, HttpError> {
    HttpResponse::builder()
        .status(405)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(html.into_string().into())
}
