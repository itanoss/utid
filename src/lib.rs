use std::fmt;

use rand::Rng;
use time::{Date, Duration, OffsetDateTime};

pub trait SpecSegment<T, R> {
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
    pub fn new_with_utc_midnight(size: u8, unit: TimestampUnit, since: Date) -> Self {
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
        let offset = if self.size == 128u8 {
            i128::MAX
        } else {
            (1 << self.size) - 1
        };
        let duration = Duration::new(
            i64::try_from(self.unit.to_nano(offset) / 1_000_000_000).unwrap(),
            i32::try_from(self.unit.to_nano(offset) % 1_000_000_000).unwrap(),
            // TODO cover overflow
        );
        self.since + duration
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

impl fmt::Display for TimestampSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TimestampSegment(since:{}, until:{})",
            self.since,
            self.upper_bound()
        )
    }
}

pub struct RandomSegment {
    size: u8,
}

impl RandomSegment {
    // TODO Consider this public modifier is needed
    pub fn new(size: u8) -> Self {
        Self { size }
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
    pub fn new(size: u8, value: i128) -> Self {
        Self { size, value }
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
    Microseconds,
    Nanoseconds,
}

impl TimestampUnit {
    fn from_nano(&self, nanos: i128) -> i128 {
        match self {
            TimestampUnit::Seconds => nanos / 1_000_000_000,
            TimestampUnit::Milliseconds => nanos / 1_000_000,
            TimestampUnit::Microseconds => nanos / 1_000,
            TimestampUnit::Nanoseconds => nanos,
        }
    }

    fn to_nano(&self, value: i128) -> i128 {
        match self {
            TimestampUnit::Seconds => value * 1_000_000_000,
            TimestampUnit::Milliseconds => value * 1_000_000,
            TimestampUnit::Microseconds => value * 1_000,
            TimestampUnit::Nanoseconds => value,
        }
    }
}
// TODO Consider macro generation to support up to 8 segments
pub struct Spec<T, R> {
    // TODO Check if removing pub modifier is possible
    pub segment: Box<dyn SpecSegment<T, R>>,
}
pub struct Spec2<T, R1, R2> {
    pub segments: (Box<dyn SpecSegment<T, R1>>, Box<dyn SpecSegment<T, R2>>),
}
pub struct Spec3<T, R1, R2, R3> {
    pub segments: (
        Box<dyn SpecSegment<T, R1>>,
        Box<dyn SpecSegment<T, R2>>,
        Box<dyn SpecSegment<T, R3>>,
    ),
}
pub struct Spec4<T, R1, R2, R3, R4> {
    pub segments: (
        Box<dyn SpecSegment<T, R1>>,
        Box<dyn SpecSegment<T, R2>>,
        Box<dyn SpecSegment<T, R3>>,
        Box<dyn SpecSegment<T, R4>>,
    ),
}

impl<R> Spec<i128, R> {
    pub fn generate(&self) -> Result<i128, Error> {
        self.segment.encode()
    }

    pub fn decompose(&self, generated: i128) -> Result<R, Error> {
        Ok(self.segment.decode(generated))
    }
}

impl<R1, R2> Spec2<i128, R1, R2> {
    pub fn generate(&self) -> Result<i128, Error> {
        let mut result = self.segments.1.encode()?;
        result |= self.segments.0.encode()? << self.segments.1.size();
        Ok(result)
    }

    pub fn decompose(&self, generated: i128) -> Result<(R1, R2), Error> {
        let second = ((1i128 << self.segments.1.size()) - 1) & generated;
        let second = self.segments.1.decode(second);

        let first = ((generated as u128) >> self.segments.1.size()) as i128;
        let first = self.segments.0.decode(first);
        Ok((first, second))
    }
}

impl<R1, R2, R3> Spec3<i128, R1, R2, R3> {
    pub fn generate(&self) -> Result<i128, Error> {
        let mut result = self.segments.2.encode()?;
        let mut shift = self.segments.2.size();
        result |= self.segments.1.encode()? << shift;
        shift += self.segments.1.size();

        result |= self.segments.0.encode()? << shift;
        Ok(result)
    }

    pub fn decompose(&self, generated: i128) -> Result<(R1, R2, R3), Error> {
        let third = ((1i128 << self.segments.2.size()) - 1) & generated;
        let third = self.segments.2.decode(third);
        let mut shift = self.segments.2.size();
        
        let second = (((1i128 << (self.segments.1.size() + shift)) - 1) & generated) >> shift;
        let second = self.segments.1.decode(second);
        shift += self.segments.1.size();

        let first = generated >> shift;
        let first = self.segments.0.decode(first);
        Ok((first, second, third))
    }
}

impl<R1, R2, R3, R4> Spec4<i128, R1, R2, R3, R4> {
    pub fn generate(&self) -> Result<i128, Error> {
        let mut result = self.segments.3.encode()?;
        let mut shift = self.segments.3.size();
        result |= self.segments.2.encode()? << shift;
        shift += self.segments.2.size();

        result |= self.segments.1.encode()? << shift;
        shift += self.segments.1.size();

        result |= self.segments.0.encode()? << shift;
        Ok(result)
    }

    pub fn decompose(&self, generated: i128) -> Result<(R1, R2, R3, R4), Error> {
        let fourth = ((1i128 << self.segments.3.size()) - 1) & generated;
        let fourth = self.segments.3.decode(fourth);
        let mut shift = self.segments.3.size();

        let third = (((1i128 << (self.segments.2.size() + shift)) - 1) & generated) >> shift;
        let third = self.segments.2.decode(third);
        shift += self.segments.2.size();

        let second = (((1i128 << (self.segments.1.size() + shift)) - 1) & generated) >> shift;
        let second = self.segments.1.decode(second);
        shift += self.segments.1.size();

        let first = generated >> shift;
        let first = self.segments.0.decode(first);
        Ok((first, second, third, fourth))
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

        let generated = spec.generate().unwrap();
        assert_eq!(12345, generated);

        let constant = spec.decompose(generated).unwrap();
        assert_eq!(12345, constant);
    }

    #[test]
    fn entire_random() {
        let spec = Spec {
            segment: Box::new(RandomSegment::new(128)),
        };

        let generated = spec.generate().unwrap();

        let random_number = spec.decompose(generated).unwrap();
        assert_eq!(generated, random_number);
        println!("Full bits of random: {}", generated);
    }

    #[test]
    fn entire_timestamp() {
        let spec = Spec {
            segment: Box::new(TimestampSegment::new_with_utc_midnight(
                128,
                TimestampUnit::Nanoseconds,
                Date::from_calendar_date(2023, time::Month::January, 1).unwrap(),
            )),
        };
        let generated = spec.generate().unwrap();

        let timestamp = spec.decompose(generated).unwrap();
        println!("Full bits of timestamp: {} ({})", generated, timestamp);
    }

    #[test]
    fn segment_display() {
        let segment = TimestampSegment::new_with_utc_midnight(
            52,
            TimestampUnit::Microseconds,
            Date::from_calendar_date(2023, time::Month::January, 1).unwrap(),
        );
        println!("timestamp segment: {}", segment);
    }

    #[test]
    fn constant_and_random() {
        let spec = Spec2 {
            segments: (
                Box::new(ConstantSegment::new(48, 12345)),
                Box::new(RandomSegment::new(80)),
            ),
        };
        let generated = spec.generate().unwrap();
        let (constant, random) = spec.decompose(generated).unwrap();
        println!("Constant: {}, random: {}", constant, random);
    }
}
