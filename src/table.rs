use crate::deck::Card;
use crate::hand::Hand;
use std::collections::HashMap;
use std::default::Default;
use std::fmt;
use std::io::{self, Read};

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Resp {
    Hit,
    Stand,
    Double,
    Split,
    //Surrender,
}

impl fmt::Display for Resp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hit => write!(f, "Hit"),
            Self::Stand => write!(f, "Stand"),
            Self::Double => write!(f, "Double"),
            Self::Split => write!(f, "Split"),
        }
    }
}

pub fn resp_from_char(c: char) -> Option<Resp> {
    match c {
        'H' => Some(Resp::Hit),
        'S' => Some(Resp::Stand),
        'D' => Some(Resp::Double),
        'P' => Some(Resp::Split),
        _ => None,
    }
}

/// Storage for the all ideal opening player responses
///
/// Table::new() takes something which impls Read. From this it ignores all comments ('#' until
/// newline) and then ignores all characters that are not a "response character." At the time of
/// writing, those are "HSDP" for hit, stand, double, and split respectively. This allows for
/// organization using whitespace and limited human readability.
///
/// The input Read-able buffer is logically split into three tables: the hard hands, soft hands,
/// and pairs. In all tables there are 10 columns (dealer shows 2, 3, 4, ... ace). The first (hard)
/// table has 17 rows (player hand value 5-21). The second (soft) table has 9 rows (player hand
/// value 13-21). The third (pairs) table has 10 rows (player hand pair of 2s, 3s, 4s ... 10s,
/// aces). This results in 370 total cells. Rows are "filled in" first. For a visual example, see
/// the Wizard of Odds website: https://wizardofodds.com/games/blackjack/strategy/calculator/
#[derive(Default)]
pub struct Table {
    hard: HashMap<(u8, u8), Resp>,
    soft: HashMap<(u8, u8), Resp>,
    pair: HashMap<(u8, u8), Resp>,
}

impl Table {
    pub fn new<R>(buf: R) -> io::Result<Self>
    where
        R: Read,
    {
        use crate::buffer::{CharWhitelistIter, CommentStripIter};
        const NUM_CELLS: usize = 10 * (17 + 9 + 10);
        let mut buf = CharWhitelistIter::new(CommentStripIter::new(buf), "HSDP");
        let mut s = String::with_capacity(NUM_CELLS);
        buf.read_to_string(&mut s)?;
        assert_eq!(s.len(), NUM_CELLS);
        let mut resps = s.chars().map(|c| resp_from_char(c).unwrap());
        let mut t = Self {
            ..Default::default()
        };
        // hard table
        for player_value in 5..=21 {
            for dealer_up in 2..=11 {
                let k = (player_value, dealer_up);
                let v = resps.next().unwrap();
                //eprintln!("{:?} {:?}", k, v);
                assert!(t.hard.insert(k, v).is_none());
            }
        }
        // soft table
        for player_value in 13..=21 {
            for dealer_up in 2..=11 {
                let k = (player_value, dealer_up);
                let v = resps.next().unwrap();
                //eprintln!("{:?} {:?}", k, v);
                assert!(t.soft.insert(k, v).is_none());
            }
        }
        // pair table
        // for the purpose of storage in the table, pair of aces is considered 22
        for player_value in &[4, 6, 8, 10, 12, 14, 16, 18, 20, 22] {
            for dealer_up in 2..=11 {
                let k = (*player_value, dealer_up);
                let v = resps.next().unwrap();
                //eprintln!("{:?} {:?}", k, v);
                assert!(t.pair.insert(k, v).is_none());
            }
        }
        assert!(resps.next().is_none());
        Ok(t)
    }

