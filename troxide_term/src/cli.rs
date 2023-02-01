pub use clap::{Parser, Subcommand};
use database_cli::DatabaseCli;
use episode_cli::EpisodeCli;
use season_cli::SeasonCli;
use series_cli::SeriesCli;
use std::num::ParseIntError;
use thiserror::Error;

pub mod database_cli;
pub mod episode_cli;
pub mod season_cli;
pub mod series_cli;

#[derive(Parser)]
#[clap(about, version, author)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Perform actions related to series
    Series(SeriesCli),

    /// Perform actions related to season
    Season(SeasonCli),

    /// Perform actions related to episode
    Episode(EpisodeCli),

    /// Perform actions related to series database
    Database(DatabaseCli),
}

/// Error cases that can be returned by methods in RangeParser Struct
#[derive(Debug, Error)]
pub enum RangeParserError {
    #[error("The string syntax is incorrect, correct form is 3-7")]
    Syntax,

    #[error("The start range number is invalid")]
    StartRange(ParseIntError),

    #[error("The end range number is invalid")]
    EndRange(ParseIntError),
}

/// Struct dealing with Parsing of ranges given by the user through the command line options
pub struct RangeParser;

impl RangeParser {
    /// Parses a Range out of a str
    pub fn get_range(range_str: &str) -> Result<std::ops::RangeInclusive<u32>, RangeParserError> {
        let range_components = range_str.split_once('-');

        let range_components = if let Some(components) = range_components {
            components
        } else {
            return Err(RangeParserError::Syntax);
        };

        let start: u32 = match range_components.0.parse() {
            Ok(num) => num,
            Err(err) => return Err(RangeParserError::StartRange(err)),
        };

        let end: u32 = match range_components.1.parse() {
            Ok(num) => num,
            Err(err) => return Err(RangeParserError::EndRange(err)),
        };

        Ok(start..=end)
    }
}
