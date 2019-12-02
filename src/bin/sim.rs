use blackjack::deck::{Card, Deck};
use blackjack::hand::Hand;
use blackjack::playstats::PlayStats;
use blackjack::table::{resp_from_char, resps_from_buf, Resp, Table};
use clap;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::{self, Write};

fn def_playstats_table() -> Table<PlayStats> {
    const NUM_CELLS: usize = 10 * (17 + 9 + 10);
    let mut t = Table::new();
    t.fill(vec![PlayStats::new(); NUM_CELLS]).unwrap();
    t
}

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

fn create_if_not_exist<T>(fname: &str, data: T) -> Result<(), io::Error>
where
    T: Serialize,
{
    match OpenOptions::new().create_new(true).write(true).open(fname) {
        Ok(fd) => {
            // able to create the file, so we need to fill it
            serde_json::to_writer(fd, &data).unwrap();
            Ok(())
        }
        Err(e) => {
            // unable to create the file, and that might be because it already exists.
            // ignore errors from it already existing, but bubble up all others
            match e.kind() {
                io::ErrorKind::AlreadyExists => Ok(()),
                _ => Err(e),
            }
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
        .arg(
            clap::Arg::with_name("stats")
                .short("s")
                .long("stats")
                .value_name("FILE")
                .help("Read/write play stats from the file")
                .default_value("play-stats.json"),
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
    // safe to unwrap bc --stats is required
    create_if_not_exist(matches.value_of("stats").unwrap(), def_playstats_table())?;
    let mut stats: Table<PlayStats> = serde_json::from_reader(
        OpenOptions::new()
            .read(true)
            .open(matches.value_of("stats").unwrap())
            .unwrap(),
    )
    .unwrap();
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
