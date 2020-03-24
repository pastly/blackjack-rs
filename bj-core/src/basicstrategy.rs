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
    use std::convert::From;
    use std::fmt;
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub enum Surrender {
        No,
        Yes,
        NotAce,
    }

    impl fmt::Display for Surrender {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Self::No => "disallowed",
                    Self::Yes => "allowed",
                    Self::NotAce => "on dealer 2-10",
                }
            )
        }
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub enum NumDecks {
        One,
        Two,
        Three,
        FourPlus,
    }

    impl fmt::Display for NumDecks {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Self::One => "1",
                    Self::Two => "2",
                    Self::Three => "3",
                    Self::FourPlus => "4+",
                }
            )
        }
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct HitSoft17(bool);

    impl fmt::Display for HitSoft17 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                if self.0 {
                    "dealer hits"
                } else {
                    "dealer stands"
                }
            )
        }
    }

    impl From<bool> for HitSoft17 {
        fn from(val: bool) -> Self {
            HitSoft17(val)
        }
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct DAS(bool);

    impl fmt::Display for DAS {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", if self.0 { "allowed" } else { "disallowed" })
        }
    }

    impl From<bool> for DAS {
        fn from(val: bool) -> Self {
            DAS(val)
        }
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct PeekBJ(bool);

    impl fmt::Display for PeekBJ {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", if self.0 { "yes" } else { "no" })
        }
    }

    impl From<bool> for PeekBJ {
        fn from(val: bool) -> Self {
            PeekBJ(val)
        }
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct Rules {
        pub decks: NumDecks,
        pub hit_soft_17: HitSoft17,
        pub double_after_split: DAS,
        pub peek_bj: PeekBJ,
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
