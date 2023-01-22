
use super::*;
use anyhow::anyhow;
use directories::ProjectDirs;
use std::{path, fs};
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

/// Exports the database to the given directory
pub fn export_database(mut destination_dir: path::PathBuf) -> Result<()>{
    let database_path = get_database_path()?;

    destination_dir.push(SERIES_DATABASE_NAME);

    std::fs::copy(database_path, destination_dir)?;
    Ok(())
}

/// Imports the database file from the given file path
pub fn import_database(import_file_path: &path::Path) -> Result<()> {     
    // Inspecting the file if it is a valid database file by try parsing it into 
    // a series collection struct
    let file_contents = fs::read_to_string(import_file_path)?;

    match SeriesCollection::load_series_with_db_content(&file_contents) {
        Ok(_) => {
            // when we successfully get a valid SeriesCollection struct, we can copy
            // it to the database path
            fs::copy(import_file_path, get_database_path()?).context("Could not copy the database to the default path")?;
            Ok(())
        },
        Err(err) => {
            Err(anyhow!(err))
        },
    }
}
