use crate::deck::{rand_suit, Card, Rank};
use crate::table::GameDesc;
use rand::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum HandType {
    Hard,
    Soft,
    Pair,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Hand {
    pub(crate) cards: Vec<Card>,
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.cards
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

impl Hand {
    pub fn new(cards: &[Card]) -> Self {
        assert!(cards.len() >= 2);
        Self {
            cards: cards.to_vec(),
        }
    }

    /// Returns whether or not a hand is soft. A hand is soft if it has at least 1 ace that can be
    /// worth either 1 or 11 without busting.
    pub fn is_soft(&self) -> bool {
        // Internally aces are worth 1 by default, so the idea is to see if we have more than 0
        // aces and if the total hand value (when calculated assuming all aces are only worth 1) is
        // 11 or less (i.e. there's "room" for at least one ace to be worth 11 instead of 1).
        // Cannot trivially use Hand::value() to get value as it returns the highest possible
        // value for non-bust hands.
        let have_ace = self.cards.iter().filter(|c| c.rank() == Rank::RA).count() > 0;
        let v = self.cards.iter().fold(0, |acc, c| acc + c.value());
        v <= 11 && have_ace
    }

    /// For non-bust hands, returns the highest possible total value of all cards in the hand. I.e.
    /// if a hand is soft, returns the high-value possibility. Use Hand::is_soft() to determine
    /// softness. For bust hands, we currently return lowest possible value (will obviously still
    /// be a bust), but since the value isn't so important once the hand is busted, this could
    /// change.
    pub fn value(&self) -> u8 {
        let mut num_ace = 0;
        let mut acc = 0;
        for c in self.cards.iter() {
            acc += match c.rank() {
                Rank::R2
                | Rank::R3
                | Rank::R4
                | Rank::R5
                | Rank::R6
                | Rank::R7
                | Rank::R8
                | Rank::R9
                | Rank::RT
                | Rank::RJ
                | Rank::RQ
                | Rank::RK => c.value(),
                Rank::RA => {
                    num_ace += 1;
                    c.value()
                }
            }
        }
        while acc <= 11 && num_ace > 0 {
            num_ace -= 1;
            acc += 10;
        }
        acc
    }

    /// Whether or not the hand has busted (whether it must be worth more than 21)
    pub fn is_bust(&self) -> bool {
        self.value() > 21
    }

    /// Whether or not the hand is a pair of same-ranked cards (never true for 3+ cards)
    pub fn is_pair(&self) -> bool {
        if self.cards.len() > 2 {
            return false;
        }
        assert_eq!(self.cards.len(), 2);
        // this is okay to do. 2-9 are obviously okay, ace is okay as long as Card::value() always
        // returns one of either 1 or 11 for an ace (which is true), and 10, J, Q, K all return 10
        // and we want to consider them all equal
        self.cards[0].value() == self.cards[1].value()
    }

    /// Whether or not the hand can be doubled (i.e. whether it has two cards or not)
    pub fn can_double(&self) -> bool {
        self.cards.len() == 2
    }

    /// Whether or not the hand can be split (i.e. whether or not it has two cards with the same
    /// value)
    fn can_split(&self) -> bool {
        self.cards.len() == 2 && self.cards[0].value() == self.cards[1].value()
    }

    /// Add a card to the hand. Upon completion of this method, all properties will update the new
    /// state of the hand: it may now be bust, no longer soft, have a different value, etc.
    pub fn push(&mut self, c: Card) {
        self.cards.push(c);
    }

    /// Split the hand into its two individual cards. If hand cannot be split (because the cards
    /// don't have the same value or because there is more than 2 cards) then returns an error.
    /// This method consumes the hand.
    pub fn split(self) -> Result<(Card, Card), HandError> {
        if !self.can_split() {
            return Err(HandError::CannotSplit(self));
        }
        Ok((self.cards[0], self.cards[1]))
    }

    /// Returns an interator over the cards in this hand
    pub fn cards(&self) -> impl Iterator<Item = &Card> {
        self.cards.iter()
    }

    /// Consumes this hand and returns an iterator over the cards in this hand
    pub fn into_cards(self) -> impl IntoIterator<Item = Card> {
        self.cards.into_iter()
    }
}

#[derive(Debug, PartialEq)]
pub enum HandError {
    CannotSplit(Hand),
    ImpossibleGameDesc(GameDesc),
}

impl std::error::Error for HandError {}

impl fmt::Display for HandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandError::ImpossibleGameDesc(desc) => write!(
                f,
                "Cannot make {:?} hand with value {} (against dealer {})",
                desc.hand, desc.player, desc.dealer
            ),
            HandError::CannotSplit(hand) => write!(f, "Impossible to split hand {}", hand),
        }
    }
}

