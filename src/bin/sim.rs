use blackjack::deck::{rand_suit, Card, Deck, Rank};
use blackjack::hand::{rand_hand, Hand};
use blackjack::playstats::PlayStats;
use blackjack::table::{resp_from_char, resps_from_buf, GameDesc, Resp, Table};
use clap::{arg_enum, crate_authors, crate_name, crate_version, value_t, App, Arg};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Read, Write};
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;

fn def_playstats_table() -> Table<PlayStats> {
    const NUM_CELLS: usize = 10 * (17 + 9 + 10);
    let mut t = Table::new();
    t.fill(vec![PlayStats::new(); NUM_CELLS]).unwrap();
    t
}

fn write_maybexz<T>(fd: impl Write, data: &T, xz: bool) -> Result<(), serde_json::error::Error>
where
    T: Serialize,
{
    if xz {
        serde_json::to_writer(XzEncoder::new(fd, 9), &data)
    } else {
        serde_json::to_writer(fd, &data)
    }
}

fn read_maybexz<T>(fd: impl Read, xz: bool) -> Result<T, serde_json::error::Error>
where
    for<'de> T: Deserialize<'de>,
{
    if xz {
        serde_json::from_reader(XzDecoder::new(fd))
    } else {
        serde_json::from_reader(fd)
    }
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

enum RandType {
    Uniform,
    Weighted,
}

impl std::fmt::Display for RandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
) -> Result<Option<Resp>, io::Error> {
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
        let c: char = s.chars().take(1).collect::<Vec<char>>()[0].to_ascii_uppercase();
        let resp = resp_from_char(c);
        if resp.is_some() {
            return Ok(resp);
        } else {
            println!("Bad response: {}", c);
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
        if let Some(choice) = prompt(
            &player,
            dealer_up,
            rand_type,
            current_stat,
            &mut BufReader::new(io::stdin()),
            &mut io::stdout(),
        )? {
            let best = table.get(&player, dealer_up)?;
            print!("{} ", choice);
            if choice == best {
                println!("correct");
            } else {
                println!("wrong. Should {}", best);
            }
            let mut stat = stats.get(&player, dealer_up)?;
            stat.inc(choice == best);
            stats.update(&player, dealer_up, stat)?;
            if choice != best {
                print_game_stats(&stats);
            }
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
        } else {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{prompt, RandType};
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

    fn prompt_with(stdin: &str) -> Option<Resp> {
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
    fn double() {
        for s in &["d", "Dd", "dlj238gf"] {
            assert_eq!(prompt_with(s), Some(Resp::Double));
        }
    }

    #[test]
    fn split() {
        for s in &["p", "Pp", "plj238gf"] {
            assert_eq!(prompt_with(s), Some(Resp::Split));
        }
    }

    #[test]
    fn hit() {
        for s in &["h", "Hh", "hlj238gf"] {
            assert_eq!(prompt_with(s), Some(Resp::Hit));
        }
    }

    #[test]
    fn stand() {
        for s in &["s", "Ss", "slj238gf"] {
            assert_eq!(prompt_with(s), Some(Resp::Stand));
        }
    }

    #[test]
    fn empty_eventually() {
        // eventually finds command even if lots of leading whitespace
        let s = "\n\n    \n  s   \n\n";
        assert_eq!(prompt_with(s), Some(Resp::Stand));
    }
}
