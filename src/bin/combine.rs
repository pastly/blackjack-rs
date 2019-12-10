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

fn agg_tables<T>(
    mut agg: Table<T>,
    tables: impl Iterator<Item = Table<T>>,
) -> Result<Table<T>, Box<dyn std::error::Error>>
where
    T: PartialEq + Copy + std::ops::AddAssign,
{
    for t in tables {
        for (game_desc, val) in t.iter() {
            let player = player_hand_from_desc(game_desc)?;
            let dealer = dealer_card_from_desc(game_desc)?;
            let mut agg_entry = agg.get(&player, dealer)?;
            agg_entry += *val;
            agg.update(&player, dealer, agg_entry)?;
        }
    }
    Ok(agg)
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
            // create empty starting table
            let mut agg = Table::new();
            agg.fill(std::iter::repeat(PlayStats::new()).take(360))?;
            // for each input
            // - try to open it (fail early and break out of the iter if we can't)
            // - try reading it (fail early [...] if we can't)
            // - aggregate it into the accumulator table
            // and if all goes succesfully, put final accumulated table in agg
            agg = inputs.into_iter().try_fold(agg, |acc, fname| {
                eprintln!("Reading {}", fname);
                let fd = OpenOptions::new().read(true).open(&fname)?;
                agg_tables(
                    acc,
                    vec![read_maybexz(fd, fname.ends_with(".xz"))?].into_iter(),
                )
            })?;
            // try writing out result
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