/// Generate and random vector of cards that constitute a soft hand of the given value while
/// respecting the given min and max length (inclusive).
///
/// This is implemented in two ways:
/// 1. For small soft hands (i.e. soft 16 or below) and for max_len = 2, generate the only 2-card
///    hand that will work. Ace and a second card.
/// 2. For all other soft hands or for max_len > 2, generate a hard hand with value amt - 11, add
///    an ace, and return that.
///
/// As this is an internal helper function, asserts are used instead of Errors at this time:
/// - No such thing as a soft hand worth less than 12 or more than 21
/// - Refuse a min_len less than 2, a max_len less than min_len (they can be equal, however), and
/// require min_len to be no more than the length of the all-ace hand that sums to the rquested
/// amount. E.g.: soft 18 with all aces is 8 cards long (11 + 7*1).
///
/// The generation of a hard hand is delegated to cards_hard_sum_to().
fn cards_soft_sum_to(amt: u8, min_len: u8, max_len: u8) -> Vec<Card> {
    // no such thing as a soft hand worth less than 12 or more than 21
    assert!(amt >= 12);
    assert!(amt <= 21);
    assert!(min_len >= 2);
    assert!(min_len <= max_len);
    // soft 12 max length is 2 (ace_1 + ace_11)
    // soft 21 max length is 11 (10*ace_1 + 1*ace_11)
    // soft X max length is X-10
    assert!(min_len <= amt - 10);
    // automatically add the ace worth 1 or 11
    let remaining = amt - 11;
    let mut cards = vec![Card::new(Rank::RA, rand_suit())];
    let mut rng = thread_rng();
    // If there is very little remaining, just pick a second card and be done with it.
    // cards_hard_sum_to() wants to be able to return a legit non-soft non-pair hand of cards with
    // 2+ cards in it. We can't do that with a small soft hand.
    // Also just pick the obvious second hard if max_len is 2.
    cards = if remaining < 5 || max_len == 2 {
        let rank = match remaining {
            1 => Rank::RA,
            2 => Rank::R2,
            3 => Rank::R3,
            4 => Rank::R4,
            5 => Rank::R5,
            6 => Rank::R6,
            7 => Rank::R7,
            8 => Rank::R8,
            9 => Rank::R9,
            10 => *[Rank::RT, Rank::RJ, Rank::RQ, Rank::RK]
                .choose(&mut rng)
                .unwrap(),
            v => unreachable!(format!("Impossible to return card with value {}", v)),
        };
        cards.push(Card::new(rank, rand_suit()));
        cards
    } else {
        let max = std::cmp::max(2, max_len - 1);
        cards.extend(cards_hard_sum_to(remaining, 2, max));
        cards
    };
    cards.shuffle(&mut rng);
    cards
}

