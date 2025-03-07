use std::{cmp::max, collections::HashMap};

use maud::{html, Markup, DOCTYPE};
use crate::{game::{hand_id, Bid, Chelem, CompletedHand, Game, Poignée}, server::routes::{url_for, Route}};

fn layout(content: Markup) -> Markup {
    let script_file = std::env::var("SCRIPT_JS").unwrap_or("script.js".to_string());
    let script_url = format!("/assets/{}", script_file);
    let style_file = std::env::var("STYLES_CSS").unwrap_or("styles.css".to_string());
    let style_url = format!("/assets/{}", style_file);

    html! {
        (DOCTYPE)
        html {
            head {
                title { "Tarot" }
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                link rel="stylesheet" href=(style_url);
                script src=(script_url) async {}
            }
            body {
                (content)
            }
        }
    }
}

fn player_select(
    players: &[String], 
    name: &str, 
    id: &str, 
    label_text: &str, 
    multiple: bool,
    required: bool,
    current_values: Vec<&String>
) -> Markup {
    html! {
        div class="form-group" {
            label for=(id) { (label_text) }
            select name=(name) id=(id) multiple[multiple] required[required] {
                @if current_values.is_empty() && required {
                    option value="" disabled selected hidden { "Selectionner" }
                }
                @if !required && !multiple {
                    option value="" { "Aucun" }
                }
                @for player in players {
                    option value=(player) selected[current_values.contains(&player)] { (player) }
                }
            }
        }
    }
}

pub fn html_index() -> Markup {
    layout(html! {
        h1 { "Tarot" }
        form action="/games" method="POST" {
            div class="form-group" {
                label for="date" { "Date" }
                input type="date" name="date" id="date" required;
            }
            div class="form-group" {
                label for="host" { "Chez" }
                input type="text" name="host" id="date" required;
            }
            div class="form-group" {
                label for="players" { "Players" }
                textarea name="players" rows="10" { }
            }
            div class="form-group" {
                label for="tables" { "Tables" }
                textarea name="tables" row="5" { }
            }
            button type="submit" { "Créer le jeu" }
        }
    })
}


fn select_options<T: PartialEq, ValFn, DisplayFn>(options: Vec<T>, current_value: Option<&T>, val_fn: ValFn, display_fn: DisplayFn) -> Markup
where
    ValFn: Fn(&T) -> String,
    DisplayFn: Fn(&T) -> String
{
    html! {
        @if current_value.is_none() {
            option value="" disabled selected hidden { "Selectionner" }
        }
        @for option in options {
            option value=(val_fn(&option)) selected[current_value.map(|val| *val == option).unwrap_or(false)] { (display_fn(&option)) };
        }
    }
}


pub fn hand_form(game: &Game, hand: Option<&CompletedHand>, next_hand_choices: Vec<(String, i32)>) -> Markup {
    let form_url = match hand {
        Some(hand) => format!("/games/{}/hands/{}", game.game_id, hand.hand_id()),
        None => format!("/games/{}/hands", game.game_id)
    };
    
    html! {
        form action=(form_url) method="POST" {
            div class="form-group" {
                label for="bid" { "Contrat" }
                select name="handId" id="handId" required {
                    @if let Some(ref h) = hand {
                        option value=(h.hand_id()) selected { "Table \"" (h.table) "\" - Partie #" (h.hand_number) }
                    }
                    @for (table, hand_number) in next_hand_choices {
                        option value=(hand_id(hand_number, &table)) { "Table \"" (table) "\" - Partie #" (hand_number) }
                    }
                }
            }

            div class="form-group" {
                label for="bid" { "Contrat" }
                select name="bid" id="bid" required {
                    (select_options(
                        vec![Bid::Petite, Bid::Garde, Bid::GardeSans, Bid::GardeContre],
                        hand.map(|h| &h.bid),
                        |v| v.to_string(),
                        |v| v.to_string()
                    ))
                }
            }

            (player_select(&game.players, "bidder", "bidder", "Preneur", false, true, hand.map(|h| vec![&h.bidder]).unwrap_or(vec![])))
            (player_select(&game.players, "partner", "partner", "Appelé", false, false, hand.map(|h| 
                if let Some(ref partner) = h.partner {
                    vec![partner]
                } else {
                    vec![]
                }
            ).unwrap_or(vec![])))
            (player_select(&game.players, "defence", "defence", "Defense", true, false, hand.map(|h| h.defence.iter().collect()).unwrap_or(vec![])))

            div class="checkbox-wrapper" {
                input type="checkbox" name="won" id="won" checked[hand.map(|h| h.won).unwrap_or(false)];
                label for="won" { "Gagnée?" }
            }

            div class="form-group" {
                label for="wonOrLostBy" { "Nombre de points gagnés/perdus" }
                input type="number" 
                      name="wonOrLostBy" 
                      id="wonOrLostBy"
                      min="0"
                      max="91"
                      value=(hand.map(|h| h.won_or_lost_by.to_string()).unwrap_or("".to_string()))
                      required;
            }

            div class="checkbox-wrapper" {
                input type="checkbox" name="petitAuBout" id="petitAuBout" checked[hand.map(|h| h.petit_au_bout).unwrap_or(false)];
                label for="petitAuBout" { "Petit au bout?" }
            }

            div class="form-group" {
                label for="poignee" { "Poignée" }
                select name="poignee" id="poignee" {
                    (select_options(
                        vec![Poignée::Aucune, Poignée::Simple, Poignée::Double, Poignée::Triple],
                        Some(hand.map(|h| &h.poignee).unwrap_or(&Poignée::Aucune)),
                        |v| v.to_string(),
                        |v| v.to_string()
                    ))
                }
            }

            div class="form-group" {
                label for="chelem" { "Chelem" }
                select name="chelem" id="chelem" {
                    (select_options(
                        vec![Chelem::Aucun, Chelem::Annoncé, Chelem::NonAnnoncé],
                        Some(hand.map(|h| &h.chelem).unwrap_or(&Chelem::Aucun)),
                        |v| v.to_string(),
                        |v| v.to_string()
                    ))
                }
            }

            button type="submit" { @if let Some(_) = hand { "Edit Hand" } @else { "Add Hand" } }
        }
    }
}

