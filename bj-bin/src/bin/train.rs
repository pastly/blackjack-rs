use bj_bin::prompt;
use bj_bin::utils::{create_if_not_exist, read_maybexz, write_maybexz};
use bj_core::deck::{rand_suit, Card, Deck, Rank};
use bj_core::hand::{rand_hand, Hand};
use bj_core::playstats::PlayStats;
use bj_core::table::{resps_from_buf, GameDesc, Table};
use clap::{arg_enum, crate_authors, crate_name, crate_version, value_t, App, Arg};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};

fn def_playstats_table() -> Table<PlayStats> {
    const NUM_CELLS: usize = 10 * (17 + 9 + 10);
    Table::new(vec![PlayStats::new(); NUM_CELLS]).unwrap()
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
) -> io::Result<prompt::Command> {
    let s = &format!(
        "({} {}/{}) {} / {}",
        rand_type,
        stat.correct(),
        stat.seen(),
        p,
        d
    );
    loop {
        match prompt::prompt(s, in_buf, out_buf)? {
            prompt::Command::Bet(_) => {
                writeln!(out_buf, "Cannot bet")?;
                continue;
            }
            cmd => break Ok(cmd),
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
            .default_value("10")
            .value_name("CNT")
            .help("Every CNT rolls, generate hand uniformally at random as opposed to weighted by play statistics. 0 means never, 1 means always.")
        )
        .get_matches();
    let mut deck = Deck::new_infinite();
    let table = Table::new(resps_from_buf(
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
    let mut streak_count = 0;
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
            prompt::Command::Quit => return Ok(()),
            prompt::Command::Save | prompt::Command::SaveQuit => {
                // This saves play stats and restarts the loop, which means it acts like this hand
                // never happened. This gives the player a way to skip a hand without consequences.
                let fd = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(stats_fname)?;
                write_maybexz(fd, &stats, stats_fname.ends_with(".xz"))?;
                print_game_stats(&stats);
                if command == prompt::Command::SaveQuit {
                    return Ok(());
                }
                continue;
            }
            prompt::Command::Bet(_) => unreachable!(),
            prompt::Command::Num(_) => unreachable!(),
            prompt::Command::Resp(_) => { /* will handle below */ }
        };
        let resp = if let prompt::Command::Resp(r) = command {
            r
        } else {
            unreachable!("Should have handled non-Command::Resp already");
        };
        // Handle the case that the user actually hit, stand, etc.
        let best = table.get(&player, dealer_up)?;
        print!("{} ", resp);
        if resp == best {
            streak_count += 1;
            println!("correct");
        } else {
            println!("wrong. Should {}. Streak was {}", best, streak_count);
            streak_count = 0;
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
