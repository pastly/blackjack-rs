use bj_core::basicstrategy::rules;
use bj_core::basicstrategy::BasicStrategy;
use bj_core::resp::resps_from_buf;
use bj_core::table::Table;
use clap::{crate_authors, crate_name, crate_version, App, Arg};
use std::error::Error;
use std::fs::OpenOptions;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new(String::from(crate_name!()) + " attach-bs")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("STRAT_CARD")
                .default_value("/dev/stdin"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("STRAT_CARD")
                .default_value("/dev/stdout"),
        )
        .arg(
            Arg::with_name("decks")
                .long("decks")
                .value_name("NUM")
                .required(true)
                .takes_value(true)
                .possible_values(&["1", "2", "3", "4+"]),
        )
        .arg(
            Arg::with_name("surrender")
                .long("surrender")
                .required(true)
                .takes_value(true)
                .possible_values(&["yes", "no", "notace"]),
        )
        .arg(
            Arg::with_name("das")
                .long("double-after-split")
                .required(true)
                .takes_value(true)
                .possible_values(&["yes", "no"]),
        )
        .arg(
            Arg::with_name("hit17")
                .long("hit-soft-17")
                .required(true)
                .takes_value(true)
                .possible_values(&["yes", "no"]),
        )
        .arg(
            Arg::with_name("peek")
                .long("peek-bj")
                .required(true)
                .takes_value(true)
                .possible_values(&["yes", "no"]),
        )
        .get_matches();
    let table = Table::new(resps_from_buf(
        OpenOptions::new()
            .read(true)
            // safe to unwrap because --input is required
            .open(matches.value_of("input").unwrap())?,
    )?)?;
    let decks = match matches.value_of("decks").unwrap() {
        "1" => rules::NumDecks::One,
        "2" => rules::NumDecks::Two,
        "3" => rules::NumDecks::Three,
        "4+" => rules::NumDecks::FourPlus,
        _ => panic!("Impossible decks"),
    };
    let surrender = match matches.value_of("surrender").unwrap() {
        "no" => rules::Surrender::No,
        "yes" => rules::Surrender::Yes,
        "notace" => rules::Surrender::NotAce,
        _ => panic!("Impossible surrender"),
    };
    let das = match matches.value_of("das").unwrap() {
        "no" => false,
        "yes" => true,
        _ => panic!("Impossible das"),
    };
    let hit17 = match matches.value_of("hit17").unwrap() {
        "no" => false,
        "yes" => true,
        _ => panic!("Impossible hit17"),
    };
    let peek = match matches.value_of("peek").unwrap() {
        "no" => false,
        "yes" => true,
        _ => panic!("Impossible peek"),
    };
    let rules = rules::Rules {
        decks,
        double_after_split: das.into(),
        hit_soft_17: hit17.into(),
        peek_bj: peek.into(),
        surrender,
    };
    let bs: BasicStrategy = (Some(rules), table).into();
    let fd = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(matches.value_of("output").unwrap())?;
    serde_json::to_writer(fd, &bs)?;
    Ok(())
}
