pub use clap::{Parser, Subcommand};
use series_cli::SeriesCli;
use season_cli::SeasonCli;
use episode_cli::EpisodeCli;
use database_cli::DatabaseCli;

pub mod series_cli;
pub mod season_cli;
pub mod episode_cli;
pub mod database_cli;

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

