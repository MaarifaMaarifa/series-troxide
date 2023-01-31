pub use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct DatabaseCli {
    #[clap(subcommand)]
    pub database_command: DatabaseCommand,
}

#[derive(Subcommand)]
pub enum DatabaseCommand {
    /// Create an empty database file
    Create,

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
