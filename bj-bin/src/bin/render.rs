use bj_core::table::{resps_from_buf, Resp, Table};
use clap::{crate_authors, crate_name, crate_version, App, Arg};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

fn header(mut fd: impl Write) -> Result<(), Box<dyn Error>> {
    writeln!(
        fd,
        "
<style>
.hit, .stand, .double, .split {{
    width:  1.5em;
    height: 1.5em;
    text-align: center;
}}
.hit    {{ background-color: #ff3333; }}
.stand  {{ background-color: #ffff00; }}
.double {{ background-color: #6666ff; }}
.split  {{ background-color: #00ff00; }}
</style>
"
    )?;
    Ok(())
}

fn footer(mut fd: impl Write) -> Result<(), Box<dyn Error>> {
    writeln!(
        fd,
        "
<span class=hit>H</span>Hit<br/>
<span class=stand>S</span>Stand<br/>
<span class=double>D</span>Double<br/>
<span class=split>P</span>Split<br/>
"
    )?;
    Ok(())
}

fn render_table_html(mut fd: impl Write, table: Table<Resp>) -> Result<(), Box<dyn Error>> {
    header(&mut fd)?;
    let (hards, softs, pairs) = table.into_values_sorted();
    inner(&mut fd, hards, "Hard")?;
    inner(&mut fd, softs, "Soft")?;
    inner(&mut fd, pairs, "Pair")?;
    footer(&mut fd)?;

    fn inner(mut fd: impl Write, v: Vec<Resp>, label: &str) -> Result<(), Box<dyn Error>> {
        let mut player_hand_val = match label {
            "Hard" => 5,
            "Soft" => 13,
            "Pair" => 2,
            _ => unreachable!("Impossible label"),
        };
        writeln!(fd, "<h1>{}</h1><table>", label)?;
        write!(fd, "<tr><td></td>")?;
        for i in 2..=11 {
            let s = val_to_string(i);
            writeln!(fd, "<th>{}</th>", s)?;
        }
        write!(fd, "</tr><tr>")?;
        for (i, resp) in v.iter().enumerate() {
            if i % 10 == 0 {
                let s = val_to_string(player_hand_val);
                writeln!(fd, "<th>{}</th>", s)?;
                player_hand_val += 1;
            }
            let (class, label) = match resp {
                Resp::Hit => ("hit", "H"),
                Resp::Stand => ("stand", "S"),
                Resp::Double => ("double", "D"),
                Resp::Split => ("split", "P"),
            };
            writeln!(fd, "<td class={}>{}</td>", class, label)?;
            if i % 10 == 9 {
                writeln!(fd, "</tr><tr>")?;
            }
        }
        writeln!(fd, "</tr></table>")?;
        Ok(())
    }

    fn val_to_string(v: u8) -> String {
        match v {
            11 => "A".to_string(),
            v => v.to_string(),
        }
    }
    Ok(())
}

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
    let table = Table::new(resps_from_buf(
        OpenOptions::new()
            .read(true)
            // safe to unwrap because --input is required
            .open(matches.value_of("input").unwrap())?,
    ))?;
    let mut fd = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            // safe to unwrap because --output is required
            .open(matches.value_of("output").unwrap())?,
    );
    render_table_html(&mut fd, table)?;
    fd.flush()?;
    Ok(())
}
