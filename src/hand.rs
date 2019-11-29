use crate::deck::{Card, Rank};
use std::fmt;

#[derive(Clone)]
pub struct Hand {
    cards: Vec<Card>,
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
    pub fn new(cards: &[Card; 2]) -> Self {
        Self {
            cards: cards.to_vec(),
        }
    }

    pub fn is_soft(&self) -> bool {
        self.cards[0].rank == Rank::RA || self.cards[1].rank == Rank::RA
    }

    pub fn value(&self) -> u8 {
        let mut num_ace = 0;
        let mut acc = 0;
        for c in self.cards.iter() {
            acc += match c.rank {
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
                    c.rank.value()
                }
            }
        }
        while acc > 21 && num_ace > 0 {
            num_ace -= 1;
            acc -= 10;
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::Hand;
    use crate::deck::{Card, Rank, Suit, ALL_RANKS};
    const SUIT: Suit = Suit::Club;

    fn all_pairs() -> Vec<Hand> {
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
    fn value_le_21() {
        for hand in all_pairs() {
            //eprintln!("{}", hand);
            let mut v = hand.cards[0].value();
            if hand.cards[1].rank == Rank::RA && v >= 11 {
                v += 1;
            } else {
                v += hand.cards[1].value();
            }
            // sanity check for the test itself, not really exercising the actual code
            assert!(v <= 21);
            // actual test
            assert_eq!(v, hand.value());
        }
    }

    #[test]
    fn is_soft() {
        for hand in all_pairs() {
            assert_eq!(
                hand.is_soft(),
                hand.cards.contains(&Card::new(Rank::RA, SUIT))
            );
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
            // might as well
            assert!(hand.is_soft());
        }
    }

    #[test]
    fn aces_2() {
        // a single ace in hand of 3 hards is either 11 or 1, depending on value of other 2 cards
        for mut hand in all_pairs() {
            let orig_val = hand.value();
            // for this test, skip hands that start out with more than 0 aces
            if hand.cards[0].rank == Rank::RA || hand.cards[1].rank == Rank::RA {
                continue;
            }
            hand.cards.push(Card::new(Rank::RA, SUIT));
            match orig_val {
                4..=10 => {
                    eprintln!("{}", hand);
                    assert_eq!(hand.value(), orig_val + 11);
                }
                11..=21 => {
                    eprintln!("{}", hand);
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
}
