use crate::deck::{Card, Rank};
use crate::hand::Hand;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::io::Read;

const HARD_CELLS: usize = 17 * 10;
const SOFT_CELLS: usize = 9 * 10;
const PAIR_CELLS: usize = 10 * 10;
const NUM_CELLS: usize = HARD_CELLS + SOFT_CELLS + PAIR_CELLS;
#[rustfmt::skip]
const HARD_KEYS: [(u8, u8); HARD_CELLS] = [
    (5, 2), (5, 3), (5, 4), (5, 5), (5, 6), (5, 7), (5, 8), (5, 9), (5, 10), (5, 11),
    (6, 2), (6, 3), (6, 4), (6, 5), (6, 6), (6, 7), (6, 8), (6, 9), (6, 10), (6, 11),
    (7, 2), (7, 3), (7, 4), (7, 5), (7, 6), (7, 7), (7, 8), (7, 9), (7, 10), (7, 11),
    (8, 2), (8, 3), (8, 4), (8, 5), (8, 6), (8, 7), (8, 8), (8, 9), (8, 10), (8, 11),
    (9, 2), (9, 3), (9, 4), (9, 5), (9, 6), (9, 7), (9, 8), (9, 9), (9, 10), (9, 11),
    (10, 2), (10, 3), (10, 4), (10, 5), (10, 6), (10, 7), (10, 8), (10, 9), (10, 10), (10, 11),
    (11, 2), (11, 3), (11, 4), (11, 5), (11, 6), (11, 7), (11, 8), (11, 9), (11, 10), (11, 11),
    (12, 2), (12, 3), (12, 4), (12, 5), (12, 6), (12, 7), (12, 8), (12, 9), (12, 10), (12, 11),
    (13, 2), (13, 3), (13, 4), (13, 5), (13, 6), (13, 7), (13, 8), (13, 9), (13, 10), (13, 11),
    (14, 2), (14, 3), (14, 4), (14, 5), (14, 6), (14, 7), (14, 8), (14, 9), (14, 10), (14, 11),
    (15, 2), (15, 3), (15, 4), (15, 5), (15, 6), (15, 7), (15, 8), (15, 9), (15, 10), (15, 11),
    (16, 2), (16, 3), (16, 4), (16, 5), (16, 6), (16, 7), (16, 8), (16, 9), (16, 10), (16, 11),
    (17, 2), (17, 3), (17, 4), (17, 5), (17, 6), (17, 7), (17, 8), (17, 9), (17, 10), (17, 11),
    (18, 2), (18, 3), (18, 4), (18, 5), (18, 6), (18, 7), (18, 8), (18, 9), (18, 10), (18, 11),
    (19, 2), (19, 3), (19, 4), (19, 5), (19, 6), (19, 7), (19, 8), (19, 9), (19, 10), (19, 11),
    (20, 2), (20, 3), (20, 4), (20, 5), (20, 6), (20, 7), (20, 8), (20, 9), (20, 10), (20, 11),
    (21, 2), (21, 3), (21, 4), (21, 5), (21, 6), (21, 7), (21, 8), (21, 9), (21, 10), (21, 11),
];
#[rustfmt::skip]
const SOFT_KEYS: [(u8, u8); SOFT_CELLS] = [
    (13, 2), (13, 3), (13, 4), (13, 5), (13, 6), (13, 7), (13, 8), (13, 9), (13, 10), (13, 11),
    (14, 2), (14, 3), (14, 4), (14, 5), (14, 6), (14, 7), (14, 8), (14, 9), (14, 10), (14, 11),
    (15, 2), (15, 3), (15, 4), (15, 5), (15, 6), (15, 7), (15, 8), (15, 9), (15, 10), (15, 11),
    (16, 2), (16, 3), (16, 4), (16, 5), (16, 6), (16, 7), (16, 8), (16, 9), (16, 10), (16, 11),
    (17, 2), (17, 3), (17, 4), (17, 5), (17, 6), (17, 7), (17, 8), (17, 9), (17, 10), (17, 11),
    (18, 2), (18, 3), (18, 4), (18, 5), (18, 6), (18, 7), (18, 8), (18, 9), (18, 10), (18, 11),
    (19, 2), (19, 3), (19, 4), (19, 5), (19, 6), (19, 7), (19, 8), (19, 9), (19, 10), (19, 11),
    (20, 2), (20, 3), (20, 4), (20, 5), (20, 6), (20, 7), (20, 8), (20, 9), (20, 10), (20, 11),
    (21, 2), (21, 3), (21, 4), (21, 5), (21, 6), (21, 7), (21, 8), (21, 9), (21, 10), (21, 11),
];
#[rustfmt::skip]
const PAIR_KEYS: [(u8, u8); PAIR_CELLS] = [
    (4, 2), (4, 3), (4, 4), (4, 5), (4, 6), (4, 7), (4, 8), (4, 9), (4, 10), (4, 11),
    (6, 2), (6, 3), (6, 4), (6, 5), (6, 6), (6, 7), (6, 8), (6, 9), (6, 10), (6, 11),
    (8, 2), (8, 3), (8, 4), (8, 5), (8, 6), (8, 7), (8, 8), (8, 9), (8, 10), (8, 11),
    (10, 2), (10, 3), (10, 4), (10, 5), (10, 6), (10, 7), (10, 8), (10, 9), (10, 10), (10, 11),
    (12, 2), (12, 3), (12, 4), (12, 5), (12, 6), (12, 7), (12, 8), (12, 9), (12, 10), (12, 11),
    (14, 2), (14, 3), (14, 4), (14, 5), (14, 6), (14, 7), (14, 8), (14, 9), (14, 10), (14, 11),
    (16, 2), (16, 3), (16, 4), (16, 5), (16, 6), (16, 7), (16, 8), (16, 9), (16, 10), (16, 11),
    (18, 2), (18, 3), (18, 4), (18, 5), (18, 6), (18, 7), (18, 8), (18, 9), (18, 10), (18, 11),
    (20, 2), (20, 3), (20, 4), (20, 5), (20, 6), (20, 7), (20, 8), (20, 9), (20, 10), (20, 11),
    (22, 2), (22, 3), (22, 4), (22, 5), (22, 6), (22, 7), (22, 8), (22, 9), (22, 10), (22, 11),
];

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
    MissingKeys(String),
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
            TableError::MissingKeys(s) => write!(
                f,
                "Table missing keys.{}",
                if s.is_empty() {
                    String::new()
                } else {
                    String::from(" ") + s
                }
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
#[derive(Debug, PartialEq)]
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

    fn get_subtable(&self, player_hand: &Hand) -> &HashMap<(u8, u8), T> {
        if player_hand.is_pair() {
            &self.pair
        } else if player_hand.is_soft() {
            &self.soft
        } else {
            &self.hard
        }
    }

    fn get_subtable_mut(&mut self, player_hand: &Hand) -> &mut HashMap<(u8, u8), T> {
        if player_hand.is_pair() {
            &mut self.pair
        } else if player_hand.is_soft() {
            &mut self.soft
        } else {
            &mut self.hard
        }
    }

    fn key(player_hand: &Hand, dealer_shows: Card) -> (u8, u8) {
        let p = if player_hand.is_pair() && player_hand.cards[0].rank == Rank::RA {
            // player having a pair of aces is a special case. Hand::value() returns 12, which
            // causes a lookup in the pair take for a pair of 6s. Thus aces are stored with keys
            // with player hand value 22.
            22
        } else {
            player_hand.value()
        };
        let d = if dealer_shows.value() == 1 {
            11
        } else {
            dealer_shows.value()
        };
        (p, d)
    }

    /// Lookup and return the value stored at the given location in the table, if it exists.
    /// The table must already be filled, else an error is returned. If the player's hand is bust,
    /// then lookup would fail and an error is returned. If a lookup fails because the calcuated
    /// key is missing (which should be impossible, but ... ya know ...), then return an error.
    pub fn get(&self, player_hand: &Hand, dealer_shows: Card) -> Result<T, TableError> {
        if !self.is_filled {
            return Err(TableError::NotFilled);
        } else if player_hand.value() > 21 {
            return Err(TableError::HandIsBust(player_hand.clone(), dealer_shows));
        }
        assert!(player_hand.value() >= 2);
        let table = self.get_subtable(player_hand);
        let key = Table::<T>::key(player_hand, dealer_shows);
        if let Some(v) = table.get(&key) {
            Ok(*v)
        } else {
            Err(TableError::MissingKeys(format!(
                "Unable to find value for hand {} with dealer {}. soft={} pair={}. key={:?}",
                player_hand,
                dealer_shows,
                player_hand.is_soft(),
                player_hand.is_pair(),
                key,
            )))
        }
    }

    /// Update the given (player_hand, dealer_shows) key to have a new value and return the old
    /// value.
    ///
    /// If the table has not been filled, return an error. If the player's hand is bust, then
    /// lookup would fail and an error is returned. Even after successfully inserting the new
    /// value, if there was no original value, will return Err(TableError::MissingKeys)
    pub fn update(
        &mut self,
        player_hand: &Hand,
        dealer_shows: Card,
        val: T,
    ) -> Result<T, TableError> {
        if !self.is_filled {
            return Err(TableError::NotFilled);
        } else if player_hand.value() > 21 {
            return Err(TableError::HandIsBust(player_hand.clone(), dealer_shows));
        }
        let table = self.get_subtable_mut(player_hand);
        let key = Table::<T>::key(player_hand, dealer_shows);
        match table.insert(key, val) {
            Some(old) => Ok(old),
            None => Err(TableError::MissingKeys(format!(
                "There was no old value at hand {} with dealer {}. soft={} pair={} key={:?}",
                player_hand,
                dealer_shows,
                player_hand.is_soft(),
                player_hand.is_pair(),
                key
            ))),
        }
    }

    /// An internal-only sanity check for help during final deserialization
    ///
    /// Checks if the given table has all the correct keys in its subtables
    #[rustfmt::skip]
    fn has_all_keys(&self) -> bool {
        if self.hard.len() != HARD_CELLS
            || self.soft.len() != SOFT_CELLS
            || self.pair.len() != PAIR_CELLS
        {
            false
        } else if HARD_KEYS.iter().filter(|k| !self.hard.contains_key(k)).count() > 0
            || SOFT_KEYS.iter().filter(|k| !self.soft.contains_key(k)).count() > 0
            || PAIR_KEYS.iter().filter(|k| !self.pair.contains_key(k)).count() > 0
        {
            false
        } else {
            true
        }
    }

    /// An internal-only constructor for help during final deserialization
    ///
    /// Takes arrays for the hard, soft, and pair subtables, checks they are the correct length,
    /// assumes they have all the right keys in their key/value pairs, builds the Table, and sets
    /// its is_filled field to true.
    fn from_raw_parts(
        hard: Vec<((u8, u8), T)>,
        soft: Vec<((u8, u8), T)>,
        pair: Vec<((u8, u8), T)>,
    ) -> Result<Self, TableError> {
        assert_eq!(hard.len(), HARD_CELLS);
        assert_eq!(soft.len(), SOFT_CELLS);
        assert_eq!(pair.len(), PAIR_CELLS);
        let hard = {
            let mut d = HashMap::<(u8, u8), T>::new();
            for kv in hard.into_iter() {
                d.insert(kv.0, kv.1);
            }
            d
        };
        let soft = {
            let mut d = HashMap::<(u8, u8), T>::new();
            for kv in soft.into_iter() {
                d.insert(kv.0, kv.1);
            }
            d
        };
        let pair = {
            let mut d = HashMap::<(u8, u8), T>::new();
            for kv in pair.into_iter() {
                d.insert(kv.0, kv.1);
            }
            d
        };
        let t = Self {
            hard,
            soft,
            pair,
            is_filled: true,
        };
        if t.has_all_keys() {
            Ok(t)
        } else {
            Err(TableError::MissingKeys(
                "Cannot contruct from raw parts".to_owned(),
            ))
        }
    }
}

