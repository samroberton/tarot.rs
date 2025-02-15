use std::collections::HashMap;

use crate::game::{Bid, Chelem, CompletedHand, Poignée};

impl Bid {
    pub fn multiplier(&self) -> i32 {
        match self {
            Bid::Petite => 1,
            Bid::Garde => 2,
            Bid::GardeSans => 4,
            Bid::GardeContre => 6,
        }
    }
}

impl Poignée {
    pub fn score(&self) -> i32 {
        match self {
            Poignée::Aucune => 0,
            Poignée::Simple => 20,
            Poignée::Double => 30,
            Poignée::Triple => 40,
        }
    }
}

impl Chelem {
    pub fn score(&self) -> i32 {
        match self {
            Chelem::Aucun => 0,
            Chelem::NonAnnoncé => 200,
            Chelem::Annoncé => 400,
        }
    }
}

pub fn score(hand: &CompletedHand) -> Result<HashMap<String, i32>, String> {
    let base_score = (25 + hand.won_or_lost_by + if hand.petit_au_bout { 10 } else { 0 }) * hand.bid.multiplier()
        + hand.poignee.score()
        + hand.chelem.score();

    let mut scores = HashMap::new();
    match (hand.players.len(), hand.defence.len(), &hand.partner) {
        (5, 4, _) => {
            // 5 players, bidder called themselves
            for player in &hand.defence {
                scores.insert(player.clone(), if hand.won { -base_score } else { base_score });
            }
            scores.insert(hand.bidder.clone(), 4 * if hand.won { base_score } else { -base_score });
        },
        (5, _, Some(partner)) => {
            // 5 players, bidder and partner are different players
            for player in &hand.defence {
                scores.insert(player.clone(), if hand.won { -base_score } else { base_score });
            }
            scores.insert(hand.bidder.clone(), 2 * if hand.won { base_score } else { -base_score });
            scores.insert(partner.clone(), if hand.won { base_score } else { -base_score });
        },
        (4, _, _) => {
            // 4 players
            for player in &hand.defence {
                scores.insert(player.clone(), if hand.won { -base_score } else { base_score });
            }
            scores.insert(hand.bidder.clone(), 3 * if hand.won { base_score } else { -base_score });
        },
        _ => return Err(format!("Invalid hand configuration: {:?}", hand)),
    }

    // Verify scores sum to 0
    let sum: i32 = scores.values().sum();
    if sum != 0 {
        return Err(format!("Scores do not sum to 0: {}", sum));
    }

    Ok(scores)
}

pub fn score_hands(hands: Vec<CompletedHand>) -> Result<(Vec<(CompletedHand, HashMap<String, i32>)>, HashMap<String, i32>), String> {
    let mut hands_with_scores = vec![];
    let mut total_scores = HashMap::new();

    for hand in hands {
        match score(&hand) {
            Ok(scores) => {
                hands_with_scores.push((hand, scores.clone()));
                for (player, score) in scores {
                    *total_scores.entry(player).or_insert(0) += score;
                }
            },
            Err(e) => return Err(format!("Error scoring hand: {}", e))
        }
    }

    Ok((hands_with_scores, total_scores))
}

