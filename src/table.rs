use crate::deck::{Card, Rank, Suit, ALL_RANKS, ALL_SUITS};
use crate::hand::Hand;
use std::collections::HashMap;
use std::default::Default;
use std::io::{self, Read};

#[derive(Debug, Copy, Clone)]
pub enum Resp {
    Hit,
    Stand,
    Double,
    Split,
    //Surrender,
}

fn resp_from_char(c: char) -> Resp {
    match c {
        'H' => Resp::Hit,
        'S' => Resp::Stand,
        'D' => Resp::Double,
        'P' => Resp::Split,
        _ => unreachable!("Calling code should have prevented unknown chars from reaching here"),
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
    fn new<R>(buf: R) -> io::Result<Self>
    where
        R: Read,
    {
        use crate::buffer::{CharWhitelistIter, CommentStripIter};
        const NUM_CELLS: usize = 10 * (17 + 9 + 10);
        let mut buf = CharWhitelistIter::new(CommentStripIter::new(buf), "HSDP");
        let mut s = String::with_capacity(NUM_CELLS);
        buf.read_to_string(&mut s)?;
        assert_eq!(s.len(), NUM_CELLS);
        let mut resps = s.chars().map(resp_from_char);
        let mut t = Self {
            ..Default::default()
        };
        // hard table
        for player_value in 5..=21 {
            for dealer_up in 2..=11 {
                let k = (player_value, dealer_up);
                let v = resps.next().unwrap();
                //eprintln!("{:?} {:?}", k, v);
                t.hard.insert(k, v);
            }
        }
        // soft table
        for player_value in 13..=21 {
            for dealer_up in 2..=11 {
                let k = (player_value, dealer_up);
                let v = resps.next().unwrap();
                //eprintln!("{:?} {:?}", k, v);
                t.hard.insert(k, v);
            }
        }
        // pair table
        // for the purpose of storage in the table, pair of aces is considered 22
        for player_value in &[4, 6, 8, 10, 12, 14, 16, 18, 20, 22] {
            for dealer_up in 2..=11 {
                let k = (*player_value, dealer_up);
                let v = resps.next().unwrap();
                eprintln!("{:?} {:?}", k, v);
                t.pair.insert(k, v);
            }
        }
        assert!(resps.next().is_none());
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use super::Table;
    use crate::deck::{Card, Rank, Suit};
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

    #[test]
    fn new_asserts() {
        // Table::new() has its own asserts (right now ...). Let's exercise them here.
        let fd = T1.as_bytes();
        let _ = Table::new(fd).unwrap();
    }
}
