use crate::deck::Card;
use crate::hand::Hand;
use std::collections::HashMap;
use std::fmt;
use std::io::Read;

const NUM_CELLS: usize = 10 * (17 + 9 + 10);

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

/// Take something that impls Read, strip out comments ('#' until end of line), ignore everything
/// that isn't in "HSDP" (hit, stand, double, split), and return a vector of these parsed
/// Vec<Resp>.
pub fn resps_from_buf<R>(buf: R) -> Vec<Resp>
where
    R: Read,
{
    use crate::buffer::{CharWhitelistIter, CommentStripIter};
    let mut buf = CharWhitelistIter::new(CommentStripIter::new(buf), "HSDP");
    let mut s = String::with_capacity(NUM_CELLS);
    buf.read_to_string(&mut s).unwrap();
    // safe to unwrap as CharWhitelistIter will remove non-Resp chars
    s.chars().map(|c| resp_from_char(c).unwrap()).collect()
}

/// Convert the given char into a Resp, or None if impossible
pub fn resp_from_char(c: char) -> Option<Resp> {
    match c {
        'H' => Some(Resp::Hit),
        'S' => Some(Resp::Stand),
        'D' => Some(Resp::Double),
        'P' => Some(Resp::Split),
        _ => None,
    }
}

#[derive(PartialEq, Debug)]
pub enum TableError {
    NotFilled,
    AlreadyFilled,
    IncorrectFillLength(usize, usize),
    HandIsBust(Hand, Card),
}

impl std::error::Error for TableError {}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableError::NotFilled => write!(f, "Table not filled in yet"),
            TableError::AlreadyFilled => write!(f, "Table has already been filled"),
            TableError::IncorrectFillLength(expect, given) => write!(
                f,
                "Table needs {} elements but was given {}{}",
                expect,
                given,
                if given > expect { " or more" } else { "" }
            ),
            TableError::HandIsBust(hand, card) => write!(
                f,
                "Cannot find item at {}/{} because hand is bust",
                hand, card
            ),
        }
    }
}

/// Store something in each cell of a blackjack strategy card. E.g. the best move to make.
///
/// Table::new() returns an empty table that then must be filled with Table::fill().
///
/// Table contains three logical subtables: the hard hands, soft hands, and pairs.  In all
/// subtables there are 10 columns (dealer shows 2, 3, 4, ... ace). The first (hard) table has 17
/// rows (player hand value 5-21). The second (soft) table has 9 rows (player hand value 13-21).
/// The third (pairs) table has 10 rows (player hand pair of 2s, 3s, 4s ... 10s, aces). This
/// results in 370 total cells. Table::fill() takes an iterable and fills in the subtables
/// left-to-right and top-to-bottom one right after another.  For a visual example of what a Table
/// looks like (e.g. if used to store the best move for a player to make), see the blackjack
/// strategy cards on the Wizard of Odds website:
/// https://wizardofodds.com/games/blackjack/strategy/calculator/.
pub struct Table<T>
where
    // might not all be necessary
    T: PartialEq + Copy + Clone,
{
    hard: HashMap<(u8, u8), T>,
    soft: HashMap<(u8, u8), T>,
    pair: HashMap<(u8, u8), T>,
    is_filled: bool,
}

