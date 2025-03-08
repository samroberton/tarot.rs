use http::Method;
use lambda_http::Error as LambdaError;
use uuid::Uuid;

use crate::db::{delete_hand, get_game, get_hands, put_game, put_hand};
use crate::scoring;
use crate::server::http_utils;
use crate::server::responses::Response;
use crate::server::routes::{match_route, Route};

fn is_delete(form_data: &Vec<(String, String)>) -> bool {
    form_data.iter().any(|(k, _)| k == "_method" && form_data.iter().any(|(_, v)| v == "DELETE"))
}

pub async fn handle(
    client: &aws_sdk_dynamodb::Client,
    host: &str,
    method: &Method,
    path: &str,
    form_data: &Option<Vec<(String, String)>>,
) -> Result<Response, LambdaError> {
    let response = if let Some(route) = match_route(path) {
        match (method, route, form_data) {
            // GET /
            (&Method::GET, Route::Index, _) => Response::CreateGamePage,
            
            // POST /games
            (&Method::POST, Route::Games, Some(form_data)) => {
                match http_utils::form_data_to_game(Uuid::new_v4().to_string(), form_data) {
                    Ok(game) => {
                        put_game(client, &game).await?;
                        Response::RedirectToGame { game }
                    },
                    Err(e) => Response::ValidationError { msg: e.to_string() },
                }
            }

            // GET /games/{game_id}
            (&Method::GET, Route::Game { game_id }, _) => {
                if let Some(game) = get_game(client, &game_id).await? {
                    let hands = get_hands(client, &game_id).await?;
                    match scoring::score_hands(hands.clone()) {
                        Ok((hands_with_scores, total_scores, player_hand_count)) => {
                            Response::GamePage { game, hands_with_scores, total_scores, player_hand_count }
                        },
                        Err(err) => {
                            Response::ValidationError { msg: format!("Error scoring hands: {:?}", err) }
                        }
                    }
                } else {
                    Response::NotFound
                }
            }
            
            // GET /games/{game_id}/qrcode
            (&Method::GET, Route::GameQRCode { game_id }, _) => {
                if let Some(game) = get_game(client, &game_id).await? {
                    Response::QRCode { domain_name: host.to_string(), game_id: game.game_id }
                } else {
                    Response::GameNotFound { game_id }
                }
            }
            
            // POST /games/{game_id}
            (&Method::POST, Route::Game { game_id }, Some(form_data)) => {
                if let Some(game) = get_game(client, &game_id).await? {
                    match http_utils::form_data_to_game(game.game_id, form_data) {
                        Ok(game) => {
                            put_game(client, &game).await?;
                            Response::RedirectToGame { game }
                        },
                        Err(e) => Response::ValidationError { msg: e.to_string() }
                    }
                } else {
                    Response::GameNotFound { game_id }
                }
            }

            // POST /games/{game_id}/hands
            (&Method::POST, Route::GameHands { game_id }, Some(form_data)) => {
                if let Some(game) = get_game(client, &game_id).await? {
                    match http_utils::form_data_to_hand(form_data) {
                        Ok(hand) => {
                            put_hand(&client, &game_id, &hand).await?;
                            Response::RedirectToGame { game }
                        },
                        Err(e) => Response::ValidationError { msg: e.to_string() },
                    }
                } else {
                    Response::GameNotFound { game_id }
                }
            }

            // GET /games/{game_id}/hands/{hand_id}
            (&Method::GET, Route::GameHand { game_id, hand_id }, _) => {
                if let Some(game) = get_game(client, &game_id).await? {
                    let hands = get_hands(client, &game_id).await?;
                    let hand = hands.iter().find(|h| h.hand_id() == hand_id).cloned();
                    if let Some(hand) = hand {
                        Response::EditHandPage { game, hands: hands, hand }
                    } else {
                        Response::HandNotFound { game_id, hand_id }
                    }
                } else {
                    Response::GameNotFound { game_id }
                }
            }
            
            // quasi DELETE /games/{game_id}/hands/{hand_id}
            (&Method::POST, Route::GameHand { game_id, hand_id }, Some(form_data)) if is_delete(form_data) => {
                if let Some(game) = get_game(client, &game_id).await? {
                    delete_hand(client, &game_id, &hand_id).await?;
                    Response::RedirectToGame { game }
                } else {
                    Response::GameNotFound { game_id }
                }
            }

            // POST /games/{game_id}/hands/{hand_id}
            (&Method::POST, Route::GameHand { game_id, hand_id }, Some(form_data)) => {
                if let Some(game) = get_game(client, &game_id).await? {
                    match http_utils::form_data_to_hand(form_data) {
                        Ok(hand) => {
                            // if hand_id is being changed, and if so, delete the old hand
                            if hand.hand_id() != hand_id {
                                delete_hand(client, &game_id, &hand_id).await?;
                            }
                            put_hand(&client, &game_id, &hand).await?;
                            Response::RedirectToGame { game }
                        },
                        Err(e) => Response::ValidationError { msg: e.to_string() },
                    }
                } else {
                    Response::GameNotFound { game_id }
                }
            }

            // 405
            _ => Response::HttpMethodNotAllowed { method: method.clone(), path: path.to_string() },
        }
    } else {
        Response::NotFound
    };
    Ok(response)
}
