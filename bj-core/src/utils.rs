use crate::deck::{rand_card, rand_suit, Card, Rank};
use crate::hand::{rand_hand, Hand};
use crate::playstats::PlayStats;
use crate::table::{GameDesc, Table};
use rand::distributions::WeightedIndex;
use rand::prelude::*;

/// Generate a weighted-random next hand using player's statistics
pub fn rand_next_hand(stats: &Table<PlayStats>) -> (Hand, Card) {
    let (hands, weights): (Vec<GameDesc>, Vec<f32>) =
        stats.iter().map(|(tkey, s)| (tkey, s.weight())).unzip();
    let dist = WeightedIndex::new(&weights).unwrap();
    //println!("{:?}", weights);
    let tkey = hands[dist.sample(&mut thread_rng())];
    let hand = rand_hand(tkey);
    let dealer_suit = rand_suit();
    let card = match tkey.dealer {
        2 => Card::new(Rank::R2, dealer_suit),
        3 => Card::new(Rank::R3, dealer_suit),
        4 => Card::new(Rank::R4, dealer_suit),
        5 => Card::new(Rank::R5, dealer_suit),
        6 => Card::new(Rank::R6, dealer_suit),
        7 => Card::new(Rank::R7, dealer_suit),
        8 => Card::new(Rank::R8, dealer_suit),
        9 => Card::new(Rank::R9, dealer_suit),
        10 => Card::new(Rank::RT, dealer_suit),
        11 => Card::new(Rank::RA, dealer_suit),
        _ => unreachable!(format!(
            "It is impossible for the dealer to have a card valued at {}",
            tkey.dealer
        )),
    };
    (hand.unwrap(), card)
}

/// Generate a random hand as if from a shuffled infinite deck
pub fn uniform_rand_2card_hand() -> (Hand, Card) {
    (Hand::new(&[rand_card(), rand_card()]), rand_card())
}
