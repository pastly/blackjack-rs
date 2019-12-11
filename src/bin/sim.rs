use blackjack::deck::{rand_suit, Card, Deck, Rank};
use blackjack::hand::{rand_hand, Hand};
use blackjack::playstats::PlayStats;
use blackjack::table::{resp_from_char, resps_from_buf, GameDesc, Resp, Table};
use blackjack::utils::{read_maybexz, write_maybexz};
use clap::{arg_enum, crate_authors, crate_name, crate_version, value_t, App, Arg};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::Serialize;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};

fn def_playstats_table() -> Table<PlayStats> {
    const NUM_CELLS: usize = 10 * (17 + 9 + 10);
    let mut t = Table::new();
    t.fill(vec![PlayStats::new(); NUM_CELLS]).unwrap();
    t
}

fn print_game_stats(stats: &Table<PlayStats>) {
    let (num_seen, num_correct) = stats
        .values()
        .fold((0, 0), |acc, s| (acc.0 + s.seen(), acc.1 + s.correct()));
    println!(
        "{:.2}% of {} recorded games correct",
        num_correct as f32 / num_seen as f32 * 100.0,
        num_seen
    );
}

/// Generate a weighted-random next hand using player's statistics
fn rand_next_hand(stats: &Table<PlayStats>) -> (Hand, Card) {
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

#[derive(Debug, PartialEq)]
enum Command {
    Quit,
    Save,
    SaveQuit,
    Resp(Resp),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Quit => write!(f, "Quit"),
            Command::Save => write!(f, "Save"),
            Command::SaveQuit => write!(f, "SaveQuit"),
            Command::Resp(r) => write!(f, "Resp({})", r),
        }
    }
}

fn command_from_str(s: &str) -> Option<Command> {
    let s: &str = &s.to_ascii_uppercase();
    match s {
        "QUIT" => Some(Command::Quit),
        "SAVE" => Some(Command::Save),
        "SAVEQUIT" | "SAVE QUIT" => Some(Command::SaveQuit),
        _ => {
            if s.len() != 1 {
                return None;
            }
            if let Some(resp) = resp_from_char(s.chars().take(1).collect::<Vec<char>>()[0]) {
                Some(Command::Resp(resp))
            } else {
                None
            }
        }
    }
}

enum RandType {
    Uniform,
    Weighted,
}

impl fmt::Display for RandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RandType::Uniform => "UR",
            RandType::Weighted => "WR",
        };
        write!(f, "{}", s)
    }
}

fn prompt(
    p: &Hand,
    d: Card,
    rand_type: RandType,
    stat: PlayStats,
    in_buf: &mut impl BufRead,
    out_buf: &mut impl Write,
) -> Result<Command, io::Error> {
    loop {
        write!(
            out_buf,
            "({} {}/{}) {} / {} > ",
            rand_type,
            stat.correct(),
            stat.seen(),
            p,
            d
        )?;
        out_buf.flush()?;
        let mut s = String::new();
        in_buf.read_line(&mut s)?;
        s = s.trim().to_string();
        if s.is_empty() {
            //println!();
            //return Ok(None);
            continue;
        }
        if let Some(cmd) = command_from_str(&s) {
            return Ok(cmd);
        } else {
            writeln!(out_buf, "Bad response: {}", s)?;
        }
    }
}

