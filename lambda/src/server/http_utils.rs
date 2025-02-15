use uuid::Uuid;

use crate::game::{Bid, Chelem, CompletedHand, Game, Poignée, ValidationError};

fn lines(s: &str) -> Vec<String> {
    s.split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn get_value(form_data: &Vec<(String, String)>, key: &str) -> String {
    form_data
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}

fn get_vec_values(form_data: &Vec<(String, String)>, key: &str) -> Vec<String> {
    form_data
        .iter()
        .filter(|(k, _)| k.starts_with(key))
        .map(|(_, v)| v.clone())
        .collect()
}

pub fn new_game(form_data: &Vec<(String, String)>) -> Game {
   let game = Game {
        game_id: Uuid::new_v4().to_string(),
        date: "".to_string(),
        host: "".to_string(),
        players: vec![],
        tables: vec![],
    };
    edit_game(game, form_data)
}

pub fn find(form_data: &Vec<(String, String)>, param: &str) -> Option<String> {
    form_data
        .iter()
        .find(|(key, _)| key == param)
        .map(|(_, value)| value.clone())
}

pub fn edit_game(mut game: Game, form_data: &Vec<(String, String)>) -> Game {
    game.date = find(form_data, "date").unwrap();
    game.host = find(form_data, "host").unwrap();
    if let Some(players) = find(form_data, "players") {
        game.players = lines(&players);
    } else {
        game.players = vec![];
    }
    if let Some(tables) = find(form_data, "tables") {
        game.tables = lines(&tables);
    } else {
        game.players = vec![];
    }
    game
}

pub fn new_hand(form_data: &Vec<(String, String)>) -> Result<CompletedHand, ValidationError> {
    let table = get_value(form_data, "table");
    let bidder = get_value(form_data, "bidder");
    let partner = if get_value(form_data, "partner").is_empty() {
        None
    } else {
        Some(get_value(form_data, "partner"))
    };

    // Get numeric values
    let hand_number = get_value(form_data, "handNumber")
        .parse::<i32>()
        .unwrap();
    let won_or_lost_by = get_value(form_data, "wonOrLostBy")
        .parse::<i32>()
        .unwrap();

    // Get boolean values
    let won = get_value(form_data, "won").to_lowercase() == "on";
    let petit_au_bout = get_value(form_data, "petitAuBout").to_lowercase() == "on";

    // Get vectors
    let players = get_vec_values(form_data, "players");
    let defence = get_vec_values(form_data, "defence");

    // Get enums (assuming you have From<String> implementations or similar)
    let bid = get_value(form_data, "bid").parse::<Bid>().unwrap();
    let poignee = get_value(form_data, "poignee").parse::<Poignée>().unwrap();
    let chelem = get_value(form_data, "chelem").parse::<Chelem>().unwrap();

    Ok(CompletedHand {
        table,
        hand_number,
        players,
        bid,
        bidder,
        partner,
        defence,
        won,
        won_or_lost_by,
        petit_au_bout,
        poignee,
        chelem,
    })
}