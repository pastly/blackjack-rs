use bj_core::deck::Card;
use bj_core::hand::Hand;
use bj_web_core::{card_char, char_card};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct HandResult {
    pub player: Hand,
    pub dealer: Card,
    pub correct: bool,
    // duration (in seconds, not ms) since first result at which point this result was recorded
    pub time: f64,
}

pub(crate) fn to_string(results: &[HandResult]) -> String {
    if results.is_empty() {
        return String::new();
    }
    results
        .iter()
        .map(|res| {
            let player: String = res.player.cards().map(|&c| card_char(c)).collect();
            let dealer = card_char(res.dealer);
            //let player = format!("{}", res.player);
            //let dealer = format!("{}", res.dealer);
            let correct: u8 = if res.correct { 1 } else { 0 };
            format!("{}/{}/{}/{}", player, dealer, correct, res.time)
        })
        .collect::<Vec<String>>()
        .join(",")
}

pub(crate) fn from_string(res_string: &str) -> Result<Vec<HandResult>, String> {
    let mut v = vec![];
    if res_string.is_empty() {
        return Ok(vec![]);
    }
    for s in res_string.split(',') {
        if s.is_empty() {
            continue;
        }
        let parts = s.split('/').collect::<Vec<_>>();
        let player: Hand = {
            let mut cards = vec![];
            for c in parts[0].chars() {
                cards.push(match char_card(c) {
                    Some(c) => c,
                    None => return Err("Invalid player card character".into()),
                });
            }
            if cards.len() < 2 {
                return Err("Invalid player cards".into());
            }
            Hand::new(&cards)
        };
        let dealer = {
            let chars: Vec<char> = parts[1].chars().collect();
            if chars.len() != 1 {
                return Err("Invalid dealer card character".into());
            }
            match char_card(chars[0]) {
                Some(c) => c,
                None => return Err("Invalid dealer card character".into()),
            }
        };
        let correct = match parts[2] {
            "1" => true,
            "0" => false,
            _ => return Err("Invalid correct bool".into()),
        };
        let time = match parts[3].parse::<f64>() {
            Ok(t) => t,
            Err(_) => return Err("Invalid timestamp".into()),
        };
        v.push(HandResult {
            player,
            dealer,
            correct,
            time,
        });
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bj_core::deck::rand_card;
    use rand::prelude::*;

    const NUM_RAND_HANDS: usize = 10_000;

    fn rand_hands(n: usize) -> Vec<Hand> {
        (0..n)
            .map(|_| {
                let n_cards = *[2, 3].choose(&mut thread_rng()).unwrap();
                rand_hand(n_cards)
            })
            .collect()
    }

    fn rand_hand(n: usize) -> Hand {
        Hand::new(&(0..n).map(|_| rand_card()).collect::<Vec<_>>())
    }

    #[test]
    fn foo() {
        let mut v = vec![];
        for _ in 0..1000 {
            v.push(HandResult{
                player: rand_hand(2),
                dealer: rand_card(),
                correct: *[true, false].choose(&mut thread_rng()).unwrap(),
                time: thread_rng().gen::<f64>(),
            });
        }
        use std::fs::OpenOptions;
        {
            let f = OpenOptions::new()
                .write(true)
                .create(true)
                .open("1.asdf").unwrap();
            serde_cbor::to_writer(f, &v);
        }
        {
            use std::io::Write;
            let s = to_string(&v);
            let mut f = OpenOptions::new()
                .write(true)
                .create(true)
                .open("2.asdf").unwrap();
            write!(f, "{}", &s);
        }
    }

    #[test]
    fn vec_identity() {
        // Starting with a valid Vec<HandResult>, it is able to be converted to a String and back
        // and still be the same.
        let mut input: Vec<HandResult> = vec![];
        for player in rand_hands(NUM_RAND_HANDS) {
            let dealer = rand_card();
            let correct = *[true, false].choose(&mut thread_rng()).unwrap();
            let time = thread_rng().gen::<f64>();
            input.push(HandResult {
                player,
                dealer,
                correct,
                time,
            });
        }
        let s = to_string(&input);
        let output = from_string(&s).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn single() {
        // A single HandResult works out fine
        for player in rand_hands(NUM_RAND_HANDS) {
            let dealer = rand_card();
            let correct = *[true, false].choose(&mut thread_rng()).unwrap();
            let time = thread_rng().gen::<f64>();
            let hr_in = HandResult {
                player,
                dealer,
                correct,
                time,
            };
            let s = to_string(&[hr_in.clone()]);
            let hr_out = from_string(&s).unwrap();
            assert_eq!(hr_out.len(), 1);
            assert_eq!(hr_in, hr_out[0]);
        }
    }

    #[test]
    fn empty_vec() {
        let v: Vec<HandResult> = vec![];
        let s = to_string(&v);
        assert!(s.is_empty());
    }

    #[test]
    fn empty_string() {
        let v = from_string("").unwrap();
        assert!(v.is_empty());
    }

    #[test]
    fn bad_player_string() {
        for player_string in &[
            "",
            "a",
            "23",
            &format!("{}{}.", card_char(rand_card()), card_char(rand_card())),
            &format!("{}.{}", card_char(rand_card()), card_char(rand_card())),
            &format!(".{}{}", card_char(rand_card()), card_char(rand_card())),
            &format!(".{}*{}a", card_char(rand_card()), card_char(rand_card())),
        ] {
            let dealer = rand_card();
            let correct = *[1, 0].choose(&mut thread_rng()).unwrap();
            let time = thread_rng().gen::<f64>();
            let out = from_string(&format!(
                "{}/{}/{}/{}",
                player_string, dealer, correct, time
            ));
            assert!(out.is_err());
        }
    }
}