// let total_scores: HashMap<String, i32> = HashMap::new();
//                     let hands_with_scores = hands
//                         .into_iter()
//                         .map(|hand| {
//                             let score = scoring::score(&hand)?;
//                             Ok((hand, score))
//                         })
//                         .collect();

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_players() -> Vec<String> {
        vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
            "David".to_string(),
        ]
    }

    fn create_test_players_five() -> Vec<String> {
        let mut players = create_test_players();
        players.push("Eve".to_string());
        players
    }

    #[test]
    fn test_four_player_hand_won() {
        let players = create_test_players();
        let hand = CompletedHand {
            table: "Atout".to_string(),
            hand_number: 1,
            players: players.clone(),
            bid: Bid::Petite,
            bidder: "Alice".to_string(),
            partner: None,
            defence: vec!["Bob".to_string(), "Charlie".to_string(), "David".to_string()],
            won: true,
            won_or_lost_by: 10,
            petit_au_bout: false,
            poignee: Poignée::Aucune,
            chelem: Chelem::Aucun,
        };

        let scores = score(&hand).unwrap();
        
        // Base score should be (25 + 10) * 1 = 35
        assert_eq!(scores.get("Alice").unwrap(), &105); // 3 * 35
        assert_eq!(scores.get("Bob").unwrap(), &-35);
        assert_eq!(scores.get("Charlie").unwrap(), &-35);
        assert_eq!(scores.get("David").unwrap(), &-35);
    }

    #[test]
    fn test_five_player_hand_with_partner_won() {
        let players = create_test_players_five();
        let hand = CompletedHand {
            table: "Atout".to_string(),
            hand_number: 1,
            players,
            bid: Bid::Garde,
            bidder: "Alice".to_string(),
            partner: Some("Bob".to_string()),
            defence: vec!["Charlie".to_string(), "David".to_string(), "Eve".to_string()],
            won: true,
            won_or_lost_by: 15,
            petit_au_bout: true,
            poignee: Poignée::Simple,
            chelem: Chelem::Aucun,
        };

        let scores = score(&hand).unwrap();
        
        // Base score should be (25 + 15 + 10) * 2 + 20 = 120
        assert_eq!(scores.get("Alice").unwrap(), &240); // 2 * 120
        assert_eq!(scores.get("Bob").unwrap(), &120);   // 1 * 120
        assert_eq!(scores.get("Charlie").unwrap(), &-120);
        assert_eq!(scores.get("David").unwrap(), &-120);
        assert_eq!(scores.get("Eve").unwrap(), &-120);
    }

    #[test]
    fn test_five_player_hand_bidder_alone_lost() {
        let players = create_test_players_five();
        let hand = CompletedHand {
            table: "Atout".to_string(),
            hand_number: 1,
            players,
            bid: Bid::GardeSans,
            bidder: "Alice".to_string(),
            partner: None,
            defence: vec!["Bob".to_string(), "Charlie".to_string(), "David".to_string(), "Eve".to_string()],
            won: false,
            won_or_lost_by: 20,
            petit_au_bout: false,
            poignee: Poignée::Aucune,
            chelem: Chelem::Aucun,
        };

        let scores = score(&hand).unwrap();
        
        // Base score should be (25 + 20) * 4 = 180
        assert_eq!(scores.get("Alice").unwrap(), &-720); // 4 * -180
        assert_eq!(scores.get("Bob").unwrap(), &180);
        assert_eq!(scores.get("Charlie").unwrap(), &180);
        assert_eq!(scores.get("David").unwrap(), &180);
        assert_eq!(scores.get("Eve").unwrap(), &180);
    }

    #[test]
    fn test_chelem_and_poignee_scoring() {
        let players = create_test_players();
        let hand = CompletedHand {
            table: "Atout".to_string(),
            hand_number: 1,
            players: players.clone(),
            bid: Bid::GardeContre,
            bidder: "Alice".to_string(),
            partner: None,
            defence: vec!["Bob".to_string(), "Charlie".to_string(), "David".to_string()],
            won: true,
            won_or_lost_by: 30,
            petit_au_bout: true,
            poignee: Poignée::Double,
            chelem: Chelem::Annoncé,
        };

        let scores = score(&hand).unwrap();
        
        // Base score should be (25 + 30 + 10) * 6 + 30 + 400 = 820
        assert_eq!(scores.get("Alice").unwrap(), &2460); // 3 * 820
        assert_eq!(scores.get("Bob").unwrap(), &-820);
        assert_eq!(scores.get("Charlie").unwrap(), &-820);
        assert_eq!(scores.get("David").unwrap(), &-820);
    }

    #[test]
    fn test_invalid_hand_configuration() {
        let players = vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()]; // Only 3 players
        let hand = CompletedHand {
            table: "Atout".to_string(),
            hand_number: 1,
            players,
            bid: Bid::Petite,
            bidder: "Alice".to_string(),
            partner: None,
            defence: vec!["Bob".to_string(), "Charlie".to_string()],
            won: true,
            won_or_lost_by: 10,
            petit_au_bout: false,
            poignee: Poignée::Aucune,
            chelem: Chelem::Aucun,
        };

        assert!(score(&hand).is_err());
    }
}