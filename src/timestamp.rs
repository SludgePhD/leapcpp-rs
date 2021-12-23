use std::{fmt, time::Duration};

/// A timestamp reported by the Leap Motion Service.
#[derive(Clone, Copy)]
pub struct Timestamp(i64);

impl Timestamp {
    #[inline]
    pub fn from_raw(raw: i64) -> Self {
        Self(raw)
    }

    #[inline]
    pub fn as_raw(self) -> i64 {
        self.0
    }

    pub fn duration_since(&self, earlier: Timestamp) -> Duration {
        Duration::from_micros(
            (self.0 as u64)
                .checked_sub(earlier.0 as u64)
                .expect("specified timestamp is later than self"),
        )
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}µs", self.0)
    }
}
