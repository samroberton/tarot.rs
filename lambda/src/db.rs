use std::str::FromStr;
use lazy_static::lazy_static;

use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::{
    get_item::GetItemError, put_item::PutItemError, query::QueryError,
};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use thiserror::Error;

use crate::game::{Bid, Chelem, CompletedHand, Game, Poignée};

lazy_static! {
    static ref APP_NAME: String = std::env::var("APP_NAME").unwrap_or("tarot".to_string());
    static ref TABLE_GAMES: String = format!("{}-games", *APP_NAME);
    static ref TABLE_HANDS: String = format!("{}-hands", *APP_NAME);
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Query error: {0}")]
    Query(#[from] SdkError<QueryError>),

    #[error("PutItem error: {0}")]
    PutItem(#[from] SdkError<PutItemError>),

    #[error("GetItem error: {0}")]
    GetItem(#[from] SdkError<GetItemError>),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub async fn get_game(client: &Client, game_id: &str) -> Result<Option<Game>, DbError> {
    let result = client
        .get_item()
        .table_name((*TABLE_GAMES).clone())
        .key("gameId", AttributeValue::S(game_id.to_string()))
        .send()
        .await?;

    if let Some(item) = result.item {
        let game = Game {
            game_id: get_s(&item, "gameId")?,
            date: get_s(&item, "date")?,
            host: get_s(&item, "host")?,
            players: get_l_of_s(&item, "players")?,
            tables: get_l_of_s(&item, "tables")?
        };
        Ok(Some(game))
    } else {
        Ok(None)
    }
}

pub async fn put_game(client: &Client, game: &Game) -> Result<(), DbError> {
    client
        .put_item()
        .table_name((*TABLE_GAMES).clone())
        .item("gameId", to_s(&game.game_id))
        .item("date", to_s(&game.date))
        .item("host", to_s(&game.host))
        .item("players", to_l_of_s(&game.players))
        .item("tables", to_l_of_s(&game.tables))
        .send()
        .await?;
    Ok(())
}

pub async fn get_hands(client: &Client, game_id: &str) -> Result<Vec<CompletedHand>, DbError> {
    let result = client
        .query()
        .table_name((*TABLE_HANDS).clone())
        .key_condition_expression("gameId = :gameId")
        .expression_attribute_values(":gameId", AttributeValue::S(game_id.to_string()))
        .scan_index_forward(true)
        .send()
        .await?;

    let items = match result.items {
        Some(items) => items,
        None => return Ok(vec![]),
    };

    let hands: Result<Vec<CompletedHand>, _> = items
        .into_iter()
        .map(|item| {
            Ok(CompletedHand {
                table: get_s(&item, "table")?,
                hand_number: get_n(&item, "handNumber")?,
                players: get_l_of_s(&item, "players")?,
                bid: Bid::from_str(
                    get_s(&item, "bid")?.as_str(),
                )
                .map_err(|e| DbError::Validation(format!("Invalid bid {:?}", e.to_string())))?,
                bidder: get_s(&item, "bidder")?,
                partner: get_option_s(&item, "partner")?,
                defence: get_l_of_s(&item, "defence")?,
                won: get_bool(&item, "won")?,
                won_or_lost_by: item
                    .get("wonOrLostBy")
                    .and_then(|av| av.as_n().ok())
                    .and_then(|n| n.parse().ok())
                    .ok_or_else(|| {
                        DbError::Validation("Missing or invalid wonOrLostBy".to_string())
                    })?,
                petit_au_bout: get_bool(&item, "petitAuBout")?,
                poignee: Poignée::from_str(
                    item.get("poignee")
                        .and_then(|av| av.as_s().ok())
                        .ok_or_else(|| DbError::Validation("Missing poignee".to_string()))?,
                )
                .unwrap(),
                chelem: Chelem::from_str(
                    item.get("chelem")
                        .and_then(|av| av.as_s().ok())
                        .ok_or_else(|| DbError::Validation("Missing chelem".to_string()))?,
                )
                .unwrap(),
            })
        })
        .collect();

    hands
}

pub async fn put_hand(client: &Client, game_id: &str, hand: &CompletedHand) -> Result<(), DbError> {
    client
        .put_item()
        .table_name((*TABLE_HANDS).clone())
        .item("gameId", to_s(&game_id.to_string()))
        .item(
            "handId",
            AttributeValue::S(format!("{}#{}", &hand.hand_number, hand.table)),
        )
        .item("table", to_s(&hand.table))
        .item("handNumber", to_n(hand.hand_number))
        .item("players", to_l_of_s(&hand.players))
        .item("bid", AttributeValue::S(hand.bid.to_string()))
        .item("bidder", to_s(&hand.bidder))
        .item(
            "partner",
            match &hand.partner {
                Some(p) => to_s(p),
                None => AttributeValue::Null(true),
            },
        )
        .item("defence", to_l_of_s(&hand.defence))
        .item("won", AttributeValue::Bool(hand.won))
        .item("wonOrLostBy", to_n(hand.won_or_lost_by))
        .item("petitAuBout", AttributeValue::Bool(hand.petit_au_bout))
        .item("poignee", AttributeValue::S(hand.poignee.to_string()))
        .item("chelem", AttributeValue::S(hand.chelem.to_string()))
        .send()
        .await?;

    Ok(())
}

fn get_option_s(
    item: &std::collections::HashMap<String, AttributeValue>,
    key: &str,
) -> Result<Option<String>, DbError> {
    match item.get(key) {
        None => Ok(None),
        Some(attr_val) => match attr_val {
            AttributeValue::Null(_b) => Ok(None),
            AttributeValue::S(s) => Ok(Some(s.clone())),
            v => Err(DbError::Validation(format!(
                "Attribute {:?} is not a string: {:?}",
                key, v
            ))),
        }
    }
}

fn get_s(
    item: &std::collections::HashMap<String, AttributeValue>,
    key: &str,
) -> Result<String, DbError> {
    match item.get(key) {
        None => Err(DbError::Validation(format!(
            "Missing attribute {:?} in: {:?}",
            key, item
        ))),
        Some(attr_val) => match attr_val {
            AttributeValue::S(s) => Ok(s.clone()),
            v => Err(DbError::Validation(format!(
                "Attribute {:?} is not a string: {:?}",
                key, v
            ))),
        }
    }
}

fn get_n(
    item: &std::collections::HashMap<String, AttributeValue>,
    key: &str,
) -> Result<i32, DbError> {
    match item.get(key) {
        None => Err(DbError::Validation(format!(
            "Missing attribute {:?} in: {:?}",
            key, item
        ))),
        Some(attr_val) => match attr_val {
            AttributeValue::N(s) => s.parse().map_err(|e| {
                DbError::Validation(format!("Can't parse attribute {:?} as a number: {:?}", key, e))
            }),
            v => Err(DbError::Validation(format!(
                "Attribute {:?} is not a number: {:?}",
                key, v
            ))),
        }
    }
}

fn get_bool(
    item: &std::collections::HashMap<String, AttributeValue>,
    key: &str,
) -> Result<bool, DbError> {
    match item.get(key) {
        None => Err(DbError::Validation(format!(
            "Missing attribute {:?} in: {:?}",
            key, item
        ))),
        Some(attr_val) => match attr_val {
            AttributeValue::Bool(b) => Ok(b.to_owned()),
            v => Err(DbError::Validation(format!(
                "Attribute {:?} is not a boolean: {:?}",
                key, v
            ))),
        }
    }
}

fn get_l_of_s(
    item: &std::collections::HashMap<String, AttributeValue>,
    key: &str,
) -> Result<Vec<String>, DbError> {
    match item.get(key) {
        None => Err(DbError::Validation(format!(
            "Missing attribute {:?} in: {:?}",
            key, item
        ))),
        Some(attr_val) => match attr_val {
            AttributeValue::L(l) => l
                .iter()
                .map(|v| match v {
                    AttributeValue::S(s) => Ok(s.clone()),
                    v => Err(DbError::Validation(format!(
                        "Item in attribute {:?} is not a string: {:?}",
                        key, v
                    )))
                })
                .collect(),
            v => Err(DbError::Validation(format!(
                "Attribute {:?} is not a list: {:?}",
                key, v
            )))
        }
    }
}

// item
//     .get("tables")
//     .unwrap()
//     .as_l()
//     .unwrap()
//     .iter()
//     .map(|v| v.as_s().unwrap().clone())
//     .collect(),



fn to_s(s: &String) -> AttributeValue {
    AttributeValue::S(s.clone())
}

fn to_n(n: i32) -> AttributeValue {
    AttributeValue::N(n.to_string())
}

fn to_l_of_s(v: &Vec<String>) -> AttributeValue {
    AttributeValue::L(v.iter().map(|s| AttributeValue::S(s.clone())).collect())
}