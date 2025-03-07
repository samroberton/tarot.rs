use std::collections::HashMap;

use http::{Error as HttpError, Method, Response as HttpResponse};
use image::{ImageEncoder, Luma};
use lambda_http::Body;
use maud::Markup;
use qrcode::QrCode;

use crate::game::{CompletedHand, Game};
use crate::html::{html_bad_request, html_edit_hand, html_game, html_index, html_not_found};
use crate::server::routes::{url_for, Route};

#[derive(Debug)]
pub enum Response {
    CreateGamePage,
    RedirectToGame { game: Game },
    GamePage {
        game: Game,
        hands_with_scores: Vec<(CompletedHand, HashMap<String, i32>)>,
        total_scores: HashMap<String, i32>
    },
    GameNotFound { game_id: String },
    EditHandPage { game: Game, hands: Vec<CompletedHand>, hand: CompletedHand },
    HandNotFound { game_id: String, hand_id: String },
    NotFound,
    HttpMethodNotAllowed { method: Method, path: String },
    ValidationError { msg: String },
    QRCode { url: String },
}

pub fn render(response: Response) -> Result<HttpResponse<Body>, HttpError> {
    match response {
        Response::CreateGamePage => http200(html_index()),
        Response::RedirectToGame { game } => http302(url_for(&Route::Game { game_id: game.game_id })),
        Response::GamePage { game, hands_with_scores, total_scores } => {
            http200(html_game(&game, &hands_with_scores, &total_scores))
        },
        Response::GameNotFound { game_id: _ } => {
            // TODO specific response for game not found
            http404(html_not_found())
        },
        Response::EditHandPage { game, hands, hand } => {
            http200(html_edit_hand(&game, &hands, &hand))
        },
        Response::HandNotFound { game_id: _, hand_id: _ } => {
            // TODO specific response for hand not found
            http404(html_not_found())
        },
        Response::NotFound => http404(html_not_found()),
        Response::HttpMethodNotAllowed { method: _, path: _ } => {
            // TODO specific response for method not allowed
            http405(html_not_found())
        }
        Response::ValidationError { msg } => {
            // TODO specific response for validation error
            http400(html_bad_request(&msg))
        }
        Response::QRCode { url } => {
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