/// Generate a random vector of cards that constitute a hard hand of the given value while
/// respecting the given min and max length (inclusive).
///
/// It was surprisingly difficult to figure out how to juggle all the things to consider when
/// generating a random hard hand. So instead of percisely generating a hand that fits the
/// parameters the first time, generate random hands worth the given amount over and over until we
/// have one that is not soft, not a pair, and has between min_len and max_len cards (inclusively).
///
/// As this is an internal helper function, asserts are used instead of Errors at this time:
/// - There is no such thing as a hard hand worth less than 5.
/// - Refuse to generate a hard hand worth more than 21 even if perfectly possible. We don't need
/// to be able to do this now, and it makes reasoning about max_len easier
/// - Refuse a min_len less than 2, a max_len less than the min_len (it can be equal, however), and
/// enforce a max_len of at least 3 if amount requested is 20 or 21.
fn cards_hard_sum_to(amt: u8, min_len: u8, max_len: u8) -> Vec<Card> {
    assert!(amt >= 5);
    assert!(amt <= 21);
    assert!(min_len >= 2);
    assert!(min_len <= max_len);
    if amt >= 20 {
        // a hard 20 must have 3+ cards: ten + ten is a pair
        // a hard 21 must have 3+ cards: ten + ace is a soft 21
        assert!(max_len > 2);
    }
    let mut rng = thread_rng();
    // start of the "potentially infinite loop" if you get really unlucky forever with RNG. Or if
    // there's a programming error, but of course that's impossible.
    let mut hand = loop {
        // gen random set of cards worth the specified amount
        let cards = {
            let mut v = vec![];
            let mut remaining = amt;
            while remaining > 0 {
                let max = if remaining < 10 { remaining } else { 13 };
                let rank = match rng.gen_range(1, max + 1) {
                    1 => Rank::RA,
                    2 => Rank::R2,
                    3 => Rank::R3,
                    4 => Rank::R4,
                    5 => Rank::R5,
                    6 => Rank::R6,
                    7 => Rank::R7,
                    8 => Rank::R8,
                    9 => Rank::R9,
                    10 => Rank::RT,
                    11 => Rank::RJ,
                    12 => Rank::RQ,
                    13 => Rank::RK,
                    v => unreachable!(format!("Impossible to return card with value {}", v)),
                };
                v.push(Card::new(rank, rand_suit()));
                remaining -= rank.value();
            }
            v
        };
        // cards can't be used if wrong number
        if cards.len() < 2 || cards.len() < min_len as usize || cards.len() > max_len as usize {
            continue;
        }
        // hand can't be used if pair or soft
        let hand = Hand::new(&cards);
        if hand.is_pair() || hand.is_soft() {
            continue;
        }
        // final sanity check. hand isn't soft so Hand::value()'s rv of "highest possible value" is
        // **THE** value.
        assert_eq!(hand.value(), amt);
        break hand;
    };
    // got lucky this time. Return the hand.
    hand.cards.shuffle(&mut rng);
    hand.cards
}

pub fn rand_hand(desc: GameDesc) -> Result<Hand, HandError> {
    let mut rng = thread_rng();
    let s1 = rand_suit();
    let s2 = rand_suit();
    let t1 = *[Rank::RT, Rank::RJ, Rank::RQ, Rank::RK]
        .choose(&mut rng)
        .unwrap();
    let t2 = *[Rank::RT, Rank::RJ, Rank::RQ, Rank::RK]
        .choose(&mut rng)
        .unwrap();
    let h = match desc.hand {
        HandType::Pair => match desc.player {
            4 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R2, s2)]),
            6 => Hand::new(&[Card::new(Rank::R3, s1), Card::new(Rank::R3, s2)]),
            8 => Hand::new(&[Card::new(Rank::R4, s1), Card::new(Rank::R4, s2)]),
            10 => Hand::new(&[Card::new(Rank::R5, s1), Card::new(Rank::R5, s2)]),
            12 => Hand::new(&[Card::new(Rank::R6, s1), Card::new(Rank::R6, s2)]),
            14 => Hand::new(&[Card::new(Rank::R7, s1), Card::new(Rank::R7, s2)]),
            16 => Hand::new(&[Card::new(Rank::R8, s1), Card::new(Rank::R8, s2)]),
            18 => Hand::new(&[Card::new(Rank::R9, s1), Card::new(Rank::R9, s2)]),
            20 => Hand::new(&[Card::new(t1, s1), Card::new(t2, s2)]),
            22 => Hand::new(&[Card::new(Rank::RA, s1), Card::new(Rank::RA, s2)]),
            _ => return Err(HandError::ImpossibleGameDesc(desc)),
        },
        HandType::Soft => {
            if desc.player < 12 || desc.player > 21 {
                return Err(HandError::ImpossibleGameDesc(desc));
            }
            let cards = cards_soft_sum_to(desc.player, 2, 2);
            assert_eq!(
                cards.iter().fold(0, |acc, c| acc + c.value()),
                desc.player - 10
            );
            let h = Hand::new(&cards);
            assert_eq!(h.value(), desc.player);
            h
        }
        HandType::Hard => {
            if desc.player < 5 {
                return Err(HandError::ImpossibleGameDesc(desc));
            }
            // 2 cards adding to 20 is a pair, not hard, so must have three
            // 2 cards adding to 21 is a blackjack, which is soft , so must have three
            let num = if desc.player < 20 { 2 } else { 3 };
            let cards = cards_hard_sum_to(desc.player, num, num);
            let h = Hand::new(&cards);
            assert_eq!(h.value(), desc.player);
            h
        }
    };
    Ok(h)
}

