use blackjack::deck::{Card, Rank, Suit};
use blackjack::hand::{Hand, HandType};
use blackjack::playstats::PlayStats;
use blackjack::table::{GameDesc, Table};
use blackjack::utils::{read_maybexz, write_maybexz};
use clap::{arg_enum, crate_authors, crate_name, crate_version, value_t, values_t, App, Arg};
use std::fs::OpenOptions;

arg_enum! {
    #[derive(PartialEq, Debug)]
    enum TableType {
        Stats,
    }
}

fn player_hand(desc: GameDesc) -> Hand {
    let s1 = Suit::Club;
    let s2 = Suit::Club;
    match desc.hand {
        HandType::Hard => match desc.player {
            5 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R3, s2)]),
            6 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R4, s2)]),
            7 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R5, s2)]),
            8 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R6, s2)]),
            9 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R7, s2)]),
            10 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R8, s2)]),
            11 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R9, s2)]),
            12 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::RT, s2)]),
            13 => Hand::new(&[Card::new(Rank::R3, s1), Card::new(Rank::RT, s2)]),
            14 => Hand::new(&[Card::new(Rank::R4, s1), Card::new(Rank::RT, s2)]),
            15 => Hand::new(&[Card::new(Rank::R5, s1), Card::new(Rank::RT, s2)]),
            16 => Hand::new(&[Card::new(Rank::R6, s1), Card::new(Rank::RT, s2)]),
            17 => Hand::new(&[Card::new(Rank::R7, s1), Card::new(Rank::RT, s2)]),
            18 => Hand::new(&[Card::new(Rank::R8, s1), Card::new(Rank::RT, s2)]),
            19 => Hand::new(&[Card::new(Rank::R9, s1), Card::new(Rank::RT, s2)]),
            20 => Hand::new(&[
                Card::new(Rank::RT, s1),
                Card::new(Rank::R8, s2),
                Card::new(Rank::R2, s2),
            ]),
            21 => Hand::new(&[
                Card::new(Rank::RT, s1),
                Card::new(Rank::R9, s2),
                Card::new(Rank::R2, s2),
            ]),
            _ => unreachable!(),
        },
        HandType::Soft => {
            let a = Card::new(Rank::RA, s1);
            match desc.player {
                13 => Hand::new(&[a, Card::new(Rank::R2, s2)]),
                14 => Hand::new(&[a, Card::new(Rank::R3, s2)]),
                15 => Hand::new(&[a, Card::new(Rank::R4, s2)]),
                16 => Hand::new(&[a, Card::new(Rank::R5, s2)]),
                17 => Hand::new(&[a, Card::new(Rank::R6, s2)]),
                18 => Hand::new(&[a, Card::new(Rank::R7, s2)]),
                19 => Hand::new(&[a, Card::new(Rank::R8, s2)]),
                20 => Hand::new(&[a, Card::new(Rank::R9, s2)]),
                21 => Hand::new(&[a, Card::new(Rank::RT, s2)]),
                _ => unreachable!(),
            }
        }
        HandType::Pair => match desc.player {
            4 => Hand::new(&[Card::new(Rank::R2, s1), Card::new(Rank::R2, s2)]),
            6 => Hand::new(&[Card::new(Rank::R3, s1), Card::new(Rank::R3, s2)]),
            8 => Hand::new(&[Card::new(Rank::R4, s1), Card::new(Rank::R4, s2)]),
            10 => Hand::new(&[Card::new(Rank::R5, s1), Card::new(Rank::R5, s2)]),
            12 => Hand::new(&[Card::new(Rank::R6, s1), Card::new(Rank::R6, s2)]),
            14 => Hand::new(&[Card::new(Rank::R7, s1), Card::new(Rank::R7, s2)]),
            16 => Hand::new(&[Card::new(Rank::R8, s1), Card::new(Rank::R8, s2)]),
            18 => Hand::new(&[Card::new(Rank::R9, s1), Card::new(Rank::R9, s2)]),
            20 => Hand::new(&[Card::new(Rank::RT, s1), Card::new(Rank::RT, s2)]),
            22 => Hand::new(&[Card::new(Rank::RA, s1), Card::new(Rank::RA, s2)]),
            _ => unreachable!(),
        },
    }
}

fn dealer_card(desc: GameDesc) -> Card {
    let s = Suit::Club;
    match desc.dealer {
        2 => Card::new(Rank::R2, s),
        3 => Card::new(Rank::R3, s),
        4 => Card::new(Rank::R4, s),
        5 => Card::new(Rank::R5, s),
        6 => Card::new(Rank::R6, s),
        7 => Card::new(Rank::R7, s),
        8 => Card::new(Rank::R8, s),
        9 => Card::new(Rank::R9, s),
        10 => Card::new(Rank::RT, s),
        11 => Card::new(Rank::RA, s),
        _ => unreachable!(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(String::from(crate_name!()) + " combine")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("type")
                .short("t")
                .long("t")
                .help("Parse files as this format of Table")
                .possible_values(&TableType::variants())
                .default_value("Stats")
                .case_insensitive(true),
        )
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .help("One or more files to read, each containing a Table")
                .required(true)
                .min_values(1)
                .multiple(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("File to which to write combined Tables")
                .default_value("/dev/stdout"),
        )
        .get_matches();
    let inputs = values_t!(matches, "input", String)?;
    match value_t!(matches, "type", TableType)? {
        TableType::Stats => {
            let mut agg = Table::new();
            agg.fill(std::iter::repeat(PlayStats::new()).take(360))?;
            for in_fname in inputs {
                eprintln!("Reading {}", in_fname);
                let fd = OpenOptions::new().read(true).open(&in_fname)?;
                let t: Table<PlayStats> = read_maybexz(fd, in_fname.ends_with(".xz"))?;
                for (game_desc, val) in t.iter() {
                    let player = player_hand(game_desc);
                    let dealer = dealer_card(game_desc);
                    let mut agg_entry = agg.get(&player, dealer)?;
                    agg_entry.inc_by(val.correct(), true);
                    agg_entry.inc_by(val.seen() - val.correct(), false);
                    agg.update(&player, dealer, agg_entry)?;
                }
            }
            let out_fname = value_t!(matches, "output", String)?;
            let out = OpenOptions::new().write(true).create(true).open(&out_fname)?;
            eprintln!("Writing {}", out_fname);
            write_maybexz(out, &agg, out_fname.ends_with(".xz"))?;
        }
    }
    Ok(())
}
