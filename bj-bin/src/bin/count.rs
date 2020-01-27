use bj_bin::prompt;
use bj_core::count::{CountSystem, HiLo};
use bj_core::deck::{Card, Deck};
use bj_core::hand::Hand;
use clap::{crate_authors, crate_name, crate_version, value_t, App, Arg};
use std::io::{self, BufRead, BufReader, Write};

fn prompt_for_num<D>(
    hand_or_card: &D,
    in_buf: &mut impl BufRead,
    out_buf: &mut impl Write,
) -> io::Result<prompt::Command>
where
    D: std::fmt::Display,
{
    let s = &format!("count of {}", hand_or_card);
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
    let matches = App::new(String::from(crate_name!()) + " count")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("cards")
                .short("c")
                .long("cards")
                .value_name("NUM")
                .help("Number of cards to show at once")
                .default_value("1"),
        )
        .get_matches();
    let num_cards = value_t!(matches, "cards", usize)?;
    if num_cards == 0 {
        return Err("Must specify at least 1 card".into());
    }
    let mut input = BufReader::new(io::stdin());
    let mut output = io::stdout();
    let mut deck = Deck::new_infinite();
    let hilo = HiLo::new();
    loop {
        let (actual, cmd) = if num_cards == 1 {
            let c = deck.draw()?;
            let cmd = prompt_for_num(&c, &mut input, &mut output)?;
            (hilo.card_value(c), cmd)
        } else {
            let cards = (0..num_cards)
                .map(|_| deck.draw().unwrap())
                .collect::<Vec<Card>>();
            let h = Hand::new(&cards);
            let cmd = prompt_for_num(&h, &mut input, &mut output)?;
            (hilo.cards_value(&cards), cmd)
        };
        let val = match cmd {
            prompt::Command::Quit => return Ok(()),
            prompt::Command::Num(n) => n as i16,
            _ => unreachable!(),
        };
        if val == actual {
            println!("Correct");
        } else {
            println!("count is {}", actual);
        }
    }
}
