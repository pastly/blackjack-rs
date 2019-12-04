use blackjack::deck::{Card, Deck};
use blackjack::hand::Hand;
use blackjack::playstats::PlayStats;
use blackjack::table::{resp_from_char, resps_from_buf, Resp, Table};
use clap::{arg_enum, crate_authors, crate_name, crate_version, value_t, App, Arg};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
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
    //for item in stats.iter() {
    //    println!("{:?}", item);
    //}
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
            if let Err(e) = write_maybexz(fd, data, fname.ends_with(".xz")) {
                return Err(Box::new(e));
            }
            Ok(())
        }
        Err(e) => {
            // unable to create the file, and that might be because it already exists.
            // ignore errors from it already existing, but bubble up all others
            match e.kind() {
                io::ErrorKind::AlreadyExists => Ok(()),
                _ => Err(Box::new(e)),
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
    let matches = App::new(crate_name!())
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
                .default_value("play-stats.json"),
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
        let player = Hand::new(&[deck.draw()?, deck.draw()?]);
        let dealer_up = deck.draw()?;
        if let Some(choice) = prompt(&player, dealer_up)? {
            let best = table.get(&player, dealer_up)?;
            print!("{} ", choice);
            if choice == best {
                println!("correct");
            } else {
                println!("wrong. Should {}", best);
                print_game_stats(&stats);
            }
            let mut stat = stats.get(&player, dealer_up)?;
            stat.inc(choice == best);
            stats.update(&player, dealer_up, stat)?;
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
