use bj_core::basicstrategy::BasicStrategy;
use bj_core::rendertable::{HTMLTableRenderer, HTMLTableRendererOpts, TXTTableRenderer};
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
        .arg(
            Arg::with_name("format")
                .short("f")
                .long("format")
                .takes_value(true)
                .possible_values(&["html", "txt"])
                .required(true),
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
    let html_opts = HTMLTableRendererOpts {
        incl_bs_rules: true,
    };
    match matches.value_of("format").unwrap() {
        "html" => HTMLTableRenderer::render(&mut fd, &bs_card, html_opts)?,
        "txt" => TXTTableRenderer::render(&mut fd, &bs_card)?,
        _ => unimplemented!(),
    };
    fd.flush()?;
    Ok(())
}
