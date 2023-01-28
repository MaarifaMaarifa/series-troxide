mod cli;
mod database;

use std::num::ParseIntError;
use anyhow::{Context, Result};
use thiserror::Error;
use cli::*;
use database::*;
use series_troxide::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let database_path = get_database_path().context("Could not get the database path")?;

    let mut series_collection =
        SeriesCollection::load_series_with_db_path(&database_path).context("Failed to load the database")?;

    match cli.command {
        Command::Episode(episode_cli) => {
            match episode_cli.episode_command {
                episode_cli::EpisodeCommand::Add(add_episode_cli) => {
                    series_collection
                        .get_series_mut(&add_episode_cli.series)?
                        .add_episode(add_episode_cli.season, add_episode_cli.episode)
                        .context("Could not add episode")?;
                }
                episode_cli::EpisodeCommand::Remove(remove_episode_cli) => {
                    series_collection
                        .get_series_mut(&remove_episode_cli.series)?
                        .remove_episode(remove_episode_cli.season, remove_episode_cli.episode)
                        .context("Could not remove episode")?;
                }
                episode_cli::EpisodeCommand::List(list_episode_cli) => {
                    let episodes = series_collection
                        .get_series(&list_episode_cli.series)?
                        .get_episodes(list_episode_cli.season).context("Could not list episodes")?;

                    episodes.iter().for_each(|episode| print!("{} ", episode));
                    println!();
                },
                episode_cli::EpisodeCommand::AddRange(add_episode_range_cli) => {
                    let episode_range = RangeParser::get_range(&add_episode_range_cli.episode_range)?;
                    series_collection
                        .get_series_mut(&add_episode_range_cli.series)?
                        .add_episode_range(add_episode_range_cli.season, episode_range)
                        .context("Could not add episode range")?;
                },
            }
            series_collection
                .save_file(database_path)
                .context("Failed to save the series file")?;

        },
        Command::Season(season_cli) => {
            match season_cli.season_command {
                season_cli::SeasonCommand::Add(add_season_cli) => {
                    series_collection
                        .get_series_mut(&add_season_cli.series)?
                        .add_season(add_season_cli.season)
                        .context("Could not add season")?;
                },
                season_cli::SeasonCommand::Remove(remove_season_cli) => {
                    series_collection
                        .get_series_mut(&remove_season_cli.series)?
                        .remove_season(remove_season_cli.season)
                        .context("Could not remove season")?;
                },
            }
            series_collection
                .save_file(database_path)
                .context("Failed to save the series file")?;
        },
        Command::Series(series_cli) => match series_cli.command {
            series_cli::SeriesCommand::List(list_cli) => {
                let series_list;
                if let Some(sort_command) = list_cli.sort_command {
                    series_list = series_collection.get_series_names_sorted(sort_command);
                } else {
                    series_list = series_collection.get_series_names_sorted(SeriesSort::Default);
                };
                series_list.iter().for_each(|name| println!("{}", name));
            }
            series_cli::SeriesCommand::Add(series_add_cli) => {
                series_collection
                    .add_series(series_add_cli.name, series_add_cli.episode_duration)
                    .context("Failed to add series")?;

                series_collection
                    .save_file(database_path)
                    .context("Failed to save the series file")?;
            }
            series_cli::SeriesCommand::Remove(series_remove_cli) => {
                series_collection
                    .remove_series(&series_remove_cli.name)
                    .context("Could not remove series")?;

                series_collection
                    .save_file(database_path)
                    .context("Failed to save the series file")?;
            }
            series_cli::SeriesCommand::Summary(series_summary_cli) => {
                println!(
                    "{}",
                    series_collection.get_summary(&series_summary_cli.name)?
                );
            }
            series_cli::SeriesCommand::WatchTime(watch_time_cli) => {
                let series = series_collection.get_series(&watch_time_cli.name)?;

                match watch_time_cli.watch_time_command {
                    series_cli::WatchTimeCommand::Seconds => {
                        println!("{:.2} seconds", series.get_total_watch_time().as_secs() as f32)
                    }
                    series_cli::WatchTimeCommand::Minutes => {
                        println!("{:.2} minutes", series.get_total_watch_time().as_secs() as f32 / 60.0)
                    }
                    series_cli::WatchTimeCommand::Hours => {
                        println!(
                            "{:.2} hours",
                            series.get_total_watch_time().as_secs() as f32 / (60 * 60) as f32
                        )
                    }
                    series_cli::WatchTimeCommand::Days => {
                        println!(
                            "{:.2} days",
                            series.get_total_watch_time().as_secs() as f32 / (60 * 60 * 24) as f32
                        )
                    }
                }
            }
            series_cli::SeriesCommand::TotalWatchTime(total_watch_time_cli) => {
                match total_watch_time_cli.watch_time_command {
                    series_cli::WatchTimeCommand::Seconds => {
                        println!(
                            "{:.2} seconds",
                            series_collection.get_total_watch_time().as_secs() as f32
                        )
                    }
                    series_cli::WatchTimeCommand::Minutes => {
                        println!(
                            "{:.2} minutes",
                            series_collection.get_total_watch_time().as_secs() as f32 / 60.0
                        )
                    }
                    series_cli::WatchTimeCommand::Hours => {
                        println!(
                            "{:.2} hours",
                            series_collection.get_total_watch_time().as_secs() as f32 / (60 * 60) as f32
                        )
                    }
                    series_cli::WatchTimeCommand::Days => {
                        println!(
                            "{:.2} days",
                            series_collection.get_total_watch_time().as_secs() as f32 / (60 * 60 * 24) as f32
                        )
                    }
                }
            }
        },
        Command::Database(database_cli) => {
            match database_cli.database_command {
                database_cli::DatabaseCommand::Import(import_database_cli) => {
                    let file_path = std::path::Path::new(&import_database_cli.file);
                    import_database(file_path).context("Failed to import database")?
                },
                database_cli::DatabaseCommand::Export(export_database_cli) => {
                    let destination_dir = std::path::PathBuf::from(export_database_cli.folder);
                    export_database(destination_dir).context("Failed to export the database")?;
                },
            }
        },
    }

    Ok(())
}

/// Error cases that can be returned by methods in RangeParser Struct
#[derive(Debug, Error)]
enum RangeParserError {
    #[error("The string syntax is incorrect, correct form is 3-7")]
    Syntax,

    #[error("The start range number is invalid")]
    StartRange(ParseIntError),

    #[error("The end range number is invalid")]
    EndRange(ParseIntError),
}

/// Struct dealing with Parsing of ranges given by the user through the command line options
struct RangeParser;

impl RangeParser {
    /// Parses a Range out of a str
    fn get_range(range_str: &str) -> Result<std::ops::RangeInclusive<u32>, RangeParserError> {
        let range_components = range_str.split_once('-');

        let range_components = if let Some(components) = range_components {
            components
        } else {
            return Err(RangeParserError::Syntax)
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