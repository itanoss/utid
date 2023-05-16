use rand::{rngs::ThreadRng, Rng};
use time::OffsetDateTime;

pub enum SegmentSpec {
    Timestamp {
        bits: u8,
        unit: TimestampUnit,
        since_timestamp: u64,
    },
    Random(u8),
    Constant(u8, u64),
}

impl SegmentSpec {
    fn bit_length(&self) -> &u8 {
        match self {
            SegmentSpec::Timestamp {
                bits,
                unit: _,
                since_timestamp: _,
            } => bits,
            SegmentSpec::Random(bits) => bits,
            SegmentSpec::Constant(bits, _) => bits,
        }
    }

    fn upper_bound(&self) -> u64 {
        let bits = *match self {
            SegmentSpec::Timestamp {
                bits,
                unit: _,
                since_timestamp: _,
            } => bits,
            SegmentSpec::Random(bits) => bits,
            SegmentSpec::Constant(bits, _) => bits,
        };
        if bits == 64u8 {
            u64::MAX
        } else {
            (1 << bits) - 1
        }
    }

    fn generate(&self) -> Result<u64, Error> {
        let candidate = match self {
            SegmentSpec::Timestamp {
                bits,
                unit,
                since_timestamp,
            } => {
                let now = OffsetDateTime::now_utc().unix_timestamp_nanos() as u64;
                now - since_timestamp // TODO Support timestamp units
            }
            SegmentSpec::Random(_) => {
                let mut rng = rand::thread_rng();
                rng.gen_range(0..=self.upper_bound())
            }
            SegmentSpec::Constant(_, value) => *value,
        };

        if self.upper_bound() < candidate {
            Err(Error::OverflowError)
        } else {
            Ok(candidate)
        }
    }
}

#[derive(Debug)]
pub enum Error {
    OverflowError,
}

pub enum TimestampUnit {
    Seconds,
    Milliseconds,
    Nanoseconds,
}

impl TimestampUnit {
    fn from_nanos(&self, nanos: u64) -> u64 {
        match self {
            TimestampUnit::Seconds => nanos / 1000 / 1000,
            TimestampUnit::Milliseconds => nanos / 1000,
            TimestampUnit::Nanoseconds => nanos,
        }
    }
}

pub struct Utid {
    specs: Vec<SegmentSpec>,
}

impl Utid {
    pub fn new(specs: Vec<SegmentSpec>) -> Self {
        let total_bit_length: u8 = (&specs).into_iter().map(|s| s.bit_length()).sum();
        if total_bit_length == 64 {
            return Utid { specs };
        }

        panic!()
    }

    pub fn generate(&self) -> Result<u64, Error> {
        let (result, _) =
            (&self.specs)
                .into_iter()
                .rev()
                .try_fold((0, 0), |(acc, shift), spec| {
                    spec.generate()
                        .map(|segment| (acc | (segment << shift), shift + spec.bit_length()))
                })?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use time::Date;

    use super::*;

    #[test]
    fn entire_constant() {
        let utid = Utid::new(vec![SegmentSpec::Constant(64, 12345)]);
        assert_eq!(12345, utid.generate().unwrap());
    }

    #[test]
    fn entire_random() {
        let utid = Utid::new(vec![SegmentSpec::Random(64)]);
        println!("Full bits of random: {}", utid.generate().unwrap());
    }

    #[test]
    fn entire_timestamp() {
        let epoch = Date::from_calendar_date(2023, time::Month::January, 1)
            .unwrap()
            .midnight()
            .assume_utc()
            .unix_timestamp_nanos() as u64;
        println!("Epoch timestamp: {}", epoch);
        let utid = Utid::new(vec![SegmentSpec::Timestamp {
            bits: 64,
            unit: TimestampUnit::Nanoseconds,
            since_timestamp: epoch,
        }]);
        println!("Full bits of timestamp: {}", utid.generate().unwrap());
    }

    #[test]
    fn random_and_constant() {
        let utid = Utid::new(vec![
            SegmentSpec::Random(32),
            SegmentSpec::Constant(32, 12345),
        ]);
        println!(
            "Half random and Half constant: {}",
            utid.generate().unwrap()
        );
    }
}
