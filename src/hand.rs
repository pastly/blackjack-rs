use crate::deck::{Card, Rank};
use std::fmt;

// might not all be necessary
#[derive(Debug, PartialEq, Eq, Hash)]
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
        let have_ace = self.cards.iter().filter(|c| c.rank == Rank::RA).count() > 0;
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
            // if the first card is an ace, let it be worth 11
            let mut v = if hand.cards[0].rank == Rank::RA {
                11
            } else {
                hand.cards[0].value()
            };
            if hand.cards[1].rank == Rank::RA && v == 11 {
                // if second card is ace, if first card was also ace, just add one for total of 12
                v += 1;
            } else if hand.cards[1].rank == Rank::RA {
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
        for hand in all_pairs() {
            assert_eq!(
                hand.is_soft(),
                hand.cards.contains(&Card::new(Rank::RA, SUIT))
            );
        }
    }

    #[test]
    fn is_soft_2() {
        // 2 low cards plus an ace is still soft
        for mut hand in all_pairs() {
            // skip if already have ace
            assert_eq!(hand.cards.len(), 2);
            if hand.cards[0].rank == Rank::RA || hand.cards[1].rank == Rank::RA {
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
        for mut hand in all_pairs() {
            let orig_val = hand.value();
            // for this test, skip hands that start out with more than 0 aces
            if hand.cards[0].rank == Rank::RA || hand.cards[1].rank == Rank::RA {
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
        for hand in all_pairs() {
            assert!(!hand.is_bust());
        }
    }

    #[test]
    fn is_bust_2() {
        // some 3-card hands are bust. Trusts Hand::value() to be perfect
        for base in all_pairs() {
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
        for base in all_pairs() {
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
        for hand in all_pairs() {
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
        for base in all_pairs() {
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
}
