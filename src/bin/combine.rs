use blackjack::playstats::PlayStats;
use blackjack::table::{dealer_card_from_desc, player_hand_from_desc, Table};
use blackjack::utils::{read_maybexz, write_maybexz};
use clap::{arg_enum, crate_authors, crate_name, crate_version, value_t, values_t, App, Arg};
use std::fs::OpenOptions;

arg_enum! {
    #[derive(PartialEq, Debug)]
    enum TableType {
        Stats,
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
                    let player = player_hand_from_desc(game_desc)?;
                    let dealer = dealer_card_from_desc(game_desc)?;
                    let mut agg_entry = agg.get(&player, dealer)?;
                    agg_entry.inc_by(val.correct(), true);
                    agg_entry.inc_by(val.seen() - val.correct(), false);
                    agg.update(&player, dealer, agg_entry)?;
                }
            }
            let out_fname = value_t!(matches, "output", String)?;
            let out = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&out_fname)?;
            eprintln!("Writing {}", out_fname);
            write_maybexz(out, &agg, out_fname.ends_with(".xz"))?;
        }
    }
    Ok(())
}
