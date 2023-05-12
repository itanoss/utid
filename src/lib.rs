use time::OffsetDateTime;

pub enum SegmentSpec {
    Timestamp {
        bits: u8,
        unit: TimestampUnit,
        sinceTimestamp: u64,
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
                sinceTimestamp: _,
            } => bits,
            SegmentSpec::Random(bits) => bits,
            SegmentSpec::Constant(bits, _) => bits,
        }
    }
}

pub enum TimestampUnit {
    Seconds,
    Milliseconds,
    Nanoseconds,
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

    pub fn generate(&self) -> u64 {
        let (result, _) = (&self.specs)
            .into_iter()
            .rev()
            .fold((0, 0), |(acc, shift), s| {
                let segment = Utid::generate_segment(s);
                (acc | (segment << shift), shift + s.bit_length())
            });
        result
    }

    fn generate_segment(segment: &SegmentSpec) -> u64 {
        match segment {
            SegmentSpec::Timestamp {
                bits,
                unit,
                sinceTimestamp,
            } => {
                let now = OffsetDateTime::now_utc().unix_timestamp_nanos() as u64;
                now - sinceTimestamp
            }
            SegmentSpec::Random(bits) => rand::random::<u64>(),
            SegmentSpec::Constant(_, value) => *value,
        }
    }
}

#[cfg(test)]
mod tests {
    use time::{Date, OffsetDateTime};

    use super::*;

    #[test]
    fn entire_constant() {
        let utid = Utid::new(vec![SegmentSpec::Constant(64, 12345)]);
        assert_eq!(12345, utid.generate());
    }

    #[test]
    fn entire_random() {
        let utid = Utid::new(vec![SegmentSpec::Random(64)]);
        println!("Full bits of random: {}", utid.generate());
    }

    #[test]
    fn entire_timestamp() {
        let epoch =  Date::from_calendar_date(2023, time::Month::January, 1)
        .unwrap()
        .midnight()
        .assume_utc()
        .unix_timestamp_nanos() as u64;
        println!("Epoch timestamp: {}", epoch);
        let utid = Utid::new(vec![SegmentSpec::Timestamp {
            bits: 64,
            unit: TimestampUnit::Nanoseconds,
            sinceTimestamp: epoch,
        }]);
        println!("Full bits of timestamp: {}", utid.generate());
    }

    #[test]
    fn random_and_constant() {
        let utid = Utid::new(vec![
            SegmentSpec::Random(32),
            SegmentSpec::Constant(32, 12345),
        ]);
        println!("Half random and Half constant: {}", utid.generate());
    }
}
