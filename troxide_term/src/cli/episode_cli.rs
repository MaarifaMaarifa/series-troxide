pub use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct EpisodeCli {
    #[clap(subcommand)]
    pub episode_command: EpisodeCommand,
}

#[derive(Subcommand)]
pub enum EpisodeCommand {
    /// Add episode into a series
    Add(AddEpisodeCli),

    /// Add episodes using a range
    AddRange(AddEpisodeRangeCli),

    /// Remove episode from a series
    Remove(RemoveEpisodeCli),

    /// Lists all the tracked episodes in a season of a series
    List(ListEpisodeCli),
}

#[derive(Parser)]
pub struct AddEpisodeCli {
    /// Series name to add the episode to
    pub series: String,

    /// Season number associated
    pub season: u32,

    /// The episode number to be added
    pub episode: u32,
}

#[derive(Parser)]
pub struct AddEpisodeRangeCli {
    /// Series name to add the episode to
    pub series: String,

    /// Season number associated
    pub season: u32,

    /// The episode range to be added ie. 3-9 means (three to nine inclusively)
    pub episode_range: String,
}

#[derive(Parser)]
pub struct RemoveEpisodeCli {
    /// Series name to remove episode from
    pub series: String,

    /// Season number associated
    pub season: u32,

    /// The episode number or range to be removed
    pub episode: u32,
}

#[derive(Parser)]
pub struct ListEpisodeCli {
    /// Series name to get the list from
    pub series: String,

    /// Season number associated
    pub season: u32,
}
