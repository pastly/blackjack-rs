use bj_core::resp::Resp;
use std::io::{self, BufRead, Write};

type NumType = i32;
type BetType = u32;

#[derive(Debug, PartialEq)]
pub enum Command {
    Quit,
    Save,
    SaveQuit,
    Num(NumType),
    Bet(BetType),
    Resp(Resp),
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Quit => write!(f, "Quit"),
            Command::Save => write!(f, "Save"),
            Command::SaveQuit => write!(f, "SaveQuit"),
            Command::Num(val) => write!(f, "Num({})", val),
            Command::Bet(amt) => write!(f, "Bet({})", amt),
            Command::Resp(r) => write!(f, "Resp({})", r),
        }
    }
}

fn command_from_str(s: &str) -> Option<Command> {
    let s: &str = &s.to_ascii_uppercase();
    let words: Vec<_> = s.split_whitespace().collect();
    if words.is_empty() {
        None
    } else if words.len() == 1 {
        if words[0] == "QUIT" {
            Some(Command::Quit)
        } else if words[0] == "SAVE" {
            Some(Command::Save)
        } else if words[0] == "SAVEQUIT" {
            Some(Command::SaveQuit)
        } else if let Ok(val) = words[0].parse::<NumType>() {
            Some(Command::Num(val))
        } else {
            match words[0] {
                "H" => Some(Command::Resp(Resp::Hit)),
                "S" => Some(Command::Resp(Resp::Stand)),
                // Double doesn't exist, so just do DoubleElseHit
                "D" => Some(Command::Resp(Resp::DoubleElseHit)),
                "P" => Some(Command::Resp(Resp::Split)),
                _ => None,
            }
        }
    } else if words.len() == 2 {
        if words[0] == "BET" || words[0] == "B" {
            if let Ok(amt) = words[1].parse::<BetType>() {
                Some(Command::Bet(amt))
            } else {
                None
            }
        } else if words[0] == "SAVE" && words[1] == "QUIT" {
            Some(Command::SaveQuit)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn prompt(s: &str, in_buf: &mut impl BufRead, out_buf: &mut impl Write) -> io::Result<Command> {
    loop {
        write!(out_buf, "{} > ", s)?;
        out_buf.flush()?;
        let mut s = String::new();
        in_buf.read_line(&mut s)?;
        s = s.trim().to_string();
        if s.is_empty() {
            continue;
        }
        if let Some(cmd) = command_from_str(&s) {
            break Ok(cmd);
        } else {
            writeln!(out_buf, "Bad response: {}", s)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{command_from_str, prompt, Command};
    use bj_core::resp::Resp;

    fn prompt_with(stdin: &str) -> Command {
        prompt("", &mut stdin.as_bytes(), &mut vec![]).unwrap()
    }

    #[test]
    fn prompt_empty_eventually() {
        // eventually finds command even if lots of leading whitespace
        let s = "\n\n    \n  s   \n\n";
        assert_eq!(prompt_with(s), Command::Resp(Resp::Stand));
        let s = "    quit        ";
        assert_eq!(prompt_with(s), Command::Quit);
    }

    #[test]
    fn double() {
        for s in &["d", "D"] {
            assert_eq!(
                command_from_str(s),
                Some(Command::Resp(Resp::DoubleElseHit))
            );
        }
    }

    #[test]
    fn split() {
        for s in &["p", "P"] {
            assert_eq!(command_from_str(s), Some(Command::Resp(Resp::Split)));
        }
    }

    #[test]
    fn hit() {
        for s in &["h", "H"] {
            assert_eq!(command_from_str(s), Some(Command::Resp(Resp::Hit)));
        }
    }

    #[test]
    fn stand() {
        for s in &["s", "S"] {
            assert_eq!(command_from_str(s), Some(Command::Resp(Resp::Stand)));
        }
    }

    #[test]
    fn quit() {
        for s in &["quit", "qUIt", "Quit"] {
            assert_eq!(command_from_str(s), Some(Command::Quit));
        }
    }

    #[test]
    fn save() {
        assert_eq!(command_from_str("save"), Some(Command::Save));
    }

    #[test]
    fn savequit() {
        assert_eq!(command_from_str("savequit"), Some(Command::SaveQuit));
        assert_eq!(command_from_str("save quit"), Some(Command::SaveQuit));
    }

    #[test]
    fn bet() {
        assert_eq!(command_from_str("bet 1"), Some(Command::Bet(1)));
        assert_eq!(command_from_str("b 1"), Some(Command::Bet(1)));
    }

    #[test]
    fn invalid_command_from_str() {
        for s in &[
            "sace",
            "",
            "\n",
            " \n",
            "save bet",
            "bet",
            "bet foo",
            "bet 10 10",
        ] {
            assert!(command_from_str(s).is_none());
        }
    }
}
