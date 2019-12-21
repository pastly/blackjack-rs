//! Keep track of how often player sees an event and chooses the correct response to that event.
//!
//! Example 1: opening player hand vs. dealer card --> did player choose hit/stand/etc. correctly
//! according to basic strategy.
//!
//! Example 2: presented with a set of cards --> did player calculate the HiLo count correctly.
//!
//! PlayStats calculates a weight for the event based on player's past performance.
//! It simply stores the number of seen and number of correct. Seen must always be equal to or
//! greater than correct.
//!
//! The weight is a value in (0, 1] and is only 1 exactly if seen is 0. Weight calculation is
//! `1 - correct / (seen + 1)`
//!
//! - Weight of untouched PlayStats is 1 = (1 - 0/1)
//! - Weight of PlayStats with 1 wrong is 1.0 = (1 - 0/2)
//! - Weight of PlayStats with 1 correct is 0.5 = (1 - 1/2)
//! - Weight of PlayStats with 2 correct is 0.333 = (1 - 2/3)
//! - Weight of PlayStats with 1 correct/1 wrong is 0.666 = (1 - 1/3)
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Copy, Clone, Default, Debug)]
pub struct PlayStats {
    seen: u16,
    correct: u16,
}

impl PlayStats {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn weight(self) -> f32 {
        // The +1 prevents division by zero. It also means that the inclusive side of the (0, 1)
        // range is at 1 instead of 0. Without the +1, getting a hand correct on the very first try
        // would result in 1 - 1/1 == 0. Zero weight means the player will never see it again. We
        // could add a very small amount to zero values to keep them non-zero, but I think adding 1
        // will create better weights that result in a better player experience.
        //
        // weight() will NOT equal correct() / seen()
        1f32 - self.correct as f32 / (self.seen + 1) as f32
    }

    pub fn inc(&mut self, correct: bool) {
        self.inc_by(1, correct)
    }

    pub fn inc_by(&mut self, amt: u16, correct: bool) {
        self.seen += amt;
        if correct {
            self.correct += amt;
        }
    }

    pub fn seen(self) -> u16 {
        self.seen
    }

    pub fn correct(self) -> u16 {
        self.correct
    }
}

impl std::ops::AddAssign for PlayStats {
    fn add_assign(&mut self, rhs: Self) {
        self.inc_by(rhs.correct(), true);
        #[allow(clippy::suspicious_op_assign_impl)]
        self.inc_by(rhs.seen() - rhs.correct(), false);
    }
}

impl std::ops::Add for PlayStats {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut new = Self::new();
        new.inc_by(self.correct(), true);
        new.inc_by(other.correct(), true);
        #[allow(clippy::suspicious_arithmetic_impl)]
        new.inc_by(self.seen() - self.correct(), false);
        #[allow(clippy::suspicious_arithmetic_impl)]
        new.inc_by(other.seen() - other.correct(), false);
        new
    }
}

#[cfg(test)]
mod tests {
    use super::PlayStats;
    const COUNT_MANY: usize = 10;

    #[test]
    fn weight_one() {
        // brand new stats have weight of 1. primarily a test to avoid NaN from 0 rolls seen
        assert_eq!(PlayStats::new().weight(), 1f32);
    }

    #[test]
    fn weight_half() {
        // getting half correct doesn't result in 0.5. It gets closer to 0.5 the more games you've
        // played (and still only gotten half correct)
        let mut s = PlayStats::new();
        s.inc_by(10, false);
        s.inc_by(10, true);
        // 1 - 10/21 == 0.52
        assert_eq!(s.weight(), 1.0 - 10.0 / 21.0);
        let mut s = PlayStats::new();
        s.inc(false);
        s.inc(true);
        // 1 - 1 /3 == 0.67
        assert_eq!(s.weight(), 1.0 - 1.0 / 3.0);
    }

    #[test]
    fn weight_incorrect() {
        // if always incorrect when inc, weight is 1
        let mut s = PlayStats::new();
        for _ in 0..COUNT_MANY {
            s.inc(false);
            assert_eq!(s.weight(), 1f32);
        }
    }

