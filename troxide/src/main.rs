use anyhow::Result;
use clap::Parser;
use troxide_term::{
    cli::{Cli, Command},
    run_series_command, 
    run_season_command, 
    run_episode_command, 
    run_database_command,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Episode(episode_cli) => {
            run_episode_command(episode_cli)?
        }
        Command::Season(season_cli) => {
            run_season_command(season_cli)?
        }
        Command::Series(series_cli) => {
            run_series_command(series_cli)?
        },
        Command::Database(database_cli) => {
            run_database_command(database_cli)?
        },
    }

    Ok(())
}
