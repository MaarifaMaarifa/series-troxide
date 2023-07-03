/// Parses the episode number or season number to proper number to be
/// displayed
///
/// This is gonna prefix a zero if a number is less than 10, and nothing for the
/// opposite.
pub fn parse_season_episode_number(number: u32) -> String {
    if number < 10_u32 {
        format!("0{}", number)
    } else {
        number.to_string()
    }
}
