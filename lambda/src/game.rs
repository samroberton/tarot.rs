use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub struct ValidationError(String);

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid value: {}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Bid {
    Petite,
    Garde,
    GardeSans,
    GardeContre,
}

impl fmt::Display for Bid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bid::Petite => write!(f, "petite"),
            Bid::Garde => write!(f, "garde"),
            Bid::GardeSans => write!(f, "garde sans"),
            Bid::GardeContre => write!(f, "garde contre"),
        }
    }
}

impl FromStr for Bid {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "petite" => Ok(Bid::Petite),
            "garde" => Ok(Bid::Garde),
            "garde sans" => Ok(Bid::GardeSans),
            "garde contre" => Ok(Bid::GardeContre),
            _ => Err(ValidationError(s.to_string())),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Poignée {
    Aucune,
    Simple,
    Double,
    Triple,
}

impl fmt::Display for Poignée {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Poignée::Aucune => write!(f, "aucune"),
            Poignée::Simple => write!(f, "simple"),
            Poignée::Double => write!(f, "double"),
            Poignée::Triple => write!(f, "triple"),
        }
    }
}

impl FromStr for Poignée {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "aucune" => Ok(Poignée::Aucune),
            "simple" => Ok(Poignée::Simple),
            "double" => Ok(Poignée::Double),
            "triple" => Ok(Poignée::Triple),
            _ => Err(ValidationError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Chelem {
    Aucun,
    NonAnnoncé,
    Annoncé,
}

impl fmt::Display for Chelem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Chelem::Aucun => write!(f, "aucun"),
            Chelem::NonAnnoncé => write!(f, "non annoncé"),
            Chelem::Annoncé => write!(f, "annoncé"),
        }
    }
}

impl FromStr for Chelem {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "aucun" => Ok(Chelem::Aucun),
            "non annoncé" => Ok(Chelem::NonAnnoncé),
            "annoncé" => Ok(Chelem::Annoncé),
            _ => Err(ValidationError(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct Game {
    pub game_id: String,
    pub date: String,
    pub host: String,
    pub players: Vec<String>,
    pub tables: Vec<String>,
}

#[derive(Debug)]
pub struct CompletedHand {
    pub table: String,
    pub hand_number: i32,
    pub players: Vec<String>,
    pub bid: Bid,
    pub bidder: String,
    pub partner: Option<String>,
    pub defence: Vec<String>,
    pub won: bool,
    pub won_or_lost_by: i32,
    pub petit_au_bout: bool,
    pub poignee: Poignée,
    pub chelem: Chelem,
}