impl<T> Serialize for Table<T>
where
    T: PartialEq + Copy + Clone + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = self.hard.len() + self.soft.len() + self.pair.len();
        let mut seq = serializer.serialize_seq(Some(len))?;
        for e in self.hard.iter() {
            seq.serialize_element(&e)?;
        }
        for e in self.soft.iter() {
            seq.serialize_element(&e)?;
        }
        for e in self.pair.iter() {
            seq.serialize_element(&e)?;
        }
        seq.end()
    }
}

impl<'de, T> Deserialize<'de> for Table<T>
where
    T: PartialEq + Copy + Clone + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut hard: Vec<((u8, u8), T)> = Vec::deserialize(deserializer)?;
        assert_eq!(hard.len(), NUM_CELLS);
        let mut soft = hard.split_off(HARD_CELLS);
        let pair = soft.split_off(SOFT_CELLS);
        assert_eq!(hard.len(), HARD_CELLS);
        assert_eq!(soft.len(), SOFT_CELLS);
        assert_eq!(pair.len(), PAIR_CELLS);
        //Ok(Self::from_raw_parts(hard, soft, pair))
        Self::from_raw_parts(hard, soft, pair).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{resps_from_buf, Resp, Table, TableError};
    use super::{HARD_CELLS, NUM_CELLS, PAIR_CELLS, SOFT_CELLS};
    use crate::deck::{Card, Rank, Suit, ALL_RANKS};
    use crate::hand::Hand;
    use serde_json;
    use std::iter::repeat;

    const SUIT: Suit = Suit::Club;
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

    fn all_unique_table_keys() -> Vec<(usize, (Hand, Card))> {
        let mut keys = Vec::with_capacity(NUM_CELLS);
        let hands = [
            //hards
            Hand::new(&[Card::new(Rank::R2, SUIT), Card::new(Rank::R3, SUIT)]), // 5
            Hand::new(&[Card::new(Rank::R2, SUIT), Card::new(Rank::R4, SUIT)]), // 6
            Hand::new(&[Card::new(Rank::R2, SUIT), Card::new(Rank::R5, SUIT)]), // 7
            Hand::new(&[Card::new(Rank::R2, SUIT), Card::new(Rank::R6, SUIT)]), // 8
            Hand::new(&[Card::new(Rank::R2, SUIT), Card::new(Rank::R7, SUIT)]), // 9
            Hand::new(&[Card::new(Rank::R2, SUIT), Card::new(Rank::R8, SUIT)]), // 10
            Hand::new(&[Card::new(Rank::R2, SUIT), Card::new(Rank::R9, SUIT)]), // 11
            Hand::new(&[Card::new(Rank::R2, SUIT), Card::new(Rank::RT, SUIT)]), // 12
            Hand::new(&[Card::new(Rank::R3, SUIT), Card::new(Rank::RT, SUIT)]), // 13
            Hand::new(&[Card::new(Rank::R4, SUIT), Card::new(Rank::RT, SUIT)]), // 14
            Hand::new(&[Card::new(Rank::R5, SUIT), Card::new(Rank::RT, SUIT)]), // 15
            Hand::new(&[Card::new(Rank::R6, SUIT), Card::new(Rank::RT, SUIT)]), // 16
            Hand::new(&[Card::new(Rank::R7, SUIT), Card::new(Rank::RT, SUIT)]), // 17
            Hand::new(&[Card::new(Rank::R8, SUIT), Card::new(Rank::RT, SUIT)]), // 18
            Hand::new(&[Card::new(Rank::R9, SUIT), Card::new(Rank::RT, SUIT)]), // 19
            Hand::new(&[
                Card::new(Rank::R8, SUIT),
                Card::new(Rank::R2, SUIT),
                Card::new(Rank::RT, SUIT),
            ]), // 20, with 3 cards to avoid soft (9, A) and pair (10, 10)
            Hand::new(&[
                Card::new(Rank::R6, SUIT),
                Card::new(Rank::R6, SUIT),
                Card::new(Rank::R9, SUIT),
            ]), // 21, with 3 cards and no soft ace
            // softs
            Hand::new(&[Card::new(Rank::R2, SUIT), Card::new(Rank::RA, SUIT)]), // 13
            Hand::new(&[Card::new(Rank::R3, SUIT), Card::new(Rank::RA, SUIT)]), // 14
            Hand::new(&[Card::new(Rank::R4, SUIT), Card::new(Rank::RA, SUIT)]), // 15
            Hand::new(&[Card::new(Rank::R5, SUIT), Card::new(Rank::RA, SUIT)]), // 16
            Hand::new(&[Card::new(Rank::R6, SUIT), Card::new(Rank::RA, SUIT)]), // 17
            Hand::new(&[Card::new(Rank::R7, SUIT), Card::new(Rank::RA, SUIT)]), // 18
            Hand::new(&[Card::new(Rank::R8, SUIT), Card::new(Rank::RA, SUIT)]), // 19
            Hand::new(&[Card::new(Rank::R9, SUIT), Card::new(Rank::RA, SUIT)]), // 20
            Hand::new(&[Card::new(Rank::RT, SUIT), Card::new(Rank::RA, SUIT)]), // 21
            // pairs
            Hand::new(&[Card::new(Rank::R2, SUIT), Card::new(Rank::R2, SUIT)]), // 2s
            Hand::new(&[Card::new(Rank::R3, SUIT), Card::new(Rank::R3, SUIT)]), // 3s
            Hand::new(&[Card::new(Rank::R4, SUIT), Card::new(Rank::R4, SUIT)]), // 4s
            Hand::new(&[Card::new(Rank::R5, SUIT), Card::new(Rank::R5, SUIT)]), // 5s
            Hand::new(&[Card::new(Rank::R6, SUIT), Card::new(Rank::R6, SUIT)]), // 6s
            Hand::new(&[Card::new(Rank::R7, SUIT), Card::new(Rank::R7, SUIT)]), // 7s
            Hand::new(&[Card::new(Rank::R8, SUIT), Card::new(Rank::R8, SUIT)]), // 8s
            Hand::new(&[Card::new(Rank::R9, SUIT), Card::new(Rank::R9, SUIT)]), // 9s
            Hand::new(&[Card::new(Rank::RT, SUIT), Card::new(Rank::RT, SUIT)]), // 10s
            Hand::new(&[Card::new(Rank::RA, SUIT), Card::new(Rank::RA, SUIT)]), // As
        ];
        for hand in hands.iter() {
            keys.extend(
                // combine a copy of the hand with each dealer card
                repeat(hand.clone()).zip(
                    ALL_RANKS
                        .iter()
                        // filter out J Q and K (leaving 10) since all worth 10
                        .filter(|r| ![Rank::RJ, Rank::RQ, Rank::RK].contains(r))
                        // turn rank into a card
                        .map(|r| Card::new(*r, SUIT)),
                ),
            );
        }
        assert_eq!(keys.len(), NUM_CELLS);
        keys.into_iter().enumerate().collect()
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

    #[test]
    fn get_pair_aces() {
        // Fetches from the correct place when player hand has pair of aces, which is handled as a
        // special case. This is/should be handled implicitly by the get_all() test, but why not
        // make double sure
        const NOT_VAL: u8 = 0;
        const VAL: u8 = 1;
        let mut t: Table<u8> = Table::new();
        // fill all but the last cell with NOT_VAL. Last cell should have key [(A, A), A], which
        // has a pair of aces for the player hand
        t.fill(
            repeat(NOT_VAL)
                .take(NUM_CELLS - 1)
                .chain(repeat(VAL).take(1)),
        )
        .unwrap();
        let h = Hand::new(&[Card::new(Rank::RA, SUIT), Card::new(Rank::RA, SUIT)]);
        let d = Card::new(Rank::RA, SUIT);
        assert_eq!(t.get(&h, d).unwrap(), VAL);
    }

    #[test]
    fn get_all() {
        // all values are stored in the expected positions
        let mut t: Table<u16> = Table::new();
        t.fill(0..NUM_CELLS as u16).unwrap();
        for (i, key) in all_unique_table_keys().into_iter() {
            //eprintln!("{} {:?}", i, key);
            assert_eq!(t.get(&key.0, key.1).unwrap(), i as u16);
        }
    }

    #[test]
    fn get_notfilled() {
        // get on unfilled table fails
        let h = Hand::new(&vec![Card::new(Rank::RT, SUIT); 3]);
        let c = Card::new(Rank::R2, SUIT);
        let t: Table<()> = Table::new();
        assert_eq!(t.get(&h, c).unwrap_err(), TableError::NotFilled);
    }

    #[test]
    fn update_notfilled() {
        // update on unfilled table fails
        let h = Hand::new(&vec![Card::new(Rank::RT, SUIT); 3]);
        let c = Card::new(Rank::R2, SUIT);
        let mut t: Table<()> = Table::new();
        assert_eq!(t.update(&h, c, ()).unwrap_err(), TableError::NotFilled);
    }

    #[test]
    fn get_bust() {
        // get on busted hand fails
        let h = Hand::new(&vec![Card::new(Rank::RT, SUIT); 3]);
        let c = Card::new(Rank::R2, SUIT);
        let mut t: Table<()> = Table::new();
        t.fill(repeat(()).take(NUM_CELLS)).unwrap();
        assert_eq!(t.get(&h, c).unwrap_err(), TableError::HandIsBust(h, c));
    }

    #[test]
    fn update_bust() {
        // update on busted hand fails
        let h = Hand::new(&vec![Card::new(Rank::RT, SUIT); 3]);
        let c = Card::new(Rank::R2, SUIT);
        let mut t: Table<()> = Table::new();
        t.fill(repeat(()).take(NUM_CELLS)).unwrap();
        assert_eq!(
            t.update(&h, c, ()).unwrap_err(),
            TableError::HandIsBust(h, c)
        );
    }

    #[test]
    fn update() {
        // updating always returns old value and correctly stores new value
        let mut t: Table<u16> = Table::new();
        t.fill(1..NUM_CELLS as u16 + 1).unwrap();
        for (i, key) in all_unique_table_keys().into_iter() {
            // cast once now to avoid doing it a bunch later, plus increment to match value stored
            // in table
            let i = i as u16 + 1;
            // update should return old value
            assert_eq!(t.update(&key.0, key.1, i * 2).unwrap(), i);
            // future gets should return new value
            assert_eq!(t.get(&key.0, key.1).unwrap(), i * 2);
        }
    }

    #[test]
    fn serialize_identity() {
        // result of serialize -> deserialize is the same as the input
        let mut t_in: Table<u16> = Table::new();
        t_in.fill(0..NUM_CELLS as u16).unwrap();
        let bytes = serde_json::to_vec(&t_in).unwrap();
        let t_out: Table<u16> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(t_in, t_out);
    }

    #[test]
    fn from_raw_parts_missing_keys() {
        // sending Vecs with missing keys to Table::from_raw_parts() causes it to fail to build a
        // Table
        let h = vec![((0, 0), 0); HARD_CELLS];
        let s = vec![((0, 0), 0); SOFT_CELLS];
        let p = vec![((0, 0), 0); PAIR_CELLS];
        if let Err(e) = Table::from_raw_parts(h, s, p) {
            match e {
                TableError::MissingKeys(_) => {}
                _ => panic!(format!("Got the wrong type of error: {}", e)),
            }
        } else {
            panic!("Should have failed Table::from_raw_parts()");
        }
    }
}
