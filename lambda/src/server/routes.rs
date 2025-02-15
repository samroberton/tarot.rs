use lazy_static::lazy_static;
use regex::Regex;

// Route patterns
lazy_static! {
    static ref ROUTE_GAME: Regex = Regex::new(r"^/games/([^/]+)$").unwrap();
    static ref ROUTE_GAME_QRCODE: Regex = Regex::new(r"^/games/([^/]+)/qrcode$").unwrap();
    static ref ROUTE_HANDS: Regex = Regex::new(r"^/games/([^/]+)/hands$").unwrap();
}

fn match_route_pattern<'a>(pattern: &Regex, path: &'a str) -> Option<Vec<&'a str>> {
    pattern.captures(path).map(|caps| {
        caps.iter().skip(1).map(|m| m.unwrap().as_str()).collect()
    })
}

pub enum Route {
    Index,
    Games,
    Game(String),
    GameQRCode(String),
    GameHands(String),
}

pub fn match_route(path: &str) -> Option<Route> {
    if path == "/" || path == "" {
        Some(Route::Index)
    } else if path == "/games" {
        Some(Route::Games)
    } else if let Some(caps) = match_route_pattern(&ROUTE_GAME, path) {
        Some(Route::Game(caps[0].to_string()))
    } else if let Some(caps) = match_route_pattern(&ROUTE_GAME_QRCODE, path) {
        Some(Route::GameQRCode(caps[0].to_string()))
    } else if let Some(caps) = match_route_pattern(&ROUTE_HANDS, path) {
        Some(Route::GameHands(caps[0].to_string()))
    } else {
        None
    }
}

pub fn url_for(route: &Route) -> String {
    match route {
        Route::Index => "/".to_string(),
        Route::Games => "/games".to_string(),
        Route::Game(game_id) => format!("/games/{}", game_id),
        Route::GameQRCode(game_id) => format!("/games/{}/qrcode", game_id),
        Route::GameHands(game_id) => format!("/games/{}/hands", game_id)
    }
}