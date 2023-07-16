//! Series Troxide module for handling command-line arguments

pub mod handle_cli {
    //! Handlers for command-line argument parsing

    use super::cli_data::*;

    /// Handles all the logic for the command line arguments
    pub fn handle_cli(_command: Command) {}
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

        /// Exports Series Troxide series tracking data
        ExportData {
            /// The path for writing exported data
            path_to_data: path::PathBuf,
        },
    }
}
