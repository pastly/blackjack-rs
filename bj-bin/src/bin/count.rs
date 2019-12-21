use bj_bin::prompt;
use bj_core::deck::{Card, Deck};
use bj_core::hand::Hand;
use std::io::{self, BufRead, BufReader, Write};

fn count_of_card(c: Card) -> i8 {
    match c.value() {
        2..=5 => 1,
        6..=8 => 0,
        9..=10 | 1 => -1,
        _ => unreachable!(),
    }
}

fn count_of_hand(h: &Hand) -> i8 {
    h.cards().fold(0, |acc, &c| acc + count_of_card(c))
}

fn prompt_for_num(
    h: &Hand,
    in_buf: &mut impl BufRead,
    out_buf: &mut impl Write,
) -> io::Result<prompt::Command> {
    let s = &format!("count of {}", h);
    loop {
        match prompt::prompt(s, in_buf, out_buf)? {
            prompt::Command::SaveQuit | prompt::Command::Quit => break Ok(prompt::Command::Quit),
            prompt::Command::Num(n) => break Ok(prompt::Command::Num(n)),
            _ => {
                writeln!(out_buf, "Give either a number or 'quit'")?;
                continue;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = BufReader::new(io::stdin());
    let mut output = io::stdout();
    let mut deck = Deck::new_infinite();
    loop {
        let hand = Hand::new(&[deck.draw()?, deck.draw()?]);
        let cmd = prompt_for_num(&hand, &mut input, &mut output)?;
        let val = match cmd {
            prompt::Command::Quit => return Ok(()),
            prompt::Command::Num(n) => n as i8,
            _ => unreachable!(),
        };
        let actual = count_of_hand(&hand);
        if val == actual {
            println!("Correct");
        } else {
            println!("count of {} is {}", hand, actual);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{count_of_card, count_of_hand};
    use bj_core::deck::Deck;
    use bj_core::hand::Hand;

    #[test]
    fn card_always_has_count() {
        // all cards have some count
        let mut d = Deck::new();
        while let Ok(c) = d.draw() {
            let count = count_of_card(c);
            assert!(count >= -1 && count <= 1);
        }
    }

    #[test]
    fn card_and_hand_count_match() {
        // getting the count for a hand matches the sum of the count value for each individual card
        let mut deck = Deck::new_infinite();
        for len in 2..=4 {
            for _rep in 0..100 {
                let hand = Hand::new(&vec![deck.draw().unwrap(); len]);
                let expect = hand.cards().fold(0, |acc, &c| acc + count_of_card(c));
                let actual = count_of_hand(&hand);
                assert_eq!(expect, actual);
            }
        }
    }
}
