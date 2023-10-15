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

pub mod time {
    //! Time related helpers
    use chrono::Duration;

    #[derive(Clone)]
    pub enum TimeKind {
        Minute,
        Hour,
        Day,
        Month,
        Year,
    }

    impl std::fmt::Display for TimeKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let time_str = match self {
                TimeKind::Minute => "Minute",
                TimeKind::Hour => "Hour",
                TimeKind::Day => "Day",
                TimeKind::Month => "Month",
                TimeKind::Year => "Year",
            };

            write!(f, "{}", time_str)
        }
    }

    pub struct SaneTime {
        time_in_minutes: u32,
    }

    impl SaneTime {
        pub fn new(time_in_minutes: u32) -> Self {
            Self { time_in_minutes }
        }

        /// Return a `Vec` of time values and their texts starting from
        /// the smallest to the largest
        ///
        /// This is useful if you want to get the highest sane time(a year for example)
        /// and all of it's remaining portions split in months days, hours and finally minutes.
        ///
        /// # Note
        /// The `Vec` returned will be empty if the time is smaller than 1 minute as that's the
        /// smallest amount time that can be returned in the collection.
        pub fn get_time_plurized(&self) -> Vec<(String, u32)> {
            self.get_time()
                .into_iter()
                .map(|(time_kind, time_value)| plurize_time((&time_kind.to_string(), time_value)))
                .collect()
        }

        pub fn get_time(&self) -> Vec<(TimeKind, u32)> {
            let mut time = vec![];

            let years = self.time_in_minutes / (60 * 24 * 365);
            let months = (self.time_in_minutes / (60 * 24 * 30)) % 12;
            let days = (self.time_in_minutes / (60 * 24)) % 30;
            let hours = (self.time_in_minutes / 60) % 24;

            let float_hours = (self.time_in_minutes as f32 / 60.0) % 24.0;
            let minutes = (float_hours.fract() * 60.0) as u32;

            if minutes > 0 {
                time.push((TimeKind::Minute, minutes))
            }
            if hours > 0 {
                time.push((TimeKind::Hour, hours))
            }
            if days > 0 {
                time.push((TimeKind::Day, days))
            }
            if months > 0 {
                time.push((TimeKind::Month, months))
            }
            if years > 0 {
                time.push((TimeKind::Year, years))
            }
            time
        }

        // pub fn get_shortest_duration(&self) -> Option<Duration> {
        //     self.get_time()
        //         .first()
        //         .map(|(time_kind, time_value)| Self::to_chrono_duration(time_kind, *time_value))
        // }

        // pub fn get_longest_duration(&self) -> Option<Duration> {
        //     self.get_time()
        //         .last()
        //         .map(|(time_kind, time_value)| Self::to_chrono_duration(time_kind, *time_value))
        // }

        /// This returns the longest time after the split in it's unit value
        ///
        /// For example, if in the split you got 5 days as the longest duration, the duration
        /// returned will be 1 day
        /// This is usefull in refreshing the upcoming episodes in the gui as if an episode is released
        /// in 5 hours, we want to refresh every 1 hour, or say 10 minutes, we will want to refresh in every
        /// minute and so on
        pub fn get_longest_unit_duration(&self) -> Option<Duration> {
            self.get_time()
                .last()
                .map(|(time_kind, _)| Self::to_chrono_duration_unit(time_kind))
        }

        // fn to_chrono_duration(time_kind: &TimeKind, time_value: u32) -> Duration {
        //     match time_kind {
        //         TimeKind::Minute => Duration::minutes(time_value as i64),
        //         TimeKind::Hour => Duration::hours(time_value as i64),
        //         TimeKind::Day => Duration::days(time_value as i64),
        //         TimeKind::Month => Duration::weeks(time_value as i64 * 4),
        //         TimeKind::Year => Duration::weeks(time_value as i64 * 52),
        //     }
        // }

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
    fn plurize_time(it: (&str, u32)) -> (String, u32) {
        let (time_text, time_value) = it;
        let word = if time_value > 1 {
            format!("{}s", time_text)
        } else {
            time_text.to_string()
        };
        (word, time_value)
    }

    impl std::fmt::Display for SaneTime {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let str = self
                .get_time_plurized()
                .into_iter()
                .rev()
                .fold(String::new(), |acc, (time_text, time_value)| {
                    acc + &format!("{} {} ", time_value, time_text)
                });

            write!(f, "{}", str)
        }
    }
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
