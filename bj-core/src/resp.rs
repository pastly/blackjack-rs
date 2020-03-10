use crate::table::NUM_CELLS;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::{self, Read};

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub enum Resp {
    Hit,
    Stand,
    DoubleElseHit,
    DoubleElseStand,
    Split,
    SurrenderElseHit,
    SurrenderElseStand,
    SurrenderElseSplit,
}

#[derive(Debug)]
pub enum RespError {
    IOError(io::Error),
    InvalidChar(char),
    InvalidCharSequence(String),
    NoSecondChar(char),
}

impl std::error::Error for RespError {}

impl fmt::Display for RespError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IOError(e) => write!(f, "io error: {}", e),
            Self::InvalidChar(c) => write!(f, "Invalid char '{}'", c),
            Self::InvalidCharSequence(s) => write!(f, "'{}' is not a valid Resp", s),
            Self::NoSecondChar(c) => {
                write!(f, "'{}' requires second char, but it doesn't exist", c)
            }
        }
    }
}

impl From<io::Error> for RespError {
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

impl fmt::Display for Resp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hit => write!(f, "Hit"),
            Self::Stand => write!(f, "Stand"),
            Self::DoubleElseHit => write!(f, "Double(Hit)"),
            Self::DoubleElseStand => write!(f, "Double(Stand)"),
            Self::Split => write!(f, "Split"),
            Self::SurrenderElseHit => write!(f, "Surrender(Hit)"),
            Self::SurrenderElseStand => write!(f, "Surrender(Stand)"),
            Self::SurrenderElseSplit => write!(f, "Surrender(Split)"),
        }
    }
}

struct RespCharIter(Vec<char>);

impl RespCharIter {
    // all valid first chars (some of which can be followed by a 2nd), and all valid second chars
    const ALL_FIRST: [char; 5] = ['H', 'S', 'P', 'D', 'R'];
    const ALL_SECOND: [char; 3] = ['h', 's', 'p'];
    // first chars that are either always by themself (SINGLE) or always followed by a second char
    // (DOUBLE)
    //const ALWAYS_SINGLE: [char; 3] = ['H', 'S', 'P'];
    const ALWAYS_DOUBLE: [char; 2] = ['D', 'R'];
    fn new(s: String) -> Self {
        Self(s.chars().rev().collect())
    }
}

impl Iterator for RespCharIter {
    type Item = Result<Resp, RespError>;
    fn next(&mut self) -> Option<Self::Item> {
        let c = match self.0.pop() {
            None => {
                return None;
            }
            Some(c) => c,
        };
        // make sure char is allowed in this position
        if !Self::ALL_FIRST.contains(&c) {
            return Some(Err(RespError::InvalidChar(c)));
        }
        let mut s = String::with_capacity(2);
        s.push(c);
        // if this char requires a second one, check that we even have a second one. If we do, take
        // it and add it to the string
        if Self::ALWAYS_DOUBLE.contains(&c) {
            let c2 = match self.0.pop() {
                None => {
                    return Some(Err(RespError::NoSecondChar(c)));
                }
                Some(c) => c,
            };
            if !Self::ALL_SECOND.contains(&c2) {
                return Some(Err(RespError::InvalidChar(c)));
            }
            s.push(c2);
        }
        match s.as_str() {
            "H" => Some(Ok(Resp::Hit)),
            "S" => Some(Ok(Resp::Stand)),
            "P" => Some(Ok(Resp::Split)),
            "Dh" => Some(Ok(Resp::DoubleElseHit)),
            "Ds" => Some(Ok(Resp::DoubleElseStand)),
            "Rh" => Some(Ok(Resp::SurrenderElseHit)),
            "Rs" => Some(Ok(Resp::SurrenderElseStand)),
            "Rp" => Some(Ok(Resp::SurrenderElseSplit)),
            _ => Some(Err(RespError::InvalidCharSequence(s))),
        }
    }
}

/// Take something that impls Read, strip out comments ('#' until end of line), ignore everything
/// that isn't in "HSDP" (hit, stand, double, split), and return a vector of these parsed
/// Vec<Resp>.
pub fn resps_from_buf<R>(buf: R) -> Result<Vec<Resp>, RespError>
where
    R: Read,
{
    use readfilter::{CharWhitelist, CommentStrip};
    let mut buf = CharWhitelist::new(CommentStrip::new(buf), "HSDPRhsp");
    let mut s = String::with_capacity(NUM_CELLS);
    buf.read_to_string(&mut s)?;
    RespCharIter::new(s).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::{once, repeat};

    #[test]
    fn count_doesnt_matter() {
        // it's not resps_from_buf's job to return the right number of elements for a Table. So it
        // won't fail with too few or too many
        for num in 0..NUM_CELLS + 50 {
            let res = resps_from_buf(repeat('H').take(num).collect::<String>().as_bytes());
            assert!(res.is_ok());
        }
    }

    #[test]
    fn invalid_single_char() {
        // an invalid char somewhere in the buffer causes failure
        let s = repeat('H')
            .take(99)
            .chain(once('h'))
            .chain(repeat('S').take(260))
            .collect::<String>();
        let buf = s.as_bytes();
        // sanity check
        assert_eq!(buf.len(), NUM_CELLS);
        // actual test
        if let Err(e) = resps_from_buf(buf) {
            if let RespError::InvalidChar(c) = e {
                assert_eq!(c, 'h');
            } else {
                panic!("Should have been different bad char");
            }
        } else {
            panic!("Should not have been ok");
        }
    }

    #[test]
    fn empty() {
        let s = String::new();
        let buf = s.as_bytes();
        if let Ok(resps) = resps_from_buf(buf) {
            assert!(resps.is_empty());
        } else {
            panic!("Should have been ok");
        }
    }

    #[test]
    fn no_second_char() {
        // a two-char sequence that ends without the second char is an error
        for prefix in ["", "H", "HH"].iter() {
            // c is the char that requires a second char after it
            for c in ['D', 'R'].iter() {
                let mut s = prefix.to_string();
                s.push(*c);
                let buf = s.as_bytes();
                let ret = resps_from_buf(buf);
                if let Err(RespError::NoSecondChar(ret_c)) = ret {
                    assert_eq!(ret_c, *c);
                } else {
                    panic!("Should have NoSecondChar error");
                }
            }
        }
    }

    #[test]
    fn bad_first_char() {
        // check that the 2nd chars htat indicates variants (e.g. 'h' in 'Dh' indicating hit if you
        // can't double) are correctly refused when they show up first. prefix is a leading set of
        // unrelated chars. c is the actual problematic char.
        for prefix in ["", "H", "HH"].iter() {
            for c in ['h', 's', 'p'].iter() {
                let mut s = prefix.to_string();
                s.push(*c);
                let buf = s.as_bytes();
                let ret = resps_from_buf(buf);
                if let Err(RespError::InvalidChar(ret_c)) = ret {
                    assert_eq!(ret_c, *c);
                } else {
                    panic!("Should have InvalidChar error");
                }
            }
        }
    }
}