impl<T> Table<T>
where
    T: PartialEq + Copy + Clone,
{
    pub fn new() -> Self {
        Self {
            hard: HashMap::new(),
            soft: HashMap::new(),
            pair: HashMap::new(),
            is_filled: false,
        }
    }

    /// Fill the Table's subtables from the given iterable.
    ///
    /// The iterable must be exactly 370 items in length, else return an error.
    /// The table must not have been filled already, else return an error.
    ///
    /// See Table's documentation for more information.
    pub fn fill<I>(&mut self, vals: I) -> Result<(), TableError>
    where
        I: IntoIterator<Item = T>,
    {
        let mut cell_idx = 0;
        if self.is_filled {
            return Err(TableError::AlreadyFilled);
        }
        let mut vals = vals.into_iter();
        // hard table
        for player_value in 5..=21 {
            for dealer_up in 2..=11 {
                let k = (player_value, dealer_up);
                if let Some(v) = vals.next() {
                    //eprintln!("{:?} {:?}", k, v);
                    assert!(self.hard.insert(k, v).is_none());
                    cell_idx += 1;
                } else {
                    return Err(TableError::IncorrectFillLength(NUM_CELLS, cell_idx));
                }
            }
        }
        // soft table
        for player_value in 13..=21 {
            for dealer_up in 2..=11 {
                let k = (player_value, dealer_up);
                if let Some(v) = vals.next() {
                    //eprintln!("{:?} {:?}", k, v);
                    assert!(self.soft.insert(k, v).is_none());
                    cell_idx += 1;
                } else {
                    return Err(TableError::IncorrectFillLength(NUM_CELLS, cell_idx));
                }
            }
        }
        // pair table
        // for the purpose of storage in the table, pair of aces is considered 22
        for player_value in &[4, 6, 8, 10, 12, 14, 16, 18, 20, 22] {
            for dealer_up in 2..=11 {
                let k = (*player_value, dealer_up);
                if let Some(v) = vals.next() {
                    //eprintln!("{:?} {:?}", k, v);
                    assert!(self.pair.insert(k, v).is_none());
                    cell_idx += 1;
                } else {
                    return Err(TableError::IncorrectFillLength(NUM_CELLS, cell_idx));
                }
            }
        }
        if vals.next().is_some() {
            return Err(TableError::IncorrectFillLength(NUM_CELLS, cell_idx + 1));
        }
        assert_eq!(NUM_CELLS, cell_idx);
        self.is_filled = true;
        Ok(())
    }

    /// Lookup and return the value stored at the given location in the table, if it exists.
    /// The table must already be filled, else an error is returned. If the player's hand is bust,
    /// then lookup would fail and an error is returned. There is no other reason for lookup to
    /// fail, so if it does, that indicates a programming error and we panic.
    pub fn get(&self, player_hand: &Hand, dealer_shows: Card) -> Result<T, TableError> {
        if !self.is_filled {
            return Err(TableError::NotFilled);
        } else if player_hand.value() > 21 {
            return Err(TableError::HandIsBust(player_hand.clone(), dealer_shows));
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
            Ok(*v)
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
    use super::{resps_from_buf, Resp, Table, TableError, NUM_CELLS};
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
    fn fill_empty() {
        // Should fail to fill table with empty iter
        let mut t = Table::<()>::new();
        assert_eq!(
            t.fill(vec![]).unwrap_err(),
            TableError::IncorrectFillLength(NUM_CELLS, 0)
        );
    }

    #[test]
    fn fill_short() {
        // Should fail to fill with too few items
        for count in 1..NUM_CELLS {
            let mut t = Table::<()>::new();
            assert_eq!(
                t.fill(vec![(); count]).unwrap_err(),
                TableError::IncorrectFillLength(NUM_CELLS, count)
            );
        }
    }

    #[test]
    fn fill_long() {
        // Should fail to fill with too few items
        for count in NUM_CELLS + 1..NUM_CELLS + 10 {
            let mut t = Table::<()>::new();
            eprintln!("{}", count);
            assert_eq!(
                t.fill(vec![(); count]).unwrap_err(),
                TableError::IncorrectFillLength(NUM_CELLS, NUM_CELLS + 1)
            );
        }
    }

    #[test]
    fn fill() {
        // Should succeed with exactly correct number of items
        let mut t = Table::<()>::new();
        assert!(t.fill(vec![(); NUM_CELLS]).is_ok());
    }

    #[test]
    fn fill_twice() {
        // cannot fill twice
        let mut t = Table::<()>::new();
        assert!(t.fill(vec![(); NUM_CELLS]).is_ok());
        assert_eq!(
            t.fill(vec![(); NUM_CELLS]).unwrap_err(),
            TableError::AlreadyFilled
        );
    }

    #[test]
    fn fill_responses() {
        // with our known-good strategy, try filling and make sure no error
        let mut t: Table<Resp> = Table::new();
        assert_eq!(t.fill(resps_from_buf(T1.as_bytes())), Ok(()));
    }

    #[test]
    fn get_1() {
        // all 2-card hands against all dealer show cards have a best response
        let mut t: Table<Resp> = Table::new();
        assert_eq!(t.fill(resps_from_buf(T1.as_bytes())), Ok(()));
        for hand in all_club_pairs() {
            for dealer in all_clubs() {
                assert!(t.get(&hand, dealer).is_ok());
            }
        }
    }

    #[test]
    fn get_2() {
        // 3-card hands should have a best response as long as they are worth 21 or less
        let mut t: Table<Resp> = Table::new();
        assert_eq!(t.fill(resps_from_buf(T1.as_bytes())), Ok(()));
        for hand in all_club_trios() {
            for dealer in all_clubs() {
                assert_eq!(t.get(&hand, dealer).is_ok(), hand.value() <= 21);
            }
        }
    }
}
