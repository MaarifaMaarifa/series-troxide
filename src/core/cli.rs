//! Series Troxide module for handling command-line arguments

pub mod cli_handler {
    //! Handlers for command-line argument parsing

    use clap::Parser;
    use std::process::exit;

    use super::cli_data::*;
    use crate::core::database;
    use crate::core::paths;

    /// Handles all the logic for the command line arguments
    pub fn handle_cli() -> anyhow::Result<()> {
        let mut cli = Cli::parse();

        let command = cli.command.take();

        setup_custom_paths(cli);

        if let Some(command) = command {
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

    fn setup_custom_paths(cli: Cli) {
        let mut paths = paths::Paths::default();

        if let Some(data_dir_path) = cli.data_dir {
            paths.set_data_dir_path(data_dir_path)
        }

        if let Some(cache_dir_path) = cli.cache_dir {
            paths.set_cache_dir_path(cache_dir_path)
        }
        if let Some(config_dir_path) = cli.config_dir {
            paths.set_config_dir_path(config_dir_path)
        }

        paths::PATHS
            .set(paths)
            .expect("paths should be initalized only once");
    }
}

pub mod cli_data {
    //! Data structures for command-line argument parsing

    use clap::{Parser, Subcommand};
    use std::path::PathBuf;

    #[derive(Parser)]
    #[command(author, version, about)]
    pub struct Cli {
        /// Custom config directory path
        #[clap(short, long)]
        pub config_dir: Option<PathBuf>,

        /// Custom cache directory path
        #[clap(short = 'a', long)]
        pub cache_dir: Option<PathBuf>,

        /// Custom data directory path
        #[clap(short, long)]
        pub data_dir: Option<PathBuf>,

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
