use std::collections::HashMap;

use maud::{html, Markup, DOCTYPE};
use crate::game::{CompletedHand, Game};

const STYLES: &str = r#"
    body {
        max-width: 600px;
        margin: 0 auto;
        font-family: system-ui, -apple-system, sans-serif;
        line-height: 1.5;
        padding: 2rem;
    }
    
    form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
    
    label {
        display: block;
        margin-bottom: 0.5rem;
    }
    
    input, select {
        padding: 0.5rem;
        border: 1px solid #ccc;
        border-radius: 4px;
        font-size: 1rem;
        width: 100%;
        box-sizing: border-box;
    }
    
    select[multiple] {
        height: 8rem;
    }
    
    button {
        padding: 0.75rem 1.5rem;
        background-color: #0066cc;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 1rem;
    }
    
    button:hover {
        background-color: #0052a3;
    }
    
    .checkbox-wrapper {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }
    
    .checkbox-wrapper input[type="checkbox"] {
        width: 1.2rem;
        height: 1.2rem;
    }

    .form-group {
        margin-bottom: 1rem;
    }
"#;

fn layout(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                title { "Tarot" }
                style { (STYLES) }
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
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
    required: bool
) -> Markup {
    html! {
        div class="form-group" {
            label for=(id) { (label_text) }
            select name=(name) id=(id) multiple[multiple] required[required] {
                option value="" disabled selected hidden { "Selectionner" }
                @if !required && !multiple {
                    option value="" { "Aucun" }
                }
                @for player in players {
                    option value=(player) { (player) }
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
                        }
                    }
                    tbody {
                        @for (hand, _) in hands {
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
            form action={"/games/" (game.game_id) "/hands"} method="POST" {
                div class="form-group" {
                    label for="table" { "Table" }
                    select name="table" id="table" required {
                        @for table in &game.tables {
                            option value=(table) { (table) }
                        }
                    }
                }
                
                div class="form-group" {
                    label for="handNumber" { "Partie #" }
                    input type="number" 
                          name="handNumber" 
                          id="handNumber"
                          value="1" 
                          min="1" 
                          required;
                }

                div class="form-group" {
                    label for="bid" { "Contrat" }
                    select name="bid" id="bid" required {
                        option value="" disabled selected hidden { "Selectionner" }
                        option value="petite" { "Petite" }
                        option value="garde" { "Garde" }
                        option value="garde sans" { "Garde Sans" }
                        option value="garde contre" { "Garde Contre" }
                    }
                }

                (player_select(&game.players, "bidder", "bidder", "Preneur", false, true))
                (player_select(&game.players, "partner", "partner", "Appelé", false, false))
                (player_select(&game.players, "defence", "defence", "Defense", true, false))

                div class="checkbox-wrapper" {
                    input type="checkbox" name="won" id="won";
                    label for="won" { "Gagnée?" }
                }

                div class="form-group" {
                    label for="wonOrLostBy" { "Nombre de points gagnés/perdus" }
                    input type="number" 
                          name="wonOrLostBy" 
                          id="wonOrLostBy"
                          min="0"
                          required;
                }

                div class="checkbox-wrapper" {
                    input type="checkbox" name="petitAuBout" id="petitAuBout";
                    label for="petitAuBout" { "Petit au bout?" }
                }

                div class="form-group" {
                    label for="poignee" { "Poignée" }
                    select name="poignee" id="poignee" {
                        option value="aucune" { "Aucune" }
                        option value="simple" { "Simple" }
                        option value="double" { "Double" }
                        option value="triple" { "Triple" }
                    }
                }

                div class="form-group" {
                    label for="chelem" { "Chelem" }
                    select name="chelem" id="chelem" {
                        option value="aucun" { "Aucun" }
                        option value="annoncé" { "Annoncé" }
                        option value="non annoncé" { "Non annoncé" }
                    }
                }

                button type="submit" { "Add Hand" }
            }
        }
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