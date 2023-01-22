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

    /// Remove episode from a series
    Remove(RemoveEpisodeCli),
}

#[derive(Parser)]
pub struct AddEpisodeCli {
    /// Series name to add the episode to
    pub series: String,

    /// Season number associated
    pub season: u32,

    /// The episode number or range to be added
    pub episode: u32,       
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