fn get_next_hand_choices(tables: &Vec<String>, hands: &Vec<&CompletedHand>) -> Vec<(String, i32)> {
    tables.iter().map(|table| {
        let mut hand_number = 1;
        for hand in hands {
            if hand.table == *table {
                hand_number = max(hand_number, hand.hand_number + 1);
            }
        }
        (table.clone(), hand_number)
    }).collect()
}


pub fn html_game(game: &Game, hands: &Vec<(CompletedHand, HashMap<String, i32>)>, totals: &HashMap<String, i32>) -> Markup {
    layout(html! {
        h1 { (game.date) ", chez " (game.host) }
        
        section {
            h2 { "Players" }
            @if !game.players.is_empty() {
                ul {
                    @for player in &game.players {
                        li { (player) }
                    }
                }
            }
        }

        @if !game.tables.is_empty() {
            section {
                h2 { "Tables" }
                ul {
                    @for table in &game.tables {
                        li { (table) }
                    }
                }
            }
        }

        section {
            h2 { "Hands" }
            @if !hands.is_empty() {
                table {
                    thead {
                        tr {
                            th { "Table" }
                            th { "Partie #" }
                            th { "Contrat" }
                            th { "Preneur" }
                            th { "Appelé" }
                            th { "Defense" }
                            th { "Gagnée?" }
                            th { "Points" }
                            th { "Petit au bout?" }
                            th { "Poignée" }
                            th { "Chelem" }
                            th { "" }
                            th { "" }
                        }
                    }
                    tbody {
                        @for (hand, _) in hands {
                            @let route_url = url_for(&Route::GameHand { game_id: game.game_id.clone(), hand_id: hand.hand_id() });
                            tr {
                                td { (hand.table) }
                                td { (hand.hand_number) }
                                td { (hand.bid) }
                                td { (hand.bidder) }
                                td {
                                    (match hand.partner.clone() {
                                        Some(p) => p,
                                        None => "-".to_string()
                                    })
                                }
                                td {
                                    @for player in &hand.defence {
                                        span { (player) }
                                    }
                                }
                                td { (if hand.won { "Oui" } else { "Non" }) }
                                td { (hand.won_or_lost_by) }
                                td { (if hand.petit_au_bout { "Oui" } else { "Non" }) }
                                td { (hand.poignee) }
                                td { (hand.chelem) }
                                td { a href=(route_url) { "Edit" } }
                                td { form action=(route_url) method="POST" {
                                    input type="hidden" name="_method" value="DELETE";
                                    button type="submit" { "Delete" }
                                } }
                            }
                        }
                    }
                }
            }
        }

        section {
            h2 { "Scores" }
            @if !hands.is_empty() {
                table {
                    thead {
                        tr {
                            th { "Table" }
                            th { "Partie #" }
                            @for player in &game.players {
                                th { (player) }
                            }
                        }
                    }
                    tbody {
                        @for (hand, scores) in hands {
                            tr {
                                td { (hand.table) }
                                td { (hand.hand_number) }
                                @for player in &game.players {
                                    th { (scores.get(player).map(|i| i.to_string()).unwrap_or("".to_string())) }
                                }
                            }
                        }

                        tr {
                            td colspan="2" { b { "Total" } }
                            @for player in &game.players {
                                th { (totals.get(player).map(|i| i.to_string()).unwrap_or("".to_string())) }
                            }
                        }
                    }
                }
            }
        }

        section {
            h2 { "Add Hand" }
            @let next_hand_choices = get_next_hand_choices(&game.tables, &hands.iter().map(|(h, _)| h).collect());
            (hand_form(&game, None, next_hand_choices))
        }
    })
}

pub fn html_edit_hand(game: &Game, hands: &Vec<CompletedHand>, hand: &CompletedHand) -> Markup {
    layout(html! {
        h1 { "Edit Hand" }
        (hand_form(game, Some(hand), get_next_hand_choices(&game.tables, &hands.iter().collect())))
    })
}

pub fn html_not_found() -> Markup {
    layout(html! {
        h1 { "404 - Not Found" }
        p { "The requested page could not be found." }
        a href="/" { "Return to Home" }
    })
}

pub fn html_bad_request(msg: &str) -> Markup {
    layout(html! {
        h1 { "Bad request" }
        p { (msg) }
        a href="/" { "Return to Home" }
    })
}