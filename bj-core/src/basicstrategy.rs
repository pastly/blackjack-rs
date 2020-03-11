use crate::resp::Resp;
use crate::table::Table;
use serde::{Deserialize, Serialize};

/// Blackjack table rules that affect basic strategy
///
/// Used as metadata for a Table<Resp> representing a basic strategy card.
///
/// Not all combinations of rules are necessarily common (thus don't expect to find a basic
/// strategy card for some random combination of these rules), but all should be technically
/// possible. E.g. A surrender-always-allowed, 3-deck, hit-17, no-double-after-split, and
/// no-peek-bj game probably doesn't exist, but it's a valid set of rules.
pub mod rules {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub enum Surrender {
        No,
        Yes,
        NotAce,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub enum NumDecks {
        One,
        Two,
        Three,
        FourPlus,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct Rules {
        pub decks: NumDecks,
        pub hit_soft_17: bool,
        pub double_after_split: bool,
        pub peek_bj: bool,
        pub surrender: Surrender,
    }
}

#[derive(Serialize, Deserialize)]
pub struct BasicStrategy {
    pub rules: rules::Rules,
    pub table: Table<Resp>,
}

impl BasicStrategy {
    pub fn new(rules: rules::Rules, table: Table<Resp>) -> Self {
        Self { rules, table }
    }
}

impl From<(rules::Rules, Table<Resp>)> for BasicStrategy {
    fn from((rules, table): (rules::Rules, Table<Resp>)) -> Self {
        Self::new(rules, table)
    }
}
