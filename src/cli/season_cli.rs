pub use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct SeasonCli {
    #[clap(subcommand)]
    pub season_command: SeasonCommand,
}

#[derive(Subcommand)]
pub enum SeasonCommand {
    /// Add season into a series
    Add(AddSeasonCli),

    /// Remove season from a series
    Remove(RemoveSeasonCli),
}

#[derive(Parser)]
pub struct AddSeasonCli {
    /// Series name to add the season to
    pub series: String,

    /// Season number or range to be added
    pub season: u32,
}

#[derive(Parser)]
pub struct RemoveSeasonCli {
    /// Series name to remove season from
    pub series: String,

    /// Season number or range to be removed
    pub season: u32,
}
