mod args;

use anyhow::{Context, Result};
use args::*;
use database::get_database_path;
use series_troxide::*;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let database_path = get_database_path().context("Could not get the database path")?;

    let mut series_collection =
        SeriesCollection::load_series(&database_path).context("Failed to load the database")?;

    match cli.command {
        Command::Episode(episode_cli) => {
            match episode_cli.episode_command {
                EpisodeCommand::Add(add_episode_cli) => {
                    series_collection
                        .get_series_mut(&add_episode_cli.series)?
                        .add_episode(add_episode_cli.season, add_episode_cli.episode)
                        .context("Could not add episode")?;
                }
                EpisodeCommand::Remove(remove_episode_cli) => {
                    series_collection
                        .get_series_mut(&remove_episode_cli.series)?
                        .remove_episode(remove_episode_cli.season, remove_episode_cli.episode)
                        .context("Could not remove episode")?;
                }
                
            }
        },
        // Command::AddEpisode(add_episode_cli) => {
        //     series_collection
        //         .get_series_mut(&add_episode_cli.series)?
        //         .add_episode(add_episode_cli.season, add_episode_cli.episode)
        //         .context("Could not add episode")?;
        // }
        // Command::RemoveEpisode(remove_episode_cli) => {
        //     series_collection
        //         .get_series_mut(&remove_episode_cli.series)?
        //         .remove_episode(remove_episode_cli.season, remove_episode_cli.episode)
        //         .context("Could not remove episode")?;
        // }
        Command::Season(season_cli) => {
            match season_cli.season_command {
                SeasonCommand::Add(add_season_cli) => {
                    series_collection
                        .get_series_mut(&add_season_cli.series)?
                        .add_season(add_season_cli.season)
                        .context("Could not add season")?;
                },
                SeasonCommand::Remove(remove_season_cli) => {
                    series_collection
                        .get_series_mut(&remove_season_cli.series)?
                        .remove_season(remove_season_cli.season)
                        .context("Could not remove season")?;
                },
            }
        },
        Command::Series(series_cli) => match series_cli.command {
            SeriesCommand::List(list_cli) => {
                let series_list;
                if let Some(sort_command) = list_cli.sort_command {
                    series_list = series_collection.get_series_names_sorted(sort_command);
                } else {
                    series_list = series_collection.get_series_names_sorted(SeriesSort::Default);
                };
                series_list.iter().for_each(|name| println!("{}", name));
            }
            SeriesCommand::Add(series_add_cli) => {
                series_collection
                    .add_series(series_add_cli.name, series_add_cli.episode_duration)
                    .context("Failed to add series")?;
            }
            SeriesCommand::Remove(series_remove_cli) => {
                series_collection
                    .remove_series(&series_remove_cli.name)
                    .context("Could not remove series")?;
            }
            SeriesCommand::Summary(series_summary_cli) => {
                println!(
                    "{}",
                    series_collection.get_summary(&series_summary_cli.name)?
                );
            }
            SeriesCommand::WatchTime(watch_time_cli) => {
                let series = series_collection.get_series(&watch_time_cli.name)?;

                match watch_time_cli.watch_time_command {
                    WatchTimeCommand::Seconds => {
                        println!("{} seconds", series.get_total_watch_time().as_secs())
                    }
                    WatchTimeCommand::Minutes => {
                        println!("{} minutes", series.get_total_watch_time().as_secs() / 60)
                    }
                    WatchTimeCommand::Hours => {
                        println!(
                            "{} hours",
                            series.get_total_watch_time().as_secs() / (60 * 60)
                        )
                    }
                    WatchTimeCommand::Days => {
                        println!(
                            "{} days",
                            series.get_total_watch_time().as_secs() / (60 * 60 * 24)
                        )
                    }
                }
            }
            SeriesCommand::TotalWatchTime(total_watch_time_cli) => {
                match total_watch_time_cli.watch_time_command {
                    WatchTimeCommand::Seconds => {
                        println!(
                            "{} seconds",
                            series_collection.get_total_watch_time().as_secs()
                        )
                    }
                    WatchTimeCommand::Minutes => {
                        println!(
                            "{} minutes",
                            series_collection.get_total_watch_time().as_secs() / 60
                        )
                    }
                    WatchTimeCommand::Hours => {
                        println!(
                            "{} hours",
                            series_collection.get_total_watch_time().as_secs() / (60 * 60)
                        )
                    }
                    WatchTimeCommand::Days => {
                        println!(
                            "{} days",
                            series_collection.get_total_watch_time().as_secs() / (60 * 60 * 24)
                        )
                    }
                }
            }
        },
    }

    series_collection
        .save_file(database_path)
        .context("Failed to save the series file")?;

    Ok(())
}

/// Module that deals with operations that involves obtaining database path
mod database {
    use super::*;
    use directories::ProjectDirs;
    use std::path;
    use thiserror::Error;

    const SERIES_DATABASE_NAME: &str = "series.ron";

    #[derive(Debug, Error)]
    pub enum DatabaseError {
        #[error("standard database path could not be found")]
        DatabasePathNotFound,
    }

    pub fn get_database_path() -> Result<path::PathBuf, DatabaseError> {
        if let Some(path) = ProjectDirs::from("", "", "series-troxide") {
            let mut path = path.data_dir().to_owned();
            path.push(SERIES_DATABASE_NAME);
            Ok(path)
        } else {
            Err(DatabaseError::DatabasePathNotFound)
        }
    }
}
