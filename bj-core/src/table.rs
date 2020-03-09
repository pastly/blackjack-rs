use crate::deck::{Card, Rank, Suit};
use crate::hand::{Hand, HandError, HandType};
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, SerializeSeq, Serializer};
use serde_derive;
use std::collections::HashMap;
use std::convert::From;
use std::fmt;

const HARD_CELLS: usize = 17 * 10;
const SOFT_CELLS: usize = 9 * 10;
const PAIR_CELLS: usize = 10 * 10;
pub(crate) const NUM_CELLS: usize = HARD_CELLS + SOFT_CELLS + PAIR_CELLS;
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

/// Key used in Table
#[derive(
    PartialEq, Debug, Copy, Clone, Eq, Hash, serde_derive::Serialize, serde_derive::Deserialize,
)]
pub struct GameDesc {
    pub hand: HandType,
    pub player: u8,
    pub dealer: u8,
}

impl GameDesc {
    pub(crate) fn new(hand: HandType, player: u8, dealer: u8) -> Self {
        Self {
            hand,
            player,
            dealer,
        }
    }
}

/// Get an arbitrary Hand that matches the given GameDesc.
///
/// While this function currently returns the same hand given the same input, this is not
/// guaranteed to be the case always.
///
/// Returns HandError::ImpossibleGameDesc() if bad desc, else a Hand.
pub fn player_hand_from_desc(desc: GameDesc) -> Result<Hand, HandError> {
    let s1 = Suit::Club;
    let s2 = Suit::Club;
    let hand = match desc.hand {
        HandType::Hard => match desc.player {
            5 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R3, s2)]),
            6 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R4, s2)]),
            7 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R5, s2)]),
            8 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R6, s2)]),
            9 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R7, s2)]),
            10 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R8, s2)]),
            11 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R9, s2)]),
            12 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::RT, s2)]),
            13 => Hand::new(&[Card::new(Rank::R3, s1), Card::new(Rank::RT, s2)]),
            14 => Hand::new(&[Card::new(Rank::R4, s1), Card::new(Rank::RT, s2)]),
            15 => Hand::new(&[Card::new(Rank::R5, s1), Card::new(Rank::RT, s2)]),
            16 => Hand::new(&[Card::new(Rank::R6, s1), Card::new(Rank::RT, s2)]),
            17 => Hand::new(&[Card::new(Rank::R7, s1), Card::new(Rank::RT, s2)]),
            18 => Hand::new(&[Card::new(Rank::R8, s1), Card::new(Rank::RT, s2)]),
            19 => Hand::new(&[Card::new(Rank::R9, s1), Card::new(Rank::RT, s2)]),
            20 => Hand::new(&[
                Card::new(Rank::RT, s1),
                Card::new(Rank::R8, s2),
                Card::new(Rank::R2, s2),
            ]),
            21 => Hand::new(&[
                Card::new(Rank::RT, s1),
                Card::new(Rank::R9, s2),
                Card::new(Rank::R2, s2),
            ]),
            _ => return Err(HandError::ImpossibleGameDesc(desc)),
        },
        HandType::Soft => {
            let a = Card::new(Rank::RA, s1);
            match desc.player {
                13 => Hand::new(&[a, Card::new(Rank::R2, s2)]),
                14 => Hand::new(&[a, Card::new(Rank::R3, s2)]),
                15 => Hand::new(&[a, Card::new(Rank::R4, s2)]),
                16 => Hand::new(&[a, Card::new(Rank::R5, s2)]),
                17 => Hand::new(&[a, Card::new(Rank::R6, s2)]),
                18 => Hand::new(&[a, Card::new(Rank::R7, s2)]),
                19 => Hand::new(&[a, Card::new(Rank::R8, s2)]),
                20 => Hand::new(&[a, Card::new(Rank::R9, s2)]),
                21 => Hand::new(&[a, Card::new(Rank::RT, s2)]),
                _ => return Err(HandError::ImpossibleGameDesc(desc)),
            }
        }
        HandType::Pair => match desc.player {
            4 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R2, s2)]),
            6 => Hand::new(&[Card::new(Rank::R3, s1), Card::new(Rank::R3, s2)]),
            8 => Hand::new(&[Card::new(Rank::R4, s1), Card::new(Rank::R4, s2)]),
            10 => Hand::new(&[Card::new(Rank::R5, s1), Card::new(Rank::R5, s2)]),
            12 => Hand::new(&[Card::new(Rank::R6, s1), Card::new(Rank::R6, s2)]),
            14 => Hand::new(&[Card::new(Rank::R7, s1), Card::new(Rank::R7, s2)]),
            16 => Hand::new(&[Card::new(Rank::R8, s1), Card::new(Rank::R8, s2)]),
            18 => Hand::new(&[Card::new(Rank::R9, s1), Card::new(Rank::R9, s2)]),
            20 => Hand::new(&[Card::new(Rank::RT, s1), Card::new(Rank::RT, s2)]),
            22 => Hand::new(&[Card::new(Rank::RA, s1), Card::new(Rank::RA, s2)]),
            _ => return Err(HandError::ImpossibleGameDesc(desc)),
        },
    };
    Ok(hand)
}

