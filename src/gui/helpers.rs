/// Parses the episode number or season number to proper number to be
/// displayed
///
/// This is gonna prefix a zero if a number is less than 10, and nothing for the
/// opposite.
fn parse_season_episode_number(number: u32) -> String {
    if number < 10_u32 {
        format!("0{}", number)
    } else {
        number.to_string()
    }
}

/// Generates the season and episode string
///
/// season no 2 and episode no 3 will generate S02E03
pub fn season_episode_str_gen(season_number: u32, episode_number: u32) -> String {
    format!(
        "S{}E{}",
        parse_season_episode_number(season_number),
        parse_season_episode_number(episode_number)
    )
}

pub fn genres_with_pipes(genres: &[String]) -> String {
    let mut genres_string = String::new();

    let mut series_result_iter = genres.iter().peekable();
    while let Some(genre) = series_result_iter.next() {
        genres_string.push_str(genre);
        if series_result_iter.peek().is_some() {
            genres_string.push_str(" | ");
        }
    }
    genres_string
}

pub mod time {
    //! Time related helpers
    use chrono::Duration;
    use smallvec::SmallVec;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TimeKind {
        Year,
        Month,
        Day,
        Hour,
        Minute,
    }

    impl std::fmt::Display for TimeKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let time_str = match self {
                TimeKind::Year => "Year",
                TimeKind::Month => "Month",
                TimeKind::Day => "Day",
                TimeKind::Hour => "Hour",
                TimeKind::Minute => "Minute",
            };

            write!(f, "{}", time_str)
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct NaiveDuration {
        time_value: u32,
        time_kind: TimeKind,
    }

    impl NaiveDuration {
        fn new(time_value: u32, time_kind: TimeKind) -> Self {
            Self {
                time_kind,
                time_value,
            }
        }

        fn plurized(&self) -> (u32, String) {
            plurize_time(self.time_value, &self.time_kind.to_string())
        }
    }

    /// `Time` split into it's remainder parts in the order of
    /// years, months, days, hours, minutes
    ///
    /// The components with value of 0 in `NaiveTime` parts won't be included
    /// but the order will be preserved
    ///
    /// `NaiveTime` is designed to make it easy to understand time with fractional parts
    /// so instead of saying 2.5 days, we can express this as 2 Days 12 Hours
    pub struct NaiveTime {
        // an array consisting of durations in the order of
        // years, months, days, hours, minutes
        time_parts: SmallVec<[NaiveDuration; 5]>,
    }

    impl NaiveTime {
        pub fn new(time_in_minutes: u32) -> Self {
            let mut time_parts = SmallVec::new();

            let years = time_in_minutes / (60 * 24 * 365);
            let months = (time_in_minutes / (60 * 24 * 30)) % 12;
            let days = (time_in_minutes / (60 * 24)) % 30;
            let hours = (time_in_minutes / 60) % 24;

            let float_hours = (time_in_minutes as f32 / 60.0) % 24.0;
            let minutes = (float_hours.fract() * 60.0) as u32;

            if years > 0 {
                time_parts.push(NaiveDuration::new(years, TimeKind::Year))
            }
            if months > 0 {
                time_parts.push(NaiveDuration::new(months, TimeKind::Month))
            }

            if days > 0 {
                time_parts.push(NaiveDuration::new(days, TimeKind::Day))
            }
            if hours > 0 {
                time_parts.push(NaiveDuration::new(hours, TimeKind::Hour))
            }
            if minutes > 0 {
                time_parts.push(NaiveDuration::new(minutes, TimeKind::Minute))
            }

            Self { time_parts }
        }

        pub fn as_parts(&self) -> SmallVec<[(u32, String); 5]> {
            self.time_parts
                .iter()
                .map(|sane_duration| sane_duration.plurized())
                .collect()
        }

        pub fn largest_part(&self) -> Option<(u32, String)> {
            self.time_parts
                .first()
                .map(|sane_duration| sane_duration.plurized())
        }

        pub fn get_longest_unit_duration(&self) -> Option<Duration> {
            self.time_parts
                .first()
                .map(|sane_duration| Self::to_chrono_duration_unit(&sane_duration.time_kind))
        }

        /// Returns the Unit of the `TimeKind`
        ///
        /// For example `TimeKind::Hour` will produce a duration of 1 hour
        fn to_chrono_duration_unit(time_kind: &TimeKind) -> Duration {
            match time_kind {
                TimeKind::Minute => Duration::minutes(1),
                TimeKind::Hour => Duration::hours(1),
                TimeKind::Day => Duration::days(1),
                TimeKind::Month => Duration::weeks(4),
                TimeKind::Year => Duration::weeks(52),
            }
        }
    }

    /// Takes the time and it's name i.e week, day, hour and concatenates the
    /// two terms handling the condition when the time is above 1 (plural)
    fn plurize_time(time_value: u32, time_text: &str) -> (u32, String) {
        let word = if time_value > 1 {
            format!("{}s", time_text)
        } else {
            time_text.to_string()
        };
        (time_value, word)
    }

    impl std::fmt::Display for NaiveTime {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let str = self
                .time_parts
                .iter()
                .map(|sane_duration| sane_duration.plurized())
                .fold(String::new(), |acc, (time_value, time_text)| {
                    acc + &format!("{} {} ", time_value, time_text)
                });

            write!(f, "{}", str)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::{NaiveDuration, NaiveTime, TimeKind};

        #[test]
        fn minute_test() {
            let time_parts = NaiveTime::new(1).time_parts.into_vec();
            assert_eq!(time_parts, vec![NaiveDuration::new(1, TimeKind::Minute)])
        }

        #[test]
        fn hour_test() {
            let time_parts = NaiveTime::new(60).time_parts.into_vec();
            assert_eq!(time_parts, vec![NaiveDuration::new(1, TimeKind::Hour)])
        }

        #[test]
        fn day_test() {
            let time_parts = NaiveTime::new(60 * 24).time_parts.into_vec();
            assert_eq!(time_parts, vec![NaiveDuration::new(1, TimeKind::Day)])
        }

        #[test]
        fn month_test() {
            let time_parts = NaiveTime::new(60 * 24 * 30).time_parts.into_vec();
            assert_eq!(time_parts, vec![NaiveDuration::new(1, TimeKind::Month)])
        }

        #[test]
        fn year_test() {
            let time_parts = NaiveTime::new(365 * 24 * 60).time_parts.into_vec();
            assert_eq!(
                time_parts,
                vec![
                    NaiveDuration::new(1, TimeKind::Year),
                    NaiveDuration::new(5, TimeKind::Day)
                ]
            )
        }

        #[test]
        fn fractional_day_test() {
            let time_parts = NaiveTime::new((24 * 60) + (12 * 60)).time_parts.into_vec();
            assert_eq!(
                time_parts,
                vec![
                    NaiveDuration::new(1, TimeKind::Day,),
                    NaiveDuration::new(12, TimeKind::Hour)
                ]
            )
        }
    }
}

pub mod empty_image {
    use crate::gui::assets::icons::SERIES_TROXIDE_GRAY_SCALED_ICON;

    use iced::widget::{svg, Svg};

    /// Placeholder for an empty image
    pub fn empty_image() -> Svg<'static> {
        let icon_handle = svg::Handle::from_memory(SERIES_TROXIDE_GRAY_SCALED_ICON);
        svg(icon_handle)
    }
}