    #[test]
    fn weight_correct() {
        // if always correct when inc, weight is near 0
        let mut s = PlayStats::new();
        for (i, _) in (0..COUNT_MANY).enumerate() {
            let i = i + 1;
            s.inc(true);
            assert_eq!(s.weight(), 1.0 - i as f32 / (i as f32 + 1.0));
        }
    }

    #[test]
    fn inc_incorrect() {
        // increment incorrect resps many times performs ... correctly
        let mut s = PlayStats::new();
        for _ in 0..COUNT_MANY {
            s.inc(false);
        }
        assert_eq!(s.seen, COUNT_MANY as u16);
        assert_eq!(s.correct, 0);
    }

    #[test]
    fn inc_correct() {
        // increment correct resps many times performs ... correctly
        let mut s = PlayStats::new();
        for _ in 0..COUNT_MANY {
            s.inc(true);
        }
        assert_eq!(s.seen, COUNT_MANY as u16);
        assert_eq!(s.correct, COUNT_MANY as u16);
    }

    #[test]
    fn inc_by_incorrect() {
        // increment by many works
        let mut s = PlayStats::new();
        s.inc_by(COUNT_MANY as u16, false);
        s.inc_by(COUNT_MANY as u16, false);
        assert_eq!(s.seen, COUNT_MANY as u16 * 2);
        assert_eq!(s.correct, 0);
    }

    #[test]
    fn inc_by_correct() {
        // increment by many works
        let mut s = PlayStats::new();
        s.inc_by(COUNT_MANY as u16, true);
        s.inc_by(COUNT_MANY as u16, true);
        assert_eq!(s.seen, COUNT_MANY as u16 * 2);
        assert_eq!(s.correct, COUNT_MANY as u16 * 2);
    }

    #[test]
    fn inc_same() {
        // inc and inc_by act the same
        let mut s1 = PlayStats::new();
        s1.inc_by(COUNT_MANY as u16, true);
        s1.inc_by(COUNT_MANY as u16, false);
        let mut s2 = PlayStats::new();
        for _ in 0..COUNT_MANY {
            s2.inc(true);
            s2.inc(false);
        }
        assert_eq!(s1, s2);
    }

    #[test]
    fn addassign_1() {
        let mut s1 = PlayStats::new();
        let mut s2 = PlayStats::new();
        s2.inc(true);
        s1 += s2;
        assert_eq!(s1.seen(), 1);
        assert_eq!(s1.correct(), 1);
        assert_eq!(s1, s2);
        let mut s3 = PlayStats::new();
        s3.inc(false);
        s1 += s3;
        assert_eq!(s1.seen(), 2);
        assert_eq!(s1.correct(), 1);
    }

    #[test]
    fn addassign_2() {
        let mut s1 = PlayStats::new();
        let mut s2 = PlayStats::new();
        s2.inc(true);
        for _ in 0..COUNT_MANY {
            s1 += s2;
        }
        assert_eq!(s1.seen(), COUNT_MANY as u16);
        assert_eq!(s1.correct(), COUNT_MANY as u16);
    }

    #[test]
    fn addassign_3() {
        let mut s1 = PlayStats::new();
        s1 += PlayStats::new();
        assert_eq!(s1.seen(), 0);
        assert_eq!(s1.correct(), 0);
        s1.inc(true);
        s1 += PlayStats::new();
        assert_eq!(s1.seen(), 1);
        assert_eq!(s1.correct(), 1);
    }

    #[test]
    fn add_1() {
        let s = PlayStats::new() + PlayStats::new();
        assert_eq!(s.seen(), 0);
        assert_eq!(s.correct(), 0);
    }

    #[test]
    fn add_2() {
        let mut s1 = PlayStats::new();
        let mut s2 = PlayStats::new();
        s1.inc_by(COUNT_MANY as u16, true);
        s2.inc_by(COUNT_MANY as u16, false);
        let s3 = s1 + s2;
        assert_eq!(s3.seen(), 2 * COUNT_MANY as u16);
        assert_eq!(s3.correct(), COUNT_MANY as u16);
    }
}
