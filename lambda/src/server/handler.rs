use http::Method;
use lambda_http::Error as LambdaError;

use crate::db::{get_game, get_hands, put_game, put_hand};
use crate::scoring;
use crate::server::http_utils;
use crate::server::responses::Response;
use crate::server::routes::{match_route, Route};

pub async fn handle(
    client: &aws_sdk_dynamodb::Client,
    host: &str,
    method: &Method,
    path: &str,
    form_data: &Option<Vec<(String, String)>>,
) -> Result<Response, LambdaError> {
    let response = if let Some(route) = match_route(path) {
        match (method, route, form_data) {
            (&Method::GET, Route::Index, _) => Response::CreateGamePage,
            (&Method::POST, Route::Games, Some(form_data)) => {
                let game = http_utils::new_game(form_data);
                put_game(client, &game).await?;
                Response::RedirectToGame(game)
            }
            (&Method::GET, Route::Game(game_id), _) => {
                if let Some(game) = get_game(client, &game_id).await? {
                    let hands = get_hands(client, &game_id).await?;
                    match scoring::score_hands(hands) {
                        Ok((hands_with_scores, total_scores)) =>
                            Response::GamePage(game, hands_with_scores, total_scores),
                        Err(err) => {
                            // TODO error!
                            print!("Error scoring hands: {:?}", err);
                            Response::ValidationError(format!("Error scoring hands: {:?}", err))
                        }
                    }
                } else {
                    Response::NotFound
                }
            }
            (&Method::GET, Route::GameQRCode(game_id), _) => {
                if let Some(game) = get_game(client, &game_id).await? {
                    Response::QRCode(format!("https://{}/games/{}/qrcode", host, game.game_id))
                } else {
                    Response::GameNotFound(game_id)
                }
            }
            (&Method::POST, Route::Game(game_id), Some(form_data)) => {
                if let Some(mut game) = get_game(client, &game_id).await? {
                    game = http_utils::edit_game(game, form_data);
                    put_game(&client, &game).await?;
                    Response::RedirectToGame(game)
                } else {
                    Response::GameNotFound(game_id)
                }
            }
            (&Method::POST, Route::GameHands(game_id), Some(form_data)) => {
                if let Some(game) = get_game(client, &game_id).await? {
                    match http_utils::new_hand(form_data) {
                        Ok(hand) => {
                            put_hand(&client, &game_id, &hand).await?;
                            Response::RedirectToGame(game)
                        },
                        Err(e) => Response::ValidationError(e.to_string()),
                    }
                } else {
                    Response::GameNotFound(game_id)
                }
            }
            _ => Response::HttpMethodNotAllowed(method.clone(), path.to_string()),
        }
    } else {
        Response::NotFound
    };
    Ok(response)
}
