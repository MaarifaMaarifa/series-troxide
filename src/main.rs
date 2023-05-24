use api::episodes_information;
use api::seasons_list;
use api::series_information;
use api::series_searching;
use clap::Parser;
use cli::{Cli, Command};

mod api;
mod cli;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        Command::search { series_name } => {
            let series_results = api::series_searching::search_series(&series_name)?;

            for series in series_results {
                println!("{} => {}", series.show.name, series.show.id);
            }
        }
    }

    Ok(())
}
