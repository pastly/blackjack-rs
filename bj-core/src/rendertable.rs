use crate::table::{Resp, Table};
use std::io::{self, Write};

pub trait TableRenderer {
    fn render(fd: impl Write, table: Table<Resp>) -> io::Result<()>;
}

pub struct HTMLTableRendererCSSOptions {
    col_table_outer_text: String,
    col_table_inner_text: String,
    col_hit: String,
    col_stand: String,
    col_double: String,
    col_split: String,
}

impl std::default::Default for HTMLTableRendererCSSOptions {
    fn default() -> Self {
        Self {
            col_table_outer_text: "#ddd".to_owned(),
            col_table_inner_text: "#333".to_owned(),
            col_hit: "#ff3333".to_owned(),
            col_stand: "#ffff00".to_owned(),
            col_double: "#6666ff".to_owned(),
            col_split: "#00ff00".to_owned(),
        }
    }
}

pub struct HTMLTableRenderer;

impl HTMLTableRenderer {
    fn header(mut _fd: impl Write) -> io::Result<()> {
        Ok(())
    }

    fn footer(mut fd: impl Write) -> io::Result<()> {
        writeln!(
            fd,
            "
<span class=strat_card_hit>H</span>&nbsp;Hit<br/>
<span class=strat_card_stand>S</span>&nbsp;Stand<br/>
<span class=strat_card_double>D</span>&nbsp;Double<br/>
<span class=strat_card_split>P</span>&nbsp;Split<br/>
Source: <a href='https://wizardofodds.com'>wizardofodds.com</a><br/>
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
                Resp::Hit => ("strat_card_hit", "H"),
                Resp::Stand => ("strat_card_stand", "S"),
                Resp::Double => ("strat_card_double", "D"),
                Resp::Split => ("strat_card_split", "P"),
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

    pub fn render_css(mut fd: impl Write, options: Option<HTMLTableRendererCSSOptions>) -> io::Result<()> {
        let options = if options.is_none() {
            std::default::Default::default()
        } else {
            options.unwrap()
        };
        writeln!(
            fd,
            "
<style>
.strat_card_hit, .strat_card_stand, .strat_card_double, .strat_card_split {{
    width:  1.5em;
    height: 1.5em;
    text-align: center;
}}
.strat_card_hit    {{ background-color: {col_hit}; }}
.strat_card_stand  {{ background-color: {col_stand}; }}
.strat_card_double {{ background-color: {col_double}; }}
.strat_card_split  {{ background-color: {col_split}; }}
a {{ color: inherit; }}
#strat_html table {{
    color: {col_table_outer_text};
}}
#strat_html table tr td {{
    color: {col_table_inner_text};
}}
</style>
",
            col_table_outer_text=options.col_table_outer_text,
            col_table_inner_text=options.col_table_inner_text,
            col_hit=options.col_hit,
            col_stand=options.col_stand,
            col_double=options.col_double,
            col_split=options.col_split,
        )
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
