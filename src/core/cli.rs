//! Series Troxide module for handling command-line arguments

pub mod cli_handler {
    //! Handlers for command-line argument parsing

    use clap::Parser;
    use std::process::exit;

    use super::cli_data::*;
    use crate::core::database;

    /// Handles all the logic for the command line arguments
    pub fn handle_cli() -> anyhow::Result<()> {
        let cli = Cli::parse();

        if let Some(command) = cli.command {
            match command {
                Command::ImportData { file_path } => {
                    database::database_transfer::TransferData::blocking_import_to_db(file_path)?;
                    println!("data imported successfully!");
                    exit(0);
                }
                Command::ExportData {
                    file_path: path_to_data,
                } => {
                    database::database_transfer::TransferData::blocking_export_from_db(
                        path_to_data,
                    )?;
                    println!("data exported successfully!");
                    exit(0);
                }
            }
        }
        Ok(())
    }
}

pub mod cli_data {
    //! Data structures for command-line argument parsing

    use clap::{Parser, Subcommand};
    use std::path::PathBuf;

    #[derive(Parser)]
    #[command(author, version, about)]
    pub struct Cli {
        /// Custom cache folder path
        #[clap(short = 'a', long)]
        cache_folder: Option<PathBuf>,

        /// Custom data folder path
        #[clap(short, long)]
        data_folder: Option<PathBuf>,

        /// Custom config folder path
        #[clap(short, long)]
        config_folder: Option<PathBuf>,

        #[clap(subcommand)]
        pub command: Option<Command>,
    }

    #[derive(Subcommand)]
    pub enum Command {
        /// Import series data
        ImportData {
            /// Import filepath
            file_path: PathBuf,
        },

        /// Export series data
        ExportData {
            /// Export filepath
            file_path: PathBuf,
        },
    }
}
