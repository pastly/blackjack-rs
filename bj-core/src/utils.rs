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

pub mod playstats_table {
    use crate::playstats::PlayStats;
    use crate::table::{Table, NUM_CELLS};

    pub fn parse_to_string(table: &Table<PlayStats>) -> String {
        // 6 chars per table item, 360 cells in the table.
        // "XX/YY,"      2 for each value, plus '/' and ','
        // Doesn't have to be perfect, but should avoid reallocation in the vast majority of cases.
        const EXPECTED_MAX_LEN: usize = 360 * 6;
        let mut s = String::with_capacity(EXPECTED_MAX_LEN);
        let (hards, softs, pairs) = table.as_values_sorted();
        for item in hards.iter().chain(softs.iter()).chain(pairs.iter()) {
            s.push_str(&format!("{}/{},", item.correct(), item.seen()));
        }
        // remove last comma
        assert!(s.ends_with(','));
        s.truncate(s.len() - 1);
        s.shrink_to_fit();
        s
    }

    pub fn parse_from_string(s: String) -> Result<Table<PlayStats>, String> {
        let fracts = s.split(',').collect::<Vec<&str>>();
        if fracts.len() != NUM_CELLS {
            return Err(format!(
                "Provided table has {} items, not {}. Not storing it.",
                fracts.len(),
                NUM_CELLS
            ));
        }
        // parse string into Vec<PlayStats>
        let mut v = Vec::with_capacity(NUM_CELLS);
        for fract in fracts {
            let mut stat = PlayStats::new();
            let parts = fract.split('/').collect::<Vec<&str>>();
            if parts.len() != 2 {
                return Err(format!("'{}' is not a valid fraction", fract));
            }
            let correct = match parts[0].parse::<u32>() {
                Ok(v) => v,
                Err(e) => {
                    return Err(format!("'{}' not a valid u32: {}", parts[0], e));
                }
            };
            let seen = match parts[1].parse::<u32>() {
                Ok(v) => v,
                Err(e) => {
                    return Err(format!("'{}' not a valid u32: {}", parts[0], e));
                }
            };
            if correct > seen {
                return Err(format!(
                    "correct {} cannot be greater than seen {}",
                    correct, seen
                ));
            }
            stat.inc_by(seen - correct, false);
            stat.inc_by(correct, true);
            v.push(stat);
        }
        // Construct table with Vec
        let table = match Table::new(v.into_iter()) {
            Ok(t) => t,
            Err(e) => {
                return Err(format!("Problem constructing table: {}", e));
            }
        };
        Ok(table)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::iter::{once, repeat};

        #[test]
        fn identity_basic() {
            let table_in = Table::new(repeat(PlayStats::new()).take(NUM_CELLS)).unwrap();
            let table_out = parse_from_string(parse_to_string(&table_in)).unwrap();
            assert_eq!(table_in, table_out);

            let mut s_in = repeat("0/0,").take(NUM_CELLS).collect::<String>();
            s_in.truncate(s_in.len() - 1);
            let s_out = parse_to_string(&parse_from_string(s_in.clone()).unwrap());
            assert_eq!(s_in, s_out);
        }

        #[test]
        fn identity_harder() {
            let mut ps1 = PlayStats::new();
            ps1.inc_by(10, true);
            ps1.inc_by(100, false);
            let table_in = Table::new(
                repeat(PlayStats::new())
                    .take(10)
                    .chain(once(ps1))
                    .chain(repeat(PlayStats::new()).take(NUM_CELLS - 10 - 1)),
            )
            .unwrap();
            let table_out = parse_from_string(parse_to_string(&table_in)).unwrap();
            assert_eq!(table_in, table_out);

            let ps2 = "100/110,";
            let mut s_in = repeat("0/0,")
                .take(10)
                .chain(once(ps2))
                .chain(repeat("0/0,").take(NUM_CELLS - 10 - 1))
                .collect::<String>();
            s_in.truncate(s_in.len() - 1);
            let s_out = parse_to_string(&parse_from_string(s_in.clone()).unwrap());
            assert_eq!(s_in, s_out);
        }

        #[test]
        fn empty_string() {
            assert!(parse_from_string("".into()).is_err());
        }

        #[test]
        fn gibberish_string() {
            let s = "23908ucsklj;a48902uyjch j.2347y90".into();
            assert!(parse_from_string(s).is_err());
        }

        #[test]
        fn invalid_num_items() {
            // also incidentally tests the correct number of items but with a trailing comma that
            // should cause parsing to fail
            for num in 1..NUM_CELLS + 10 {
                let s = repeat("0/0,").take(num).collect::<String>();
                assert!(parse_from_string(s).is_err());
            }
        }

        #[test]
        fn bad_fraction() {
            // number of correct cannot be more than number seen
            for ps in &["1/0,", "100/0", "100/99"] {
                let mut s = repeat("0/0,")
                    .take(10)
                    .chain(once(*ps))
                    .chain(repeat("0/0,").take(NUM_CELLS - 10 - 1))
                    .collect::<String>();
                s.truncate(s.len() - 1);
                assert!(parse_from_string(s).is_err());
            }
        }
    }
}
