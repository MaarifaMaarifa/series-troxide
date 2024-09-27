use tracing::info;

use crate::core::paths;

pub mod database_transfer;
pub mod db_models;
pub mod series_tree;

// The last digit represents the version of the database.
const DATABASE_FOLDER_NAME: &str = "series-troxide-db-1";

pub fn open_database() -> anyhow::Result<sled::Db> {
    let mut database_path = paths::PATHS
        .read()
        .expect("failed to read paths")
        .get_data_dir_path()
        .to_path_buf();

    info!("initializing database at {}", database_path.display());

    database_path.push(DATABASE_FOLDER_NAME);

    let db = sled::open(database_path).unwrap();

    if !db.was_recovered() {
        info!("created a fresh database as none was found");
    }

    Ok(db)
}
