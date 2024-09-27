use super::{database_transfer, db_models};

/// Adds the given series to the database.
///
/// # Note
/// This will overwrite any previous series with the same id.
pub fn add_series(db: sled::Db, series_id: u32, series: &db_models::Series) {
    db.insert(series_id.to_string(), bincode::serialize(series).unwrap())
        .unwrap();
}

/// Removes a series in the database.
///
/// # Note
/// Does nothing when the series does not exist
pub fn remove_series(db: sled::Db, series_id: u32) {
    db.remove(series_id.to_string()).unwrap();
}

pub fn get_series(db: sled::Db, series_id: u32) -> Option<db_models::Series> {
    let series_bytes = db.get(series_id.to_string()).unwrap()?;
    Some(bincode::deserialize(&series_bytes).unwrap())
}

pub fn get_series_collection(db: sled::Db) -> Vec<db_models::Series> {
    db.iter()
        .values()
        .map(|series| {
            let series = series.unwrap();
            bincode::deserialize(&series).unwrap()
        })
        .collect()
}

pub fn get_series_id_collection(db: sled::Db) -> Vec<String> {
    db.iter()
        .keys()
        .map(|series| {
            let series = series.unwrap();
            // bincode::deserialize(&series).unwrap()
            String::from_utf8_lossy(&series).into_owned()
        })
        .collect()
}

/// get series ids and their corresponding series structures
pub fn get_ids_and_series(db: sled::Db) -> Vec<(String, db_models::Series)> {
    db.iter()
        .map(|tup| {
            let (series_id, series) = tup.unwrap();
            let series_id = String::from_utf8_lossy(&series_id).into_owned();
            let series = bincode::deserialize::<db_models::Series>(&series).unwrap();
            (series_id, series)
        })
        .collect()
}

/// Returns the total number of series being tracked
pub fn get_total_series(db: sled::Db) -> usize {
    db.len()
}

/// Get the total amount of seasons watched across all
/// series in the database
pub fn get_total_seasons(db: sled::Db) -> usize {
    get_series_collection(db)
        .iter()
        .map(|series| series.get_total_seasons())
        .sum()
}

/// Get the total amount of episodes watched across all
/// series in the database
pub fn get_total_episodes(db: sled::Db) -> usize {
    get_series_collection(db)
        .iter()
        .map(|series| series.get_total_episodes())
        .sum()
}

pub fn export(db: sled::Db) -> database_transfer::TransferData {
    database_transfer::TransferData::new(get_series_collection(db))
}

pub fn import(db: sled::Db, transfer_data: &database_transfer::TransferData) {
    for series in transfer_data.get_series() {
        add_series(db.clone(), series.id(), series);
    }

    db.flush().expect("flushing database");
}
