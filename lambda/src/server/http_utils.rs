use crate::game::{hand_number_and_table, Bid, Chelem, CompletedHand, Game, Poignée, ValidationError};

fn lines(s: &str) -> Vec<String> {
    s.split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn form_value<'a>(form_data: &'a Vec<(String, String)>, key: &'a str) -> Option<&'a String> {
    form_data
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
}

fn bool_form_value(form_data: &Vec<(String, String)>, key: &str) -> bool {
    form_value(form_data, key).map(|s| {
        let s = s.to_lowercase();
        s == "on" || s == "true"
    }).unwrap_or(false)
}

fn form_values<'a>(form_data: &'a Vec<(String, String)>, key: &str) -> Vec<&'a String> {
    form_data
        .iter()
        .filter(|(k, _)| k.starts_with(key))
        .map(|(_, v)| v)
        .collect()
}

fn reqd_form_value<'a>(form_data: &'a Vec<(String, String)>, key: &'a str) -> Result<&'a String, ValidationError> {
    form_value(form_data, key).ok_or(ValidationError { msg: format!("Missing required field: {}", key) })
}

pub fn form_data_to_game(game_id: String, form_data: &Vec<(String, String)>) -> Result<Game, ValidationError> {
   Ok(Game {
        game_id,
        date: reqd_form_value(form_data, "date")?.clone(),
        host: reqd_form_value(form_data, "host")?.clone(),
        players: if let Some(players) = form_value(form_data, "players") {
            lines(&players)
        } else {
            vec![]
        },
        tables: if let Some(tables) = form_value(form_data, "tables") {
            lines(&tables)
        } else {
            vec![]
        }
    })
}

pub fn form_data_to_hand(form_data: &Vec<(String, String)>) -> Result<CompletedHand, ValidationError> {
    let hand_id = reqd_form_value(form_data, "handId")?;
    let (hand_number, table) = hand_number_and_table(hand_id)?;

    let bidder = reqd_form_value(form_data, "bidder")?.clone();
    let partner = match form_value(form_data, "partner") {
        None => None,
        Some(s) if s.is_empty() => None,
        Some(s) => Some(s.clone())
    };
    let defence: Vec<String> = form_values(form_data, "defence").iter().map(|s| (*s).clone()).collect();
    if defence.is_empty() {
        return Err(ValidationError { msg: "Missing required field: defence".to_string() });
    }

    // `players` is always inferred for a completed hand, regardless of what was supplied.
    let mut players = defence.clone();
    players.push(bidder.clone());
    if let Some(ref partner) = partner {
        players.push(partner.clone());
    }

    let bid = reqd_form_value(form_data, "bid")?.parse::<Bid>().unwrap();
    let won = bool_form_value(form_data, "won");
    let won_or_lost_by = reqd_form_value(form_data, "wonOrLostBy")?
        .parse::<i32>()
        .unwrap();
    let petit_au_bout = bool_form_value(form_data, "petitAuBout");
    let poignee = reqd_form_value(form_data, "poignee")?.parse::<Poignée>().unwrap();
    let chelem = reqd_form_value(form_data, "chelem")?.parse::<Chelem>().unwrap();

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