#[cfg(test)]
mod tests {
    use super::{rand_hand, Hand, HandError, HandType};
    use crate::deck::{Card, Rank, Suit, ALL_RANKS};
    use crate::table::GameDesc;
    const SUIT: Suit = Suit::Club;
    const DEALER_VAL: u8 = 2;
    const RAND_REPS: usize = 1000;

    fn all_2card_hands() -> Vec<Hand> {
        let mut hands = vec![];
        for r1 in ALL_RANKS.iter() {
            let c1 = Card::new(*r1, SUIT);
            for r2 in ALL_RANKS.iter() {
                let c2 = Card::new(*r2, SUIT);
                hands.push(Hand::new(&[c1, c2]));
            }
        }
        hands
    }

    #[test]
    fn rand_pair_bad() {
        for _ in 0..RAND_REPS {
            // cannot ask for a random pair with value zero, odd value, or even value larger than 22
            for v in (0..=2)
                .chain((1..=23).step_by(2))
                .chain((24..=std::u8::MAX).step_by(2))
            {
                let desc = GameDesc::new(HandType::Pair, v, DEALER_VAL);
                assert_eq!(rand_hand(desc), Err(HandError::ImpossibleGameDesc(desc)));
            }
        }
    }

    #[test]
    fn rand_pair_good() {
        for _ in 0..RAND_REPS {
            // test all valid pairs
            for v in (4..=22).step_by(2) {
                let desc = GameDesc::new(HandType::Pair, v, DEALER_VAL);
                let h = rand_hand(desc).unwrap();
                // pair aces stored at value 22, not 12 like Hand::value() would say
                if v == 22 {
                    assert_eq!(h.value(), 12);
                } else {
                    assert_eq!(h.value(), v);
                }
            }
        }
    }

    #[test]
    fn rand_soft_bad() {
        for _ in 0..RAND_REPS {
            // cannot ask for a soft hand outside of valid soft hand range
            for v in (0..=11).chain(22..=std::u8::MAX) {
                let desc = GameDesc::new(HandType::Soft, v, DEALER_VAL);
                assert_eq!(rand_hand(desc), Err(HandError::ImpossibleGameDesc(desc)));
            }
        }
    }

    #[test]
    fn rand_soft_good() {
        for _ in 0..RAND_REPS {
            // test all valid soft hands
            for v in 12..=21 {
                let desc = GameDesc::new(HandType::Soft, v, DEALER_VAL);
                let h = rand_hand(desc).unwrap();
                assert_eq!(h.value(), v);
                assert!(h.cards.iter().filter(|c| c.rank() == Rank::RA).count() >= 1);
            }
        }
    }

    #[test]
    fn rand_hard_bad() {
        for _ in 0..RAND_REPS {
            for v in 0..=1 {
                let desc = GameDesc::new(HandType::Hard, v, DEALER_VAL);
                assert_eq!(rand_hand(desc), Err(HandError::ImpossibleGameDesc(desc)));
            }
        }
    }

    #[test]
    fn rand_hard_good() {
        for _ in 0..RAND_REPS {
            for v in 5..=21 {
                let desc = GameDesc::new(HandType::Hard, v, DEALER_VAL);
                let h = rand_hand(desc).unwrap();
                //eprintln!("{} {}", h, v);
                assert_eq!(h.value(), v);
            }
        }
    }

