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
