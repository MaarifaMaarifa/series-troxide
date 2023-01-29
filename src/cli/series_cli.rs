pub use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct SeriesCli {
    #[clap(subcommand)]
    pub command: SeriesCommand,
}

#[derive(Subcommand)]
pub enum SeriesCommand {
    /// List all the current tracked Series
    List(ListCli),

    /// Add series to the collection
    Add(SeriesAddCli),

    /// Remove a whole series
    Remove(SeriesRemoveCli),

    /// Get the summary of the specified series
    Summary(SeriesSummaryCli),

    /// Get the seasons summary of the specified series
    SeasonSummary(SeasonSummaryCli),

    /// Change episode duration of the specified series
    ChangeEpisodeDuration(SeriesChangeDurationCli),

    /// Get the total watch time of a particular series
    WatchTime(WatchTimeCli),

    /// Get the total watch time of all series collection
    TotalWatchTime(TotalWatchTimeCli),
}

#[derive(Parser)]
pub struct ListCli {
    #[clap(subcommand)]
    pub sort_command: Option<series_troxide::SeriesSort>,
}

#[derive(Parser)]
pub struct SeriesAddCli {
    /// The name of the series
    pub name: String,

    /// The duration of episode in minutes
    pub episode_duration: u32,
}

#[derive(Parser)]
pub struct SeriesRemoveCli {
    /// The name of the series to remove
    pub name: String,
}

#[derive(Parser)]
pub struct SeriesChangeDurationCli {
    /// The name of the series
    pub name: String,

    /// The duration of episode in minutes
    pub episode_duration: u32,
}


#[derive(Parser)]
pub struct WatchTimeCli {
    /// The name of the series
    pub name: String,

    #[clap(subcommand)]
    pub watch_time_command: WatchTimeCommand,
}

#[derive(Parser)]
pub struct TotalWatchTimeCli {
    #[clap(subcommand)]
    pub watch_time_command: WatchTimeCommand,
}

#[derive(Parser)]
pub struct SeriesSummaryCli {
    /// Series' name
    pub name: String,
}

#[derive(Parser)]
pub struct SeasonSummaryCli {
    /// Series' name
    pub name: String,
}

#[derive(Subcommand, Clone)]
pub enum WatchTimeCommand {
    /// Watch time in seconds
    Seconds,

    /// Watch time in minutes
    Minutes,

    /// Watch time in hours
    Hours,

    /// Watch time in days
    Days,
}