    #[test]
    fn value_le_21() {
        for hand in all_2card_hands() {
            // if the first card is an ace, let it be worth 11
            let mut v = if hand.cards[0].rank() == Rank::RA {
                11
            } else {
                hand.cards[0].value()
            };
            if hand.cards[1].rank() == Rank::RA && v == 11 {
                // if second card is ace, if first card was also ace, just add one for total of 12
                v += 1;
            } else if hand.cards[1].rank() == Rank::RA {
                // else if second card is ace, there's room for it to be worth 11
                v += 11;
            } else {
                // else just take the second card's value
                v += hand.cards[1].value();
            }
            // sanity check for the test itself, not really exercising the actual code
            assert!(v <= 21);
            // actual test
            assert_eq!(v, hand.value());
        }
    }

    #[test]
    fn is_soft_1() {
        // only tests the pairs for softness
        for hand in all_2card_hands() {
            assert_eq!(
                hand.is_soft(),
                hand.cards.contains(&Card::new(Rank::RA, SUIT))
            );
        }
    }

    #[test]
    fn is_soft_2() {
        // 2 low cards plus an ace is still soft
        for mut hand in all_2card_hands() {
            // skip if already have ace
            assert_eq!(hand.cards.len(), 2);
            if hand.cards[0].rank() == Rank::RA || hand.cards[1].rank() == Rank::RA {
                continue;
            }
            // note whether we expect it to be soft with an added ace
            let expect = hand.value() <= 10;
            // add an ace
            hand.cards.push(Card::new(Rank::RA, SUIT));
            eprintln!("{}", hand);
            // actual test
            assert_eq!(hand.is_soft(), expect);
        }
    }

    #[test]
    fn aces_1() {
        // 2..=11 aces are 12..=21 value
        // 12.. aces are 12.. value
        let base = Hand::new(&[Card::new(Rank::RA, SUIT), Card::new(Rank::RA, SUIT)]);
        // "extra" aces in addition to 2 already in the hand
        for extra in 0..=21 {
            let mut hand = base.clone();
            hand.cards.append(
                &mut std::iter::repeat(Card::new(Rank::RA, SUIT))
                    .take(extra)
                    .collect(),
            );
            // sanity check for test
            assert_eq!(hand.cards.len(), 2 + extra);
            // actual test
            if hand.cards.len() <= 11 {
                assert_eq!(hand.value(), 12 + extra as u8);
            } else {
                assert_eq!(hand.value(), 2 + extra as u8);
            }
            // might as well. soft if 11 or fewer aces
            assert_eq!(hand.is_soft(), hand.cards.len() <= 11);
        }
    }

