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
        /// Any missing portion/fraction will not be part of the collection
        pub fn get_time(&self) -> Vec<(String, u32)> {
            let mut time = vec![];

            let years = self.time_in_minutes / (60 * 24 * 365);
            let months = (self.time_in_minutes / (60 * 24 * 30)) % 12;
            let days = (self.time_in_minutes / (60 * 24)) % 30;
            let hours = (self.time_in_minutes / 60) % 24;

            let float_hours = (self.time_in_minutes as f32 / 60.0) % 24.0;
            let minutes = (float_hours.fract() * 60.0) as u32;

            if minutes > 0 {
                time.push(("Minute", minutes))
            }
            if hours > 0 {
                time.push(("Hour", hours))
            }
            if days > 0 {
                time.push(("Day", days))
            }
            if months > 0 {
                time.push(("Month", months))
            }
            if years > 0 {
                time.push(("Year", years))
            }
            time.into_iter().map(plurize_time).collect()
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
}
