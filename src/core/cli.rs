//! Series Troxide module for handling command-line arguments

pub mod handle_cli {
    //! Handlers for command-line argument parsing

    use crate::core::database;

    use super::cli_data::*;

    /// Handles all the logic for the command line arguments
    pub fn handle_cli(command: Command) -> anyhow::Result<()> {
        match command {
            Command::ImportData { path_to_data } => {
                database::database_transfer::read_database_from_path(&path_to_data)?;
                println!("data imported successfully");
                Ok(())
            }
            Command::ExportData {
                path_to_data,
                export_name,
            } => {
                database::database_transfer::write_database_to_path(
                    &path_to_data,
                    export_name.as_deref(),
                )?;
                println!("data exported successfully");
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
    pub struct Cli {
        #[clap(subcommand)]
        pub command: Option<Command>,
    }

    #[derive(Subcommand)]
    pub enum Command {
        /// Imports Series Troxide series tracking data
        ImportData {
            /// The path to the data to import
            path_to_data: path::PathBuf,
        },

        /// Exports Series Troxide series tracking data, overwritting
        /// any file of the same name if it exists.
        ExportData {
            /// The folder path for writing exported data
            path_to_data: path::PathBuf,

            /// An optional name given to the exported data.
            /// Defaults to "series-troxide-export" when no name given
            export_name: Option<String>,
        },
    }
}
