use bj_core::basicstrategy::BasicStrategy;
use bj_core::rendertable::{HTMLTableRenderer, TableRenderer};
use clap::{crate_authors, crate_name, crate_version, App, Arg};
use serde_json;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new(String::from(crate_name!()) + " render")
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
    let bs_card: BasicStrategy = serde_json::from_reader(
        OpenOptions::new()
            .read(true)
            // safe to unwrap because --input is required
            .open(matches.value_of("input").unwrap())?,
    )?;
    let mut fd = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            // safe to unwrap because --output is required
            .open(matches.value_of("output").unwrap())?,
    );
    HTMLTableRenderer::render(&mut fd, &bs_card)?;
    fd.flush()?;
    Ok(())
}