/// Get an arbitrary card that matches the dealer's card in the given GameDesc
///
/// While this function currently returns the same card given the same input, this is not
/// guaranteed to always be the case. The suit might change in future implementations.
///
/// Returns HandError::ImpossibleGameDesc() if bad desc, else a Card.
pub fn dealer_card_from_desc(desc: GameDesc) -> Result<Card, HandError> {
    let s = Suit::Club;
    let card = match desc.dealer {
        2 => Card::new(Rank::R2, s),
        3 => Card::new(Rank::R3, s),
        4 => Card::new(Rank::R4, s),
        5 => Card::new(Rank::R5, s),
        6 => Card::new(Rank::R6, s),
        7 => Card::new(Rank::R7, s),
        8 => Card::new(Rank::R8, s),
        9 => Card::new(Rank::R9, s),
        10 => Card::new(Rank::RT, s),
        11 => Card::new(Rank::RA, s),
        _ => return Err(HandError::ImpossibleGameDesc(desc)),
    };
    Ok(card)
}

#[derive(PartialEq, Debug)]
pub enum TableError {
    IncorrectFillLength(usize, usize),
    HandIsBust(Hand, Card),
    MissingKeys(String),
}

impl std::error::Error for TableError {}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
/// Table contains three logical subtables: the hard hands, soft hands, and pairs.  In all
/// subtables there are 10 columns (dealer shows 2, 3, 4, ... ace). The first (hard) table has 17
/// rows (player hand value 5-21). The second (soft) table has 9 rows (player hand value 13-21).
/// The third (pairs) table has 10 rows (player hand pair of 2s, 3s, 4s ... 10s, aces). This
/// results in 360 total cells. Table::new() takes an iterable and fills in the subtables
/// left-to-right and top-to-bottom one right after another.  For a visual example of what a Table
/// looks like (e.g. if used to store the best move for a player to make), see the blackjack
/// strategy cards on the Wizard of Odds website:
/// https://wizardofodds.com/games/blackjack/strategy/calculator/.
#[derive(Debug, PartialEq)]
pub struct Table<T>(HashMap<GameDesc, T>)
where
    T: PartialEq + Copy;

