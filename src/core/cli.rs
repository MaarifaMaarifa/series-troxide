//! Series Troxide module for handling command-line arguments

pub mod handle_cli {
    //! Handlers for command-line argument parsing

    use crate::core::database;

    use super::cli_data::*;

    /// Handles all the logic for the command line arguments
    pub fn handle_cli(command: Command) -> anyhow::Result<()> {
        match command {
            Command::ImportData { file_path } => {
                database::database_transfer::TransferData::import_to_db(file_path)?;
                println!("data imported successfully!");
                Ok(())
            }
            Command::ExportData {
                file_path: path_to_data,
            } => {
                database::database_transfer::TransferData::export_from_db(path_to_data)?;
                println!("data exported successfully!");
                Ok(())
            }
        }
    }
}

pub mod cli_data {
    //! Data structures for command-line argument parsing

    use clap::{Parser, Subcommand};
    use std::path;

    #[derive(Parser)]
    #[command(author, version, about)]
    pub struct Cli {
        #[clap(subcommand)]
        pub command: Option<Command>,
    }

    #[derive(Subcommand)]
    pub enum Command {
        /// Import series data
        ImportData {
            /// Import filepath
            file_path: path::PathBuf,
        },

        /// Export series data
        ExportData {
            /// Export filepath
            file_path: path::PathBuf,
        },
    }
}