/// Create the given file if it doesn't already exist. If it needs to be created, fill it with the
/// given serializable data. Otherwise don't use the given data at all. Bubbles up any file system
/// errors (other than the error of "already exists." Panics if unable to serialize/write the data
/// to the file.
fn create_if_not_exist<T>(fname: &str, data: &T) -> Result<(), Box<dyn std::error::Error>>
where
    T: Serialize,
{
    match OpenOptions::new().create_new(true).write(true).open(fname) {
        Ok(fd) => {
            // able to create the file, so we need to fill it
            println!("Creating and filling {}", fname);
            match write_maybexz(fd, data, fname.ends_with(".xz")) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => {
            // unable to create the file, and that might be because it already exists.
            // ignore errors from it already existing, but bubble up all others
            match e.kind() {
                io::ErrorKind::AlreadyExists => Ok(()),
                _ => Err(e.into()),
            }
        }
    }
}

arg_enum! {
    #[derive(PartialEq, Debug)]
    enum StatsSaveStrat {
        Never,
        EveryHand,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(String::from(crate_name!()) + " sim")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("table")
                .short("t")
                .long("table")
                .value_name("FILE")
                .help("Table of ideal moves")
                .required(true),
        )
        .arg(
            Arg::with_name("stats")
                .short("s")
                .long("stats")
                .value_name("FILE")
                .help("Read/write play stats from the file")
                .default_value("play-stats.json.xz"),
        )
        .arg(
            Arg::with_name("statssave")
                .long("save-stats")
                .possible_values(&StatsSaveStrat::variants())
                .case_insensitive(true)
                .default_value("EveryHand")
                .value_name("WHEN")
                .help("When to save play statistics to disk"),
        )
        .arg(
            Arg::with_name("unirand")
            .long("rand-every")
            .default_value("2")
            .value_name("CNT")
            .help("Every CNT rolls, generate hand uniformally at random as opposed to weighted by play statistics. 0 means never, 1 means always.")
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
    let stats_fname = matches.value_of("stats").unwrap();
    let save_stats = value_t!(matches, "statssave", StatsSaveStrat)?;
    let uni_rand_every = {
        let val = value_t!(matches, "unirand", u64)?;
        if val == 0 {
            std::u64::MAX
        } else {
            val
        }
    };
    let mut hand_count = 0;
    let mut stats = match save_stats {
        StatsSaveStrat::Never => def_playstats_table(),
        _ => {
            create_if_not_exist(stats_fname, &def_playstats_table())?;
            println!("Reading PlayStats from {}", stats_fname);
            let fd = OpenOptions::new().read(true).open(stats_fname).unwrap();
            read_maybexz(fd, stats_fname.ends_with(".xz"))?
        }
    };
    print_game_stats(&stats);
    loop {
        hand_count += 1;
        let (player, dealer_up, rand_type) = if hand_count == uni_rand_every {
            // played enough hands that we should generate the next hand uniformally at random.
            // Reset hand count and do so.
            hand_count = 0;
            //println!("Uniformally random hand chosen, not based on play stats");
            (
                Hand::new(&[deck.draw()?, deck.draw()?]),
                deck.draw()?,
                RandType::Uniform,
            )
        } else {
            // haven't played enough hands yet, so generate randomly using play stats for weight
            let (h, d) = rand_next_hand(&stats);
            (h, d, RandType::Weighted)
        };
        let current_stat = stats.get(&player, dealer_up)?;
        let command = prompt(
            &player,
            dealer_up,
            rand_type,
            current_stat,
            &mut BufReader::new(io::stdin()),
            &mut io::stdout(),
        )?;
        // handle easy commands first. New commands should either return from main() entirely or
        // restart the loop
        match command {
            Command::Quit => return Ok(()),
            Command::Save | Command::SaveQuit => {
                // This saves play stats and restarts the loop, which means it acts like this hand
                // never happened. This gives the player a way to skip a hand without consequences.
                let fd = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(stats_fname)?;
                write_maybexz(fd, &stats, stats_fname.ends_with(".xz"))?;
                print_game_stats(&stats);
                if command == Command::SaveQuit {
                    return Ok(());
                }
                continue;
            }
            Command::Resp(_) => { /* will handle below */ }
        };
        let resp = if let Command::Resp(r) = command {
            r
        } else {
            unreachable!("Should have handled non-Command::Resp already");
        };
        // Handle the case that the user actually hit, stand, etc.
        let best = table.get(&player, dealer_up)?;
        print!("{} ", resp);
        if resp == best {
            println!("correct");
        } else {
            println!("wrong. Should {}", best);
        }
        // increment the stats for this hand type
        let mut stat = stats.get(&player, dealer_up)?;
        stat.inc(resp == best);
        stats.update(&player, dealer_up, stat)?;
        // print stats if user got it wrong
        if resp != best {
            print_game_stats(&stats);
        }
        // maybe save
        match save_stats {
            StatsSaveStrat::Never => {}
            StatsSaveStrat::EveryHand => {
                let fd = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(stats_fname)?;
                write_maybexz(fd, &stats, stats_fname.ends_with(".xz"))?;
            }
        }
    }
    //Ok(())
}

#[cfg(test)]
mod tests {
    use super::{command_from_str, prompt, Command, RandType};
    use blackjack::deck::{Card, Rank, Suit};
    use blackjack::hand::Hand;
    use blackjack::playstats::PlayStats;
    use blackjack::table::Resp;
    const SUIT: Suit = Suit::Club;
    const RANDTYPE: RandType = RandType::Uniform;

    fn get_hand() -> Hand {
        Hand::new(&[Card::new(Rank::R2, SUIT), Card::new(Rank::R3, SUIT)])
    }

    fn get_card() -> Card {
        Card::new(Rank::R4, SUIT)
    }

    fn get_stats() -> PlayStats {
        PlayStats::new()
    }

    fn prompt_with(stdin: &str) -> Command {
        prompt(
            &get_hand(),
            get_card(),
            RANDTYPE,
            get_stats(),
            &mut stdin.as_bytes(),
            &mut vec![],
        )
        .unwrap()
    }

    #[test]
    fn prompt_empty_eventually() {
        // eventually finds command even if lots of leading whitespace
        let s = "\n\n    \n  s   \n\n";
        assert_eq!(prompt_with(s), Command::Resp(Resp::Stand));
        let s = "    quit        ";
        assert_eq!(prompt_with(s), Command::Quit);
    }

    #[test]
    fn double() {
        for s in &["d", "D"] {
            assert_eq!(command_from_str(s), Some(Command::Resp(Resp::Double)));
        }
    }

    #[test]
    fn split() {
        for s in &["p", "P"] {
            assert_eq!(command_from_str(s), Some(Command::Resp(Resp::Split)));
        }
    }

    #[test]
    fn hit() {
        for s in &["h", "H"] {
            assert_eq!(command_from_str(s), Some(Command::Resp(Resp::Hit)));
        }
    }

    #[test]
    fn stand() {
        for s in &["s", "S"] {
            assert_eq!(command_from_str(s), Some(Command::Resp(Resp::Stand)));
        }
    }

    #[test]
    fn quit() {
        for s in &["quit", "qUIt", "Quit"] {
            assert_eq!(command_from_str(s), Some(Command::Quit));
        }
    }

    #[test]
    fn save() {
        assert_eq!(command_from_str("save"), Some(Command::Save));
    }

    #[test]
    fn savequit() {
        assert_eq!(command_from_str("savequit"), Some(Command::SaveQuit));
        assert_eq!(command_from_str("save quit"), Some(Command::SaveQuit));
    }

    #[test]
    fn invalid_command_from_str() {
        for s in &["sace", "", "\n", " \n", " s", "s ", " s "] {
            assert!(command_from_str(s).is_none());
        }
    }
}
