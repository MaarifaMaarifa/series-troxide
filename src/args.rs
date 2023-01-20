pub use clap::{Parser, Subcommand};

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

#[derive(Parser)]
pub struct DatabaseCli {
    #[clap(subcommand)]
    pub database_command: DatabaseCommand,
}

#[derive(Subcommand)]
pub enum DatabaseCommand {
    /// Import series database file from a specified file path
    Import(ImportDatabaseCli),

    /// Export series database file to a specified directory
    Export(ExportDatabaseCli),
}

#[derive(Parser)]
pub struct ImportDatabaseCli {
    /// File Path to database file to be imported
    pub file: String,
}

#[derive(Parser)]
pub struct ExportDatabaseCli {
    /// Directory path to export the database file
    pub folder: String,
}

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


