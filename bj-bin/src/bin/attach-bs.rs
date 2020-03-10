use bj_core::basicstrategy::rules;
use bj_core::basicstrategy::BasicStrategy;
use bj_core::resp::resps_from_buf;
use bj_core::table::Table;
use clap::{crate_authors, crate_name, crate_version, App, Arg};
use serde_json;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;

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
        .get_matches();
    let table = Table::new(resps_from_buf(
        OpenOptions::new()
            .read(true)
            // safe to unwrap because --input is required
            .open(matches.value_of("input").unwrap())?,
    )?)?;
    let fd =
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            // safe to unwrap because --output is required
            .open(matches.value_of("output").unwrap())?;
    let rules = rules::Rules {
        decks: rules::NumDecks::FourPlus,
        double_after_split: true,
        hit_soft_17: true,
        peek_bj: true,
        surrender: rules::Surrender::No,
    };
    let bs: BasicStrategy = (rules, table).into();
    serde_json::to_writer(fd, &bs)?;
    //fd.flush()?;
    Ok(())
}
