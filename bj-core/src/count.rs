use crate::deck::{Card, Rank};
const DECK_LEN: u16 = 52;

pub trait CountSystem {
    fn card_value(&self, card: Card) -> i16;
    fn cards_value(&self, cards: &[Card]) -> i16 {
        cards.iter().fold(0, |acc, &c| acc + self.card_value(c))
    }
}

pub struct HiLo;

impl HiLo {
    pub fn new() -> Self {
        Self {}
    }
}

impl CountSystem for HiLo {
    fn card_value(&self, card: Card) -> i16 {
        match card.rank() {
            Rank::R2 | Rank::R3 | Rank::R4 | Rank::R5 | Rank::R6 => 1,
            Rank::R7 | Rank::R8 | Rank::R9 => 0,
            Rank::RT | Rank::RJ | Rank::RQ | Rank::RK | Rank::RA => -1,
        }
    }
}

pub struct StatefulHiLo {
    hl: HiLo,
    num_decks: u8,
    seen_cards: u16,
    count: i16,
}

impl StatefulHiLo {
    pub fn new(num_decks: u8) -> Self {
        assert!(num_decks > 0);
        Self {
            hl: HiLo::new(),
            num_decks,
            seen_cards: 0,
            count: 0,
        }
    }

    pub fn update(&mut self, card: Card) {
        assert!(self.seen_cards < u16::from(self.num_decks) * DECK_LEN);
        self.seen_cards += 1;
        self.count += self.hl.card_value(card);
    }

    pub fn update_many(&mut self, cards: &[Card]) {
        for c in cards {
            self.update(*c);
        }
    }

    pub fn true_count(&self) -> f32 {
        let decks_remaining =
            f32::from(self.num_decks) - (f32::from(self.seen_cards) / f32::from(DECK_LEN));
        if self.count == 0 || decks_remaining == 0.0 {
            return 0.0;
        }
        f32::from(self.count) / decks_remaining
    }
}

#[cfg(test)]
mod hilo_tests {
    use super::{CountSystem, HiLo, StatefulHiLo};
    use crate::deck::{Deck, Rank};

    #[test]
    fn each_card_correct() {
        // each individual card has its count correctly fetched. At this time, this is basically
        // just reimplementing the card_value() method
        let mut d = Deck::new();
        let hl = HiLo::new();
        while let Ok(card) = d.draw() {
            let expect = match card.rank() {
                Rank::R2 | Rank::R3 | Rank::R4 | Rank::R5 | Rank::R6 => 1,
                Rank::R7 | Rank::R8 | Rank::R9 => 0,
                Rank::RT | Rank::RJ | Rank::RQ | Rank::RK | Rank::RA => -1,
            };
            let actual = hl.card_value(card);
            assert_eq!(actual, expect);
        }
    }

    #[test]
    fn full_deck_correct() {
        // store a full deck in a vector and get its count. Should be 0.
        let mut d = Deck::new();
        let hl = HiLo::new();
        let cards = {
            let mut cards = vec![];
            while let Ok(c) = d.draw() {
                cards.push(c);
            }
            cards
        };
        assert_eq!(hl.cards_value(&cards), 0);
    }

    #[test]
    fn running_zero_is_true_zero() {
        // stateful hilo should start at true 0 and also be true 0 when it has seen an equal number
        // of high and low cards
        use crate::deck::{Card, Rank, Suit};
        let mut hl = StatefulHiLo::new(1);
        assert_eq!(hl.count, 0);
        assert_eq!(hl.true_count(), 0.0);
        let lo = Card::new(Rank::R2, Suit::Club);
        let hi = Card::new(Rank::RT, Suit::Club);
        hl.update(lo);
        hl.update(hi);
        assert_eq!(hl.seen_cards, 2);
        assert_eq!(hl.count, 0);
        assert_eq!(hl.true_count(), 0.0);
        for _ in 0..10 {
            hl.update(lo);
        }
        for _ in 0..10 {
            hl.update(hi);
        }
        assert_eq!(hl.count, 0);
        assert_eq!(hl.true_count(), 0.0);
    }

    #[test]
    fn true_count_ends_zero() {
        // for a variety of shoe sizes, after counting through all cards, stateful hilo should
        // still report true count of 0
        for num_decks in &[1, 6, 8, std::u8::MAX] {
            let mut d = Deck::with_length(usize::from(*num_decks));
            let mut hl = StatefulHiLo::new(*num_decks);
            while let Ok(card) = d.draw() {
                hl.update(card);
                // if we happen to have a running count of 0, let's check true count is
                // zero too ... just for fun!
                if hl.count == 0 {
                    assert_eq!(hl.true_count(), 0.0);
                }
            }
            assert_eq!(hl.true_count(), 0.0);
        }
    }

    #[test]
    fn run_through_decks() {
        use super::DECK_LEN;
        // for a variety of shoe sizes, make sure the true count is calculated correctly for every
        // single card drawn
        for num_decks in &[1, 6, 8, std::u8::MAX] {
            let mut d = Deck::with_length(usize::from(*num_decks));
            let mut hl = StatefulHiLo::new(*num_decks);
            let mut running = 0;
            let mut num_seen: u16 = 0;
            while let Ok(card) = d.draw() {
                num_seen += 1;
                running += HiLo::new().card_value(card);
                let true_count = {
                    if num_seen == u16::from(*num_decks) * DECK_LEN {
                        0.0
                    } else {
                        f32::from(running)
                            / (f32::from(*num_decks) - (f32::from(num_seen) / f32::from(DECK_LEN)))
                    }
                };
                hl.update(card);
                //eprintln!(
                //    "num={} run={} true={} hl.true={}",
                //    num_seen,
                //    running,
                //    true_count,
                //    hl.true_count()
                //);
                assert_eq!(hl.true_count(), true_count);
            }
        }
    }

    #[test]
    fn update_many() {
        // updating the count in one big batch produces the same result as doing it one card at a
        // time. Test a handful of times since decks are shuffled
        const NUM_DECKS: u8 = 10;
        for _ in 0..100 {
            let mut d = Deck::with_length(NUM_DECKS.into());
            // take the first 20 cards
            let cards = {
                let mut cards = vec![];
                while let Ok(card) = d.draw() {
                    cards.push(card);
                    if cards.len() == 20 {
                        break;
                    }
                }
                cards
            };
            let mut hl1 = StatefulHiLo::new(NUM_DECKS);
            let mut hl2 = StatefulHiLo::new(NUM_DECKS);
            for c in cards.iter() {
                hl1.update(*c);
            }
            hl2.update_many(&cards);
            assert_eq!(hl1.true_count(), hl2.true_count());
            assert_eq!(hl1.count, hl2.count);
            assert_eq!(hl1.seen_cards, hl2.seen_cards);
        }
    }
}