    #[test]
    fn aces_2() {
        // a single ace in hand of 3 hards is either 11 or 1, depending on value of other 2 cards
        for mut hand in all_2card_hands() {
            let orig_val = hand.value();
            // for this test, skip hands that start out with more than 0 aces
            if hand.cards[0].rank() == Rank::RA || hand.cards[1].rank() == Rank::RA {
                continue;
            }
            hand.cards.push(Card::new(Rank::RA, SUIT));
            match orig_val {
                4..=10 => {
                    assert_eq!(hand.value(), orig_val + 11);
                }
                11..=21 => {
                    assert_eq!(hand.value(), orig_val + 1);
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn aces_3() {
        // a pair of aces with 3rd card worth 9 or less is 12 + 3rd card
        // else 2 + 3rd card
        let base = Hand::new(&[Card::new(Rank::RA, SUIT), Card::new(Rank::RA, SUIT)]);
        for r in ALL_RANKS.iter() {
            let mut hand = base.clone();
            hand.cards.push(Card::new(*r, SUIT));
            if r.value() <= 9 {
                assert_eq!(hand.value(), 12 + r.value());
            } else {
                // test sanity check, not actual test
                assert!(r.value() > 9);
                // actual test
                assert_eq!(hand.value(), 2 + r.value());
            }
        }
    }

    #[test]
    fn is_bust_1() {
        // all pairs are not bust
        for hand in all_2card_hands() {
            assert!(!hand.is_bust());
        }
    }

    #[test]
    fn is_bust_2() {
        // some 3-card hands are bust. Trusts Hand::value() to be perfect
        for base in all_2card_hands() {
            for r1 in ALL_RANKS.iter() {
                let mut hand = base.clone();
                hand.cards.push(Card::new(*r1, SUIT));
                assert_eq!(hand.is_bust(), hand.value() > 21);
            }
        }
    }

    #[test]
    fn is_bust_3() {
        // some 4-card hands are bust. Trusts Hand::value() to be perfect
        for base in all_2card_hands() {
            for r1 in ALL_RANKS.iter() {
                for r2 in ALL_RANKS.iter() {
                    let mut hand = base.clone();
                    hand.cards.push(Card::new(*r1, SUIT));
                    hand.cards.push(Card::new(*r2, SUIT));
                    assert_eq!(hand.is_bust(), hand.value() > 21);
                }
            }
        }
    }

    #[test]
    fn is_pair_1() {
        // hand is a pair if both cards have equal value
        for hand in all_2card_hands() {
            assert_eq!(hand.cards.len(), 2);
            assert_eq!(
                hand.is_pair(),
                hand.cards[0].value() == hand.cards[1].value()
            );
        }
    }

    #[test]
    fn is_pair_2() {
        // 3 cards are never a pair
        for base in all_2card_hands() {
            for r1 in ALL_RANKS.iter() {
                let mut hand3 = base.clone();
                hand3.cards.push(Card::new(*r1, SUIT));
                assert_eq!(hand3.cards.len(), 3);
                assert!(!hand3.is_pair());
                // 4 cards aren't either
                for r2 in ALL_RANKS.iter() {
                    let mut hand4 = hand3.clone();
                    hand4.cards.push(Card::new(*r2, SUIT));
                    assert_eq!(hand4.cards.len(), 4);
                    assert!(!hand4.is_pair());
                }
            }
        }
    }

    #[test]
    fn push() {
        // adding a 3rd and 4th card
        // - adds the correct cards to the hand
        // - adds them in the correct order
        // and does so for all possible 2-card starting hands. Yes this last bit is unnecessary.
        let c1 = Card::new(Rank::R2, Suit::Club);
        let c2 = Card::new(Rank::R2, Suit::Diamond);
        for mut hand in all_2card_hands() {
            hand.push(c1);
            assert_eq!(hand.cards.len(), 3);
            assert_eq!(hand.cards[2], c1);
            hand.push(c2);
            assert_eq!(hand.cards.len(), 4);
            assert_eq!(hand.cards[3], c2);
        }
    }

    #[test]
    fn split_1() {
        // splitting a hand with over 2 cards is not possible
        for base in all_2card_hands() {
            for r1 in ALL_RANKS.iter() {
                let mut hand3 = base.clone();
                hand3.cards.push(Card::new(*r1, SUIT));
                assert!(!hand3.can_split());
                assert_eq!(
                    hand3.clone().split(),
                    Err(HandError::CannotSplit(hand3.clone()))
                );
                for r2 in ALL_RANKS.iter() {
                    let mut hand4 = hand3.clone();
                    hand4.cards.push(Card::new(*r2, SUIT));
                    assert!(!hand4.can_split());
                    assert_eq!(hand4.clone().split(), Err(HandError::CannotSplit(hand4)));
                }
            }
        }
    }

    #[test]
    fn split_2() {
        // can only split if the 2 cards have the same value
        for hand in all_2card_hands() {
            assert_eq!(
                hand.clone().split().is_ok(),
                hand.cards[0].value() == hand.cards[1].value(),
            );
            assert_eq!(
                hand.can_split(),
                hand.cards[0].value() == hand.cards[1].value(),
            );
        }
    }

    #[test]
    fn can_double() {
        // doubling is possible with and 2 cards, but not 3 or 4
        for base in all_2card_hands() {
            assert!(base.can_double());
            for r1 in ALL_RANKS.iter() {
                let mut hand3 = base.clone();
                hand3.cards.push(Card::new(*r1, SUIT));
                assert!(!hand3.can_double());
                for r2 in ALL_RANKS.iter() {
                    let mut hand4 = hand3.clone();
                    hand4.cards.push(Card::new(*r2, SUIT));
                    assert!(!hand4.can_double());
                }
            }
        }
    }
}
