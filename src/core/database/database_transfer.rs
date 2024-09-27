//! Implementations of importing and exporting series tracking data

use std::{io, path};

use super::{db_models, series_tree};

use ron::ser;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const CURRENT_DATA_VERSION: u16 = 1;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("IO error: {0}")]
    Io(io::Error),
    #[error("incompatible version. Expected version {0}, found {1}")]
    Version(u16, u16),
    #[error("deserialization error: {0}")]
    Deserialization(ron::de::SpannedError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferData {
    version: u16,
    series: Vec<db_models::Series>,
}

impl TransferData {
    pub fn new(series: Vec<db_models::Series>) -> Self {
        Self {
            version: CURRENT_DATA_VERSION,
            series,
        }
    }

    fn error_when_incompatible(import_data_version: u16) -> Result<(), ImportError> {
        if import_data_version == CURRENT_DATA_VERSION {
            Ok(())
        } else {
            Err(ImportError::Version(
                CURRENT_DATA_VERSION,
                import_data_version,
            ))
        }
    }

    pub fn blocking_import(path: impl AsRef<path::Path>) -> Result<Self, ImportError> {
        let import = std::fs::read_to_string(path).map_err(ImportError::Io)?;
        let imported_data = ron::from_str::<Self>(&import).map_err(ImportError::Deserialization)?;

        Self::error_when_incompatible(imported_data.version).map(|_| imported_data)
    }

    pub fn blocking_import_to_db(
        db: sled::Db,
        path: impl AsRef<path::Path>,
    ) -> Result<(), ImportError> {
        series_tree::import(db, &Self::blocking_import(path)?);
        Ok(())
    }

    pub async fn async_import(path: impl AsRef<path::Path>) -> Result<Self, ImportError> {
        let import = tokio::fs::read_to_string(path)
            .await
            .map_err(ImportError::Io)?;
        let imported_data = ron::from_str::<Self>(&import).map_err(ImportError::Deserialization)?;

        Self::error_when_incompatible(imported_data.version).map(|_| imported_data)
    }

    pub async fn async_import_to_db(
        db: sled::Db,
        path: impl AsRef<path::Path>,
    ) -> Result<(), ImportError> {
        series_tree::import(db, &Self::async_import(path).await?);
        Ok(())
    }

    pub fn get_series(&self) -> &[db_models::Series] {
        &self.series
    }

    fn ron_str(&self) -> String {
        let pretty_config = ser::PrettyConfig::new().depth_limit(4);
        ser::to_string_pretty(self, pretty_config).expect("transfer data serialization")
    }

    pub fn blocking_export(&self, path: impl AsRef<path::Path>) -> Result<(), io::Error> {
        let ron_str = self.ron_str();
        std::fs::write(path, ron_str)
    }

    pub fn blocking_export_from_db(
        db: sled::Db,
        path: impl AsRef<path::Path>,
    ) -> Result<(), io::Error> {
        series_tree::export(db).blocking_export(path)
    }

    pub async fn async_export(&self, path: impl AsRef<path::Path>) -> Result<(), io::Error> {
        let ron_str = self.ron_str();
        tokio::fs::write(path, ron_str).await
    }

    pub async fn async_export_from_db(
        db: sled::Db,
        path: impl AsRef<path::Path>,
    ) -> Result<(), io::Error> {
        series_tree::export(db).async_export(path).await
    }
}
