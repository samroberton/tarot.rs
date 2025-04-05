use std::{cmp::max, collections::HashMap};

use http::Method;
use maud::{html, Markup, DOCTYPE};
use crate::{game::{hand_id, Bid, Chelem, CompletedHand, Game, Poignée}, server::routes::{url_for, Route}};

const ADD_ICON: &str = "/assets/add_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg";
const CLOSE_ICON: &str = "/assets/close_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg";
const DELETE_ICON: &str = "/assets/delete_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg";
const EDIT_ICON: &str = "/assets/edit_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg";
const GROUPS_ICON: &str = "/assets/groups_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg";
const PLAYING_CARDS_ICON: &str = "/assets/playing_cards_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg";
const QRCODE_ICON: &str = "/assets/qr_code_scanner_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg";
const SCOREBOARD_ICON: &str = "/assets/scoreboard_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg";

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

pub fn html_new_or_edit_game(game: Option<&Game>) -> Markup {
    layout(html! {
        h1 { "Tarot" }
        form action="/games" method="POST" {
            label for="date" { "Date" }
            input type="date" name="date" id="date" required value=(game.map(|g| g.date.clone()).unwrap_or("".to_string()));

            label for="host" { "Chez" }
            input type="text" name="host" id="date" required value=(game.map(|g| g.host.clone()).unwrap_or("".to_string()));

            label for="players" { "Players" }
            textarea name="players" rows="10" {
                @if let Some(g) = game {
                    (g.players.join("\n"))
                }
             }

            label for="tables" { "Tables" }
            textarea name="tables" row="5" {
                @if let Some(g) = game {
                    (g.tables.join("\n"))
                }
             }

            button type="submit" { 
                @if let Some(_) = game { "Modifier le jeu" } @else { "Créer le jeu" }
            }
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
        form .hand-form id="hand-form" action=(form_url) method="POST" {
            label for="bid" { "Partie" }
            select name="handId" id="handId" required {
                @if let Some(ref h) = hand {
                    option value=(h.hand_id()) selected { "Table \"" (h.table) "\" - Partie #" (h.hand_number) }
                }
                @for (table, hand_number) in next_hand_choices {
                    option value=(hand_id(hand_number, &table)) { "Table \"" (table) "\" - Partie #" (hand_number) }
                }
            }

            label for="bid" { "Contrat" }
            select name="bid" id="bid" required {
                (select_options(
                    vec![Bid::Petite, Bid::Garde, Bid::GardeSans, Bid::GardeContre],
                    hand.map(|h| &h.bid),
                    |v| v.to_string(),
                    |v| v.to_string()
                ))
            }

            (player_select(&game.players, "bidder", "bidder", "Preneur", false, true, hand.map(|h| vec![&h.bidder]).unwrap_or(vec![])))
            (player_select(&game.players, "partner", "partner", "Appelé", false, false, hand.map(|h| 
                if let Some(ref partner) = h.partner {
                    vec![partner]
                } else {
                    vec![]
                }
            ).unwrap_or(vec![])))
            (player_select(&game.players, "defence", "defence", "Defense", true, true, hand.map(|h| h.defence.iter().collect()).unwrap_or(vec![])))

            label for="won" { "Gagnée?" }
            input type="checkbox" name="won" id="won" checked[hand.map(|h| h.won).unwrap_or(false)];

            label for="wonOrLostBy" { "Nombre de points gagnés/perdus" }
            input type="number" 
                    name="wonOrLostBy" 
                    id="wonOrLostBy"
                    min="0"
                    step="1"
                    max="91"
                    value=(hand.map(|h| h.won_or_lost_by.to_string()).unwrap_or("".to_string()))
                    required;

            label for="petitAuBout" { "Petit au bout?" }
            input type="checkbox" name="petitAuBout" id="petitAuBout" checked[hand.map(|h| h.petit_au_bout).unwrap_or(false)];

            label for="poignee" { "Poignée" }
            select name="poignee" id="poignee" {
                (select_options(
                    vec![Poignée::Aucune, Poignée::Simple, Poignée::Double, Poignée::Triple],
                    Some(hand.map(|h| &h.poignee).unwrap_or(&Poignée::Aucune)),
                    |v| v.to_string(),
                    |v| v.to_string()
                ))
            }

            label for="chelem" { "Chelem" }
            select name="chelem" id="chelem" {
                (select_options(
                    vec![Chelem::Aucun, Chelem::Annoncé, Chelem::NonAnnoncé],
                    Some(hand.map(|h| &h.chelem).unwrap_or(&Chelem::Aucun)),
                    |v| v.to_string(),
                    |v| v.to_string()
                ))
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


pub fn html_game(game: &Game, hands: &Vec<(CompletedHand, HashMap<String, i32>)>, total_scores: &HashMap<String, i32>, player_hand_count: &HashMap<String, i32>) -> Markup {
    let qrcode_url = url_for(&Route::GameQRCode { game_id: game.game_id.clone() });

    layout(html! {
        h1 { (game.date) ", chez " (game.host) }

        dialog id="qrcode-dialog" {
            form autofocus method="dialog" {
                button .icon.close {
                    img src=(CLOSE_ICON) alt="Close" width="24" height="24";
                }
            }
            img .qrcode src=(qrcode_url) alt="QR Code";
        }
        
        button class="icon qrcode" onclick="document.getElementById('qrcode-dialog').showModal();" { 
            img src=(QRCODE_ICON) alt="QR Code" width="24" height="24";
         }
        
        nav {
            button onclick="toggleNavigableSection(event)" data-navigable="summary" {
                img src=(GROUPS_ICON) alt="Résumé" width="16" height="16";
                "Résumé"
            }
            button onclick="toggleNavigableSection(event)" data-navigable="hands" {
                img src=(PLAYING_CARDS_ICON) alt="Parties" width="16" height="16";
                "Parties"
            }
            button onclick="toggleNavigableSection(event)" data-navigable="scores" {
                img src=(SCOREBOARD_ICON) alt="Scores" width="16" height="16";
                "Scores"
            }
            button onclick="toggleNavigableSection(event)" data-navigable="add-hand" {
                img src=(ADD_ICON) alt="Ajoute une partie" width="16" height="16";
                "Ajouter"
            }
        }
        
        section data-navigable="summary" {
            h2 { "Joueurs" }
            @if !game.players.is_empty() {
                table .text-center {
                    thead {
                        tr {
                            th { "Joueur" }
                            th { "# parties" }
                        }
                    }
                    tbody {
                        @for player in &game.players {
                            tr { 
                                td { (player) }
                                td { (player_hand_count.get(player).map(|i| i.to_string()).unwrap_or("".to_string())) }
                            }
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
        }

        section data-navigable="hands" hidden {
            h2 { "Parties" }
            @if !hands.is_empty() {
                (hands_table(&game, hands))
            }
        }

        section data-navigable="scores" hidden {
            h2 { "Scores" }
            @if !hands.is_empty() {
                table .text-center {
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
                                th { (total_scores.get(player).map(|i| i.to_string()).unwrap_or("".to_string())) }
                            }
                        }
                    }
                }
            }
        }

        section data-navigable="add-hand" hidden {
            h2 { "Ajoute une partie" }
            @let next_hand_choices = get_next_hand_choices(&game.tables, &hands.iter().map(|(h, _)| h).collect());
            (hand_form(&game, None, next_hand_choices))
        }
    })
}

fn hands_table(game: &Game, hands: &Vec<(CompletedHand, HashMap<String, i32>)>) -> Markup {
    html! {
        table .hands.text-center {
            thead {
                tr {
                    th { "Partie" }
                    th { "Contrat" }
                    th { 
                        "Preneur"
                        br;
                        "avec Appelé"
                    }
                    th { "Défense" }
                    th { "Resultat" }
                }
            }
            tbody {
                @for (hand, _) in hands {
                    @let route_url = url_for(&Route::GameHand { game_id: game.game_id.clone(), hand_id: hand.hand_id() });
                    tr {
                        td {
                            div.cols {
                                span { (hand.table) ", #" (hand.hand_number) }
                                span {
                                    a .icon role="button" href=(route_url) { 
                                        img src=(EDIT_ICON) alt="Edit" width="16" height="16";
                                    }
                                    button .icon onclick=(format!("document.getElementById('delete-dialog-{}').showModal();", hand.hand_id())) { 
                                        img src=(DELETE_ICON) alt="Delete" width="16" height="16";
                                    }
                                    
                                    dialog id=(format!("delete-dialog-{}", hand.hand_id())) {
                                        p { "Are you sure you want to delete this hand?" }
                                        form action=(route_url) method="POST" {
                                            input type="hidden" name="_method" value="DELETE";
                                            button type="submit" { "Yes, delete" }
                                            button type="submit" formmethod="dialog" { "No, cancel" }
                                        }
                                    }
                                }
                            }
                        }
                        td { (hand.bid) }
                        td { 
                            (hand.bidder)
                            br;
                            (match hand.partner.clone() {
                                Some(p) => format!("avec {}", p),
                                None => "seul(e)".to_string()
                            })
                        }
                        td {
                            div .cols {
                                @for player in &hand.defence {
                                    span { (player) }
                                }
                            }
                        }
                        td {
                            div .cols {
                                span { (if hand.won { "gagnée" } else { "chutée" }) " de " (hand.won_or_lost_by) }
                                @if hand.petit_au_bout { span { "avec petit au bout" } }
                                @if hand.poignee != Poignée::Aucune { span { "avec une poignée " (hand.poignee) } }
                                @if hand.chelem != Chelem::Aucun { span { "avec un chelem " (hand.chelem) } }
                            }
                        }
                    }
                }
            }
        }
    }
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

pub fn html_game_not_found(game_id: &str) -> Markup {
    layout(html! {
        h1 { "Oops!" }
        h2 { "404 - Not Found" }
        p { (format!("The requested game ({}) could not be found.", game_id)) }
        a href="/" { "Return to Home" }
    })
}

pub fn html_hand_not_found(game_id: &str, hand_id: &str) -> Markup {
    layout(html! {
        h1 { "Oops!" }
        h2 { "404 - Not Found" }
        p { (format!("The requested hande (gamd_id = {}, hand_id = {}) could not be found.", game_id, hand_id)) }
        a href="/" { "Return to Home" }
    })
}

pub fn html_method_not_allowed(method: &Method, path: &str) -> Markup {
    layout(html! {
        h1 { "Oops!" }
        h2 { "Method Not Allowed" }
        pre { (method) " " (path) }
        p { "That's not a thing!"}
        a href="/" { "Return to Home" }
    })
}

pub fn html_validation_error(msg: &str) -> Markup {
    layout(html! {
        h1 { "Validation error" }
        p { (msg) }
        p { a href="" onclick="window.history.back(); return false;" { "Go back and try again." } }
    })
}