#[allow(dead_code)]
mod csx_temporal {
    use std::fmt::{self, Display, Formatter};

    #[derive(Clone, Copy)]
    pub struct Date {
        pub year: i64,
        pub month: i64,
        pub day: i64,
    }
    #[derive(Clone, Copy)]
    pub struct Time {
        pub hour: i64,
        pub minute: i64,
        pub second: i64,
    }
    #[derive(Clone, Copy)]
    pub struct DateTime {
        pub date: Date,
        pub time: Time,
    }
    #[derive(Clone, Copy)]
    pub struct Span(i64);

    pub fn date(year: i64, month: i64, day: i64) -> Result<Date, String> {
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let days = match month {
            2 => 28 + i64::from(leap),
            4 | 6 | 9 | 11 => 30,
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            _ => 0,
        };
        if !(1..=9999).contains(&year) || !(1..=days).contains(&day) {
            return Err("ArgumentOutOfRangeException: invalid Gregorian date".into());
        }
        Ok(Date { year, month, day })
    }

    pub fn time(hour: i64, minute: i64, second: i64) -> Result<Time, String> {
        if !(0..24).contains(&hour) || !(0..60).contains(&minute) || !(0..60).contains(&second) {
            return Err("ArgumentOutOfRangeException: invalid time of day".into());
        }
        Ok(Time {
            hour,
            minute,
            second,
        })
    }

    pub fn date_time(y: i64, m: i64, d: i64, h: i64, min: i64, s: i64) -> Result<DateTime, String> {
        Ok(DateTime {
            date: date(y, m, d)?,
            time: time(h, min, s)?,
        })
    }

    pub fn span(days: i64, hours: i64, minutes: i64, seconds: i64) -> Result<Span, String> {
        let value = i128::from(days) * 86_400
            + i128::from(hours) * 3_600
            + i128::from(minutes) * 60
            + i128::from(seconds);
        if i64::try_from(value * 10_000_000).is_err() {
            return Err("ArgumentOutOfRangeException: duration exceeds TimeSpan range".into());
        }
        Ok(Span(value as i64))
    }

    impl Span {
        pub fn days(self) -> i64 {
            self.0 / 86_400
        }
        pub fn hours(self) -> i64 {
            self.0 / 3_600 % 24
        }
        pub fn minutes(self) -> i64 {
            self.0 / 60 % 60
        }
        pub fn seconds(self) -> i64 {
            self.0 % 60
        }
    }

    impl Display for Date {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
        }
    }
    impl Display for Time {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
        }
    }
    impl Display for DateTime {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{}T{}", self.date, self.time)
        }
    }
    impl Display for Span {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            if self.0 < 0 {
                f.write_str("-")?;
            }
            if self.days() != 0 {
                write!(f, "{}.", self.days().abs())?;
            }
            write!(
                f,
                "{:02}:{:02}:{:02}",
                self.hours().abs(),
                self.minutes().abs(),
                self.seconds().abs()
            )
        }
    }
}
