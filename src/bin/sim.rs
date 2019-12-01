use blackjack::deck::{Card, Deck};
use blackjack::hand::Hand;
use blackjack::table::{resp_from_char, resps_from_buf, Resp, Table};
use clap;
use std::fs::OpenOptions;
use std::io::{self, Write};

fn prompt(p: &Hand, d: Card) -> Result<Option<Resp>, io::Error> {
    loop {
        print!("{} / {} > ", p, d);
        io::stdout().flush()?;
        let mut s = String::new();
        io::stdin().read_line(&mut s)?;
        s = s.trim().to_string();
        if s.is_empty() {
            println!();
            return Ok(None);
        }
        let c: char = s.chars().take(1).collect::<Vec<char>>()[0].to_ascii_uppercase();
        let resp = resp_from_char(c);
        if resp.is_some() {
            return Ok(resp);
        } else {
            println!("Bad response: {}", c);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap::App::new(clap::crate_name!())
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .arg(
            clap::Arg::with_name("table")
                .short("t")
                .long("table")
                .value_name("FILE")
                .help("Table of ideal moves")
                .required(true),
        )
        .get_matches();
    let mut deck = Deck::new_infinite();
    let mut table = Table::new();
    table.fill(resps_from_buf(
        OpenOptions::new()
            .read(true)
            // safe to unwrap because --table is required
            .open(matches.value_of("table").unwrap())?,
    ))?;
    loop {
        let player = Hand::new(&[deck.draw()?, deck.draw()?]);
        let dealer_up = deck.draw()?;
        if let Some(choice) = prompt(&player, dealer_up)? {
            // safe to unwrap because table is filled, player has 21 or less, and all other errors
            // are panic!()
            let best = table.get(&player, dealer_up).unwrap();
            print!("{} ", choice);
            if choice == best {
                println!("correct");
            } else {
                println!("wrong. Should {}", best);
            }
        } else {
            break;
        }
    }
    Ok(())
}