    /// Lookup and return the best response for the player, if it exists. The only valid reason for
    /// it to not exist is if the player has busted already, and in this None is returned. Else
    /// Some(Resp) is returned. A table lookup error is an indication of a programming error, not
    /// of an error/problem/etc. on the user's part, thus is handled with a panic instead of
    /// returning None.
    pub fn best_resp(&self, player_hand: &Hand, dealer_shows: Card) -> Option<Resp> {
        if player_hand.value() > 21 {
            return None;
        }
        assert!(player_hand.value() >= 2);
        let table = if player_hand.is_pair() {
            &self.pair
        } else if player_hand.is_soft() {
            &self.soft
        } else {
            &self.hard
        };
        let p = player_hand.value();
        let d = if dealer_shows.value() == 1 {
            11
        } else {
            dealer_shows.value()
        };
        let key = (p, d);
        if let Some(v) = table.get(&key) {
            Some(*v)
        } else {
            panic!(format!(
                "Unable to find best resp for hand {} with dealer {}. soft={} pair={}. key={:?}",
                player_hand,
                dealer_shows,
                player_hand.is_soft(),
                player_hand.is_pair(),
                key,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Table;
    use crate::deck::{Card, Suit, ALL_RANKS};
    use crate::hand::Hand;

    const T1: &str = "
# It's not important which tables these are, but for completeness,
# these are the tables for 4+ deck, dealer hit soft 17, double after split,
# no surrender, peek for BJ
# https://wizardofodds.com/games/blackjack/strategy/calculator/
# hard
HHHHHHHHHH
HHHHHHHHHH
HHHHHHHHHH
HHHHHHHHHH
HDDDDHHHHH
DDDDDDDDHH
DDDDDDDDDD
HHSSSHHHHH
SSSSSHHHHH
SSSSSHHHHH
SSSSSHHHHH
SSSSSHHHHH
SSSSSSSSSS
SSSSSSSSSS
SSSSSSSSSS
SSSSSSSSSS
SSSSSSSSSS
# soft
HHHDDHHHHH
HHHDDHHHHH
HHDDDHHHHH
HHDDDHHHHH
HDDDDHHHHH
DDDDDSSHHH
SSSSDSSSSS
SSSSSSSSSS
SSSSSSSSSS
# pair
PPPPPPHHHH
PPPPPPHHHH
HHHPPHHHHH
DDDDDDDDHH
PPPPPHHHHH
PPPPPPHHHH
PPPPPPPPPP
PPPPPSPPSS
SSSSSSSSSS
PPPPPPPPPP
    ";

    fn all_clubs() -> Vec<Card> {
        let mut v = vec![];
        for r in ALL_RANKS.iter() {
            for s in &[Suit::Club] {
                v.push(Card::new(*r, *s));
            }
        }
        v
    }

    fn all_club_pairs() -> Vec<Hand> {
        let mut hands = vec![];
        for c1 in all_clubs() {
            for c2 in all_clubs() {
                hands.push(Hand::new(&[c1, c2]));
            }
        }
        hands
    }

    fn all_club_trios() -> Vec<Hand> {
        let mut hands = vec![];
        for c1 in all_clubs() {
            for c2 in all_clubs() {
                for c3 in all_clubs() {
                    hands.push(Hand::new(&[c1, c2, c3]));
                }
            }
        }
        hands
    }

    #[test]
    fn new_asserts() {
        // Table::new() has its own asserts (right now ...). Let's exercise them here.
        let fd = T1.as_bytes();
        let _ = Table::new(fd).unwrap();
    }

    #[test]
    fn best_resp_1() {
        // all 2-card hands against all dealer show cards have a best response
        let t = Table::new(T1.as_bytes()).unwrap();
        for hand in all_club_pairs() {
            for dealer in all_clubs() {
                assert!(t.best_resp(&hand, dealer).is_some());
            }
        }
    }

    #[test]
    fn best_resp_2() {
        // 3-card hands should have a best response as long as they are worth 21 or less
        let t = Table::new(T1.as_bytes()).unwrap();
        for hand in all_club_trios() {
            for dealer in all_clubs() {
                assert_eq!(t.best_resp(&hand, dealer).is_some(), hand.value() <= 21);
            }
        }
    }
}
