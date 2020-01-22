use crate::table::{Resp, Table};
use std::io::{self, Write};

pub trait TableRenderer {
    fn render(fd: impl Write, table: Table<Resp>) -> io::Result<()>;
}

pub struct HTMLTableRenderer;

impl HTMLTableRenderer {
    fn header(mut fd: impl Write) -> io::Result<()> {
        writeln!(
            fd,
            "
<style>
#strat_html table tr td {{
}}
.hit, .stand, .double, .split {{
    width:  1.5em;
    height: 1.5em;
    text-align: center;
    color: #333;
}}
.hit    {{ background-color: #ff3333; }}
.stand  {{ background-color: #ffff00; }}
.double {{ background-color: #6666ff; }}
.split  {{ background-color: #00ff00; }}
</style>
"
        )
    }

    fn footer(mut fd: impl Write) -> io::Result<()> {
        writeln!(
            fd,
            "
<span class=hit>H</span>Hit<br/>
<span class=stand>S</span>Stand<br/>
<span class=double>D</span>Double<br/>
<span class=split>P</span>Split<br/>
"
        )
    }

    fn subtable(mut fd: impl Write, v: Vec<Resp>, label: &str) -> io::Result<()> {
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
        return Ok(());

        fn val_to_string(v: u8) -> String {
            match v {
                11 => "A".to_string(),
                v => v.to_string(),
            }
        }
    }
}

impl TableRenderer for HTMLTableRenderer {
    fn render(mut fd: impl Write, table: Table<Resp>) -> io::Result<()> {
        let (hards, softs, pairs) = table.into_values_sorted();
        HTMLTableRenderer::header(&mut fd)?;
        HTMLTableRenderer::subtable(&mut fd, hards, "Hard")?;
        HTMLTableRenderer::subtable(&mut fd, softs, "Soft")?;
        HTMLTableRenderer::subtable(&mut fd, pairs, "Pair")?;
        HTMLTableRenderer::footer(&mut fd)?;
        Ok(())
    }
}
