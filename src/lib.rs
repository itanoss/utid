use rand::{Rng, rngs::ThreadRng};
use time::{Date, OffsetDateTime};

trait SpecSegment<T, R> {
    fn size(&self) -> u8;
    fn upper_bound(&self) -> R;
    fn encode(&self) -> Result<T, Error>;
    fn decode(&self, encoded: T) -> R;
}

pub struct TimestampSegment {
    size: u8,
    unit: TimestampUnit,
    since: OffsetDateTime,
}

impl TimestampSegment {
    fn new_with_utc_midnight(size: u8, unit: TimestampUnit, since: Date) -> Self {
        Self {
            size,
            unit,
            since: since.midnight().assume_utc(),
        }
    }
}

impl SpecSegment<i128, OffsetDateTime> for TimestampSegment {
    fn size(&self) -> u8 {
        self.size
    }

    fn upper_bound(&self) -> OffsetDateTime {
        let origin = self.unit.from_nano(self.since.unix_timestamp_nanos());
        let offset = if self.size == 128u8 {
            i128::MAX
        } else {
            (1 << self.size) - 1
        };
        OffsetDateTime::from_unix_timestamp_nanos(origin + offset).unwrap() // TODO Cover overflow
    }

    fn encode(&self) -> Result<i128, Error> {
        let now = OffsetDateTime::now_utc();
        let duration = now - self.since;
        Ok(self.unit.from_nano(duration.whole_nanoseconds()))
    }

    fn decode(&self, encoded: i128) -> OffsetDateTime {
        let origin = self.unit.from_nano(self.since.unix_timestamp_nanos());
        OffsetDateTime::from_unix_timestamp_nanos(origin + encoded).unwrap() // TODO Cover overflow
    }
}

pub struct RandomSegment {
    size: u8,
}

impl RandomSegment {
    fn new(size: u8) -> Self {
        Self { size, }
    }
}

impl SpecSegment<i128, i128> for RandomSegment {
    fn size(&self) -> u8 {
        self.size
    }

    fn upper_bound(&self) -> i128 {
        if self.size == 128u8 {
            i128::MAX
        } else {
            (1 << self.size) - 1
        }
    }

    fn encode(&self) -> Result<i128, Error> {
        let mut rng = rand::thread_rng();
        Ok(rng.gen_range(0..=self.upper_bound()))

    }

    fn decode(&self, encoded: i128) -> i128 {
        encoded
    }
}

pub struct ConstantSegment<T> {
    size: u8,
    value: T,
}

impl ConstantSegment<i128> {
    fn new(size: u8, value: i128) -> Self {
        Self {
            size,
            value,
        }
    }
}

impl SpecSegment<i128, i128> for ConstantSegment<i128> {
    fn size(&self) -> u8 {
        self.size
    }

    fn upper_bound(&self) -> i128 {
        if self.size == 128u8 {
            i128::MAX
        } else {
            (1 << self.size) - 1
        }
    }

    fn encode(&self) -> Result<i128, Error> {
        Ok(self.value)
    }

    fn decode(&self, encoded: i128) -> i128 {
        encoded
    }
}

#[derive(Debug)]
pub enum Error {
    OverflowError,
}

#[derive(Debug)]
pub enum TimestampUnit {
    Seconds,
    Milliseconds,
    Nanoseconds,
}

impl TimestampUnit {
    fn from_nano(&self, nanos: i128) -> i128 {
        match self {
            TimestampUnit::Seconds => nanos / 1000 / 1000,
            TimestampUnit::Milliseconds => nanos / 1000,
            TimestampUnit::Nanoseconds => nanos,
        }
    }

    fn to_nano(&self, value: i128) -> i128 {
        match self {
            TimestampUnit::Seconds => value * 1000 * 1000,
            TimestampUnit::Milliseconds => value * 1000,
            TimestampUnit::Nanoseconds => value,
        }
    }
}

pub struct Spec<T, R> {
    segment: Box<dyn SpecSegment<T, R>>,
}
pub struct Spec2<T, R1, R2> {
    segments: (Box<dyn SpecSegment<T, R1>>, Box<dyn SpecSegment<T, R2>>),
}
pub struct Spec3<T, R1, R2, R3> {
    segments: (Box<dyn SpecSegment<T, R1>>, Box<dyn SpecSegment<T, R2>>, Box<dyn SpecSegment<T, R3>>),
}
pub struct Spec4<T, R1, R2, R3, R4> {
    segments: (Box<dyn SpecSegment<T, R1>>, Box<dyn SpecSegment<T, R2>>, Box<dyn SpecSegment<T, R3>>, Box<dyn SpecSegment<T, R4>>),
}

impl<R> Spec<i128, R> {
    fn generate(&self) -> Result<i128, Error> {
        self.segment.encode()
    }
}

impl<R1, R2> Spec2<i128, R1, R2> {
    fn generate(&self) -> Result<i128, Error> {
        let mut result = self.segments.1.encode()?;
        result |= self.segments.0.encode()? << self.segments.1.size();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use time::Date;

    use super::*;

    #[test]
    fn entire_constant() {
        let spec = Spec {
            segment: Box::new(ConstantSegment::new(128, 12345)),
        };
        assert_eq!(12345, spec.generate().unwrap());
    }

    #[test]
    fn entire_random() {
        let spec = Spec {
            segment: Box::new(RandomSegment::new(128)),
        };
        println!("Full bits of random: {}", spec.generate().unwrap());
    }

}