impl<T> Table<T>
where
    T: PartialEq + Copy,
{
    /// Construct a new Table from the given iterable.
    ///
    /// The iterable must be exactly 360 items in length, else returns a TableError.
    ///
    /// See Table's documentation for more information.
    pub fn new<I>(vals: I) -> Result<Self, TableError>
    where
        I: IntoIterator<Item = T>,
    {
        let mut t = Self {
            0: HashMap::with_capacity(NUM_CELLS),
        };
        t.fill(vals)?;
        Ok(t)
    }

    fn fill<I>(&mut self, vals: I) -> Result<(), TableError>
    where
        I: IntoIterator<Item = T>,
    {
        let mut cell_idx = 0;
        let mut vals = vals.into_iter();
        // hard table
        for player_value in 5..=21 {
            for dealer_up in 2..=11 {
                let k = GameDesc::new(HandType::Hard, player_value, dealer_up);
                if let Some(v) = vals.next() {
                    assert!(self.0.insert(k, v).is_none());
                    cell_idx += 1;
                } else {
                    return Err(TableError::IncorrectFillLength(NUM_CELLS, cell_idx));
                }
            }
        }
        // soft table
        for player_value in 13..=21 {
            for dealer_up in 2..=11 {
                let k = GameDesc::new(HandType::Soft, player_value, dealer_up);
                if let Some(v) = vals.next() {
                    assert!(self.0.insert(k, v).is_none());
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
                let k = GameDesc::new(HandType::Pair, *player_value, dealer_up);
                if let Some(v) = vals.next() {
                    assert!(self.0.insert(k, v).is_none());
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
        Ok(())
    }

    fn key(player_hand: &Hand, dealer_shows: Card) -> GameDesc {
        let p = if player_hand.is_pair() && player_hand.cards[0].rank() == Rank::RA {
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
        let ty = if player_hand.is_pair() {
            HandType::Pair
        } else if player_hand.is_soft() {
            HandType::Soft
        } else {
            HandType::Hard
        };
        GameDesc::new(ty, p, d)
    }

    /// Lookup and return the value stored at the given location in the table, if it exists.
    /// If the player's hand is bust, then lookup would fail and an error is returned. If a lookup
    /// fails because the calcuated key is missing (which should be impossible, but ... ya know
    /// ...), then return an error.
    pub fn get(&self, player_hand: &Hand, dealer_shows: Card) -> Result<T, TableError> {
        if player_hand.value() > 21 {
            return Err(TableError::HandIsBust(player_hand.clone(), dealer_shows));
        }
        assert!(player_hand.value() >= 2);
        let key = Table::<T>::key(player_hand, dealer_shows);
        if let Some(v) = self.0.get(&key) {
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
    /// If the player's hand is bust, then lookup would fail and an error is returned. Even after
    /// successfully inserting the new value, if there was no original value, will return
    /// Err(TableError::MissingKeys)
    pub fn update(
        &mut self,
        player_hand: &Hand,
        dealer_shows: Card,
        val: T,
    ) -> Result<T, TableError> {
        if player_hand.value() > 21 {
            return Err(TableError::HandIsBust(player_hand.clone(), dealer_shows));
        }
        let key = Table::<T>::key(player_hand, dealer_shows);
        match self.0.insert(key, val) {
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
    fn has_all_keys(&self) -> bool {
        // easy check: do all subtables have the right number of keys
        self.0.keys().filter(|k| k.hand == HandType::Hard).count() == HARD_CELLS
            && self.0.keys().filter(|k| k.hand == HandType::Soft).count() == SOFT_CELLS
            && self.0.keys().filter(|k| k.hand == HandType::Pair).count() == PAIR_CELLS
            // harder check: are there any keys that don't exist in the subtables
            && HARD_KEYS.iter().filter(|k| self.0.contains_key(&GameDesc::new(HandType::Hard, k.0, k.1))).count() == HARD_CELLS
            && SOFT_KEYS.iter().filter(|k| self.0.contains_key(&GameDesc::new(HandType::Soft, k.0, k.1))).count() == SOFT_CELLS
            && PAIR_KEYS.iter().filter(|k| self.0.contains_key(&GameDesc::new(HandType::Pair, k.0, k.1))).count() == PAIR_CELLS
    }

    /// An internal-only constructor for help during final deserialization
    ///
    /// Takes arrays for the hard, soft, and pair subtables, checks they are the correct length,
    /// assumes they have all the right keys in their key/value pairs, and builds the Table.
    fn from_single_vec(v: Vec<(GameDesc, T)>) -> Result<Self, TableError> {
        assert_eq!(v.len(), NUM_CELLS);
        let mut d = HashMap::with_capacity(NUM_CELLS);
        for kv in v.into_iter() {
            d.insert(kv.0, kv.1);
        }
        let t = Self { 0: d };
        if t.has_all_keys() {
            Ok(t)
        } else {
            Err(TableError::MissingKeys(
                "Cannot contruct from raw parts".to_owned(),
            ))
        }
    }

    /// Consume the Table and split it into sorted vectors of the hard, soft, and pair subtables.
    pub fn into_values_sorted(self) -> (Vec<T>, Vec<T>, Vec<T>) {
        let mut resps = Vec::with_capacity(NUM_CELLS);
        // Safety: okay to set len <= capacity. Rest of function will fill in each item in this vec
        // such that it is valid
        unsafe {
            resps.set_len(NUM_CELLS);
        }
        for (desc, resp) in self.into_iter() {
            let start = match desc.hand {
                HandType::Hard => 0,
                HandType::Soft => HARD_CELLS,
                HandType::Pair => HARD_CELLS + SOFT_CELLS,
            };
            const WIDTH: usize = 10;
            let col = usize::from(desc.dealer - 2);
            let row: usize = match desc.hand {
                HandType::Hard => usize::from(desc.player - 5),
                HandType::Soft => usize::from(desc.player - 13),
                HandType::Pair => usize::from(desc.player / 2 - 2),
            };
            let idx = start + row * WIDTH + col;
            assert!(idx < NUM_CELLS);
            std::mem::replace(&mut resps[idx], resp);
        }
        let mut hards = resps;
        let mut softs = hards.split_off(HARD_CELLS);
        let pairs = softs.split_off(SOFT_CELLS);
        (hards, softs, pairs)
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.0.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.0.values_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&GameDesc, &T)> {
        self.0.iter()
    }
}

impl<T> IntoIterator for Table<T>
where
    T: PartialEq + Copy,
{
    type Item = (GameDesc, T);
    type IntoIter = std::collections::hash_map::IntoIter<GameDesc, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> std::ops::AddAssign for Table<T>
where
    T: PartialEq + Copy + std::ops::AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        for (game_desc, val) in rhs.iter() {
            let player = player_hand_from_desc(*game_desc).unwrap();
            let dealer = dealer_card_from_desc(*game_desc).unwrap();
            let mut agg = self.get(&player, dealer).unwrap();
            agg += *val;
            self.update(&player, dealer, agg).unwrap();
        }
    }
}

impl<T> Serialize for Table<T>
where
    T: PartialEq + Copy + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for e in self.0.iter() {
            seq.serialize_element(&e)?;
        }
        seq.end()
    }
}

impl<'de, T> Deserialize<'de> for Table<T>
where
    T: PartialEq + Copy + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Vec<(GameDesc, T)> = Vec::deserialize(deserializer)?;
        assert_eq!(v.len(), NUM_CELLS);
        //Ok(Self::from_single_vec(hard, soft, pair))
        Self::from_single_vec(v).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::{Card, Rank, Suit, ALL_RANKS};
    use crate::hand::{Hand, HandError, HandType};
    use crate::resp::resps_from_buf;
    use serde_json;
    use std::iter::repeat;

    const SUIT: Suit = Suit::Club;
    const T1: &str = "
# It's not important which tables these are, but for completeness,
# these are the tables for 4+ deck, dealer hit soft 17, double after split,
# no surrender, peek for BJ
# https://wizardofodds.com/games/blackjack/strategy/calculator/
# hard
H  H  H  H  H  H  H  H  H  H
H  H  H  H  H  H  H  H  H  H
H  H  H  H  H  H  H  H  H  H
H  H  H  H  H  H  H  H  H  H
H  Dh Dh Dh Dh H  H  H  H  H
Dh Dh Dh Dh Dh Dh Dh Dh H  H
Dh Dh Dh Dh Dh Dh Dh Dh Dh Dh
H  H  S  S  S  H  H  H  H  H
S  S  S  S  S  H  H  H  H  H
S  S  S  S  S  H  H  H  H  H
S  S  S  S  S  H  H  H  H  H
S  S  S  S  S  H  H  H  H  H
S  S  S  S  S  S  S  S  S  S
S  S  S  S  S  S  S  S  S  S
S  S  S  S  S  S  S  S  S  S
S  S  S  S  S  S  S  S  S  S
S  S  S  S  S  S  S  S  S  S
# soft
H  H  H  Dh Dh H  H  H  H  H
H  H  H  Dh Dh H  H  H  H  H
H  H  Dh Dh Dh H  H  H  H  H
H  H  Dh Dh Dh H  H  H  H  H
H  Dh Dh Dh Dh H  H  H  H  H
Ds Ds Ds Ds Ds S  S  H  H  H
S  S  S  S  Ds S  S  S  S  S
S  S  S  S  S  S  S  S  S  S
S  S  S  S  S  S  S  S  S  S
# pair
P  P  P  P  P  P  H  H  H  H
P  P  P  P  P  P  H  H  H  H
H  H  H  P  P  H  H  H  H  H
Dh Dh Dh Dh Dh Dh Dh Dh H  H
P  P  P  P  P  H  H  H  H  H
P  P  P  P  P  P  H  H  H  H
P  P  P  P  P  P  P  P  P  P
P  P  P  P  P  S  P  P  S  S
S  S  S  S  S  S  S  S  S  S
P  P  P  P  P  P  P  P  P  P
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
    fn new_empty() {
        // Should fail to fill table with empty iter
        assert_eq!(
            Table::<()>::new(vec![]).unwrap_err(),
            TableError::IncorrectFillLength(NUM_CELLS, 0)
        );
    }

    #[test]
    fn new_short() {
        // Should fail to fill with too few items
        for count in 1..NUM_CELLS {
            assert_eq!(
                Table::new(vec![(); count]).unwrap_err(),
                TableError::IncorrectFillLength(NUM_CELLS, count)
            );
        }
    }

    #[test]
    fn new_long() {
        // Should fail to fill with too few items
        for count in NUM_CELLS + 1..NUM_CELLS + 10 {
            assert_eq!(
                Table::new(vec![(); count]).unwrap_err(),
                TableError::IncorrectFillLength(NUM_CELLS, NUM_CELLS + 1)
            );
        }
    }

    #[test]
    fn fill() {
        // Should succeed with exactly correct number of items
        assert!(Table::new(vec![(); NUM_CELLS]).is_ok());
    }

    #[test]
    fn new_responses() {
        // with our known-good strategy, try filling and make sure no error
        assert!(Table::new(resps_from_buf(T1.as_bytes()).unwrap()).is_ok());
    }

    #[test]
    fn get_1() {
        // all 2-card hands against all dealer show cards have a best response
        let t = Table::new(resps_from_buf(T1.as_bytes()).unwrap());
        assert!(t.is_ok());
        let t = t.unwrap();
        for hand in all_club_pairs() {
            for dealer in all_clubs() {
                assert!(t.get(&hand, dealer).is_ok());
            }
        }
    }

    #[test]
    fn get_2() {
        // 3-card hands should have a best response as long as they are worth 21 or less
        let t = Table::new(resps_from_buf(T1.as_bytes()).unwrap());
        assert!(t.is_ok());
        let t = t.unwrap();
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
        // fill all but the last cell with NOT_VAL. Last cell should have key [(A, A), A], which
        // has a pair of aces for the player hand
        let t = Table::<u8>::new(
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
        let t = Table::new(0..NUM_CELLS as u16).unwrap();
        for (i, key) in all_unique_table_keys().into_iter() {
            //eprintln!("{} {:?}", i, key);
            assert_eq!(t.get(&key.0, key.1).unwrap(), i as u16);
        }
    }

    #[test]
    fn get_bust() {
        // get on busted hand fails
        let h = Hand::new(&vec![Card::new(Rank::RT, SUIT); 3]);
        let c = Card::new(Rank::R2, SUIT);
        let t = Table::new(repeat(()).take(NUM_CELLS)).unwrap();
        assert_eq!(t.get(&h, c).unwrap_err(), TableError::HandIsBust(h, c));
    }

    #[test]
    fn update_bust() {
        // update on busted hand fails
        let h = Hand::new(&vec![Card::new(Rank::RT, SUIT); 3]);
        let c = Card::new(Rank::R2, SUIT);
        let mut t = Table::new(repeat(()).take(NUM_CELLS)).unwrap();
        assert_eq!(
            t.update(&h, c, ()).unwrap_err(),
            TableError::HandIsBust(h, c)
        );
    }

    #[test]
    fn update() {
        // updating always returns old value and correctly stores new value
        let mut t = Table::new(1..NUM_CELLS as u16 + 1).unwrap();
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
        let t_in = Table::new(0..NUM_CELLS as u16).unwrap();
        let bytes = serde_json::to_vec(&t_in).unwrap();
        let t_out: Table<u16> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(t_in, t_out);
    }

    #[test]
    fn from_single_vec_missing_keys() {
        // sending Vecs with missing keys to Table::from_single_vec() causes it to fail to build a
        // Table
        let h = vec![(GameDesc::new(HandType::Hard, 0, 0), 0); HARD_CELLS].into_iter();
        let s = vec![(GameDesc::new(HandType::Soft, 0, 0), 0); SOFT_CELLS].into_iter();
        let p = vec![(GameDesc::new(HandType::Pair, 0, 0), 0); PAIR_CELLS].into_iter();
        let v: Vec<(GameDesc, u8)> = h.chain(s).chain(p).collect();
        if let Err(e) = Table::from_single_vec(v) {
            match e {
                TableError::MissingKeys(_) => {}
                _ => panic!(format!("Got the wrong type of error: {}", e)),
            }
        } else {
            panic!("Should have failed Table::from_single_vec()");
        }
    }

    fn assert_bad_player(t: HandType, p: u8) {
        let desc = GameDesc {
            hand: t,
            player: p,
            dealer: 2,
        };
        assert_eq!(
            player_hand_from_desc(desc),
            Err(HandError::ImpossibleGameDesc(desc))
        );
    }

    fn assert_good_player(t: HandType, p: u8) {
        let desc = GameDesc {
            hand: t,
            player: p,
            dealer: 2,
        };
        let hand = player_hand_from_desc(desc).unwrap();
        if t != HandType::Pair || p != 22 {
            assert_eq!(hand.value(), p);
        } else {
            // special case for pair of aces. Value is 12, but stored at 22
            assert_eq!(hand.value(), 12);
        }
    }

    #[test]
    fn player_hand_bad() {
        // fail to get player hand from GameDesc with bad player hand descriptions
        for val in (0..=4).chain(22..=std::u8::MAX) {
            assert_bad_player(HandType::Hard, val);
        }
        for val in
            (23..=std::u8::MAX).chain(vec![0, 1, 2, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21].drain(0..))
        {
            assert_bad_player(HandType::Pair, val);
        }
        for val in (0..=12).chain(22..=std::u8::MAX) {
            assert_bad_player(HandType::Soft, val);
        }
    }

    #[test]
    fn player_hand_good() {
        for val in 5..=21 {
            assert_good_player(HandType::Hard, val);
        }
        for val in 13..=21 {
            assert_good_player(HandType::Soft, val);
        }
        for val in vec![4, 6, 8, 10, 12, 14, 16, 18, 20, 22].into_iter() {
            assert_good_player(HandType::Pair, val);
        }
    }

    #[test]
    fn dealer_card_bad() {
        for val in (0..=1).chain(12..=std::u8::MAX) {
            let desc = GameDesc {
                hand: HandType::Hard,
                player: 5,
                dealer: val,
            };
            assert_eq!(
                dealer_card_from_desc(desc),
                Err(HandError::ImpossibleGameDesc(desc))
            );
        }
    }

    #[test]
    fn dealer_card_good() {
        for val in 2..=11 {
            let desc = GameDesc {
                hand: HandType::Hard,
                player: 5,
                dealer: val,
            };
            let card = dealer_card_from_desc(desc).unwrap();
            if val != 11 {
                assert_eq!(card.value(), val);
            } else {
                // aces worth one
                assert_eq!(card.value(), 1);
            }
        }
    }

    #[test]
    fn addassign_1() {
        let mut t1 = Table::new(repeat(1).take(360)).unwrap();
        let t2 = Table::new(repeat(2).take(360)).unwrap();
        t1 += t2;
        for v in t1.values() {
            assert_eq!(*v, 3);
        }
    }

    #[test]
    fn addassign_2() {
        let mut t1 = Table::new(repeat(0).take(359).chain(repeat(1).take(1))).unwrap();
        let t2 = Table::new(repeat(1).take(1).chain(repeat(0).take(359))).unwrap();
        t1 += t2;
        let num_worth_1 = t1.values().filter(|&&v| v == 1).count();
        assert_eq!(num_worth_1, 2);
    }
}
