use thiserror::Error;

pub mod import_shows {
    //! Import user shows from Trakt api
    use std::mem::ManuallyDrop;

    use thiserror::Error;
    use tokio::sync::mpsc;

    pub use super::convert::ProgressData;

    use super::trakt_data::{TraktShow, TraktStatusCode};
    use super::ApiError as TraktApiError;
    use crate::core::api::tv_maze::ApiError as TvMazeApiError;
    use crate::core::database::Series;
    use reqwest::header::HeaderValue;

    const USER_WATCHED_SHOWS_ADDRESS: &str = "https://api.trakt.tv/users/SLUG/watched/shows";

    #[derive(Debug, Error)]
    pub enum ImportError {
        #[error("tvmaze api error: {0}")]
        TvMazeApi(TvMazeApiError),
        #[error("trakt api error: {0}")]
        TraktApi(TraktApiError),
    }

    /// Fetches shows watched by a user based on their trakt slug
    async fn fetch_user_shows(
        slug: &str,
        client_id: &'static str,
    ) -> Result<Vec<TraktShow>, super::ApiError> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("trakt-api-version", HeaderValue::from_static("2"));
        headers.insert("trakt-api-key", HeaderValue::from_static(client_id));

        let url = USER_WATCHED_SHOWS_ADDRESS.replace("SLUG", slug);

        let pretty_json_str =
            super::get_pretty_json_from_url(url, headers, TraktStatusCode::Success).await?;

        super::deserialize_json(&pretty_json_str)
    }

    pub async fn import(
        slug: &str,
        client_id: &'static str,
        progress_sender: mpsc::Sender<ProgressData>,
    ) -> Result<(Vec<(u32, ManuallyDrop<Series>)>, Vec<TraktShow>), ImportError> {
        let trakt_shows = fetch_user_shows(slug, client_id)
            .await
            .map_err(ImportError::TraktApi)?;
        super::convert::convert_trakt_shows_to_troxide(trakt_shows, progress_sender)
            .await
            .map_err(ImportError::TvMazeApi)
    }
}

mod convert {
    //! Convert Trakt Shows to SerieTroxides' database Shows

    use std::mem::ManuallyDrop;

    use tokio::sync::mpsc;

    use super::trakt_data::TraktShow;
    use crate::core::api::tv_maze::show_lookup::{show_lookup, Id};
    use crate::core::api::tv_maze::ApiError as TvMazeApiError;
    use crate::core::caching::series_information::cache_series_information;
    use crate::core::database::Series;

    async fn trakt_show_to_troxide(
        trakt_show: &TraktShow,
    ) -> Result<Option<(u32, ManuallyDrop<Series>)>, TvMazeApiError> {
        let imdb_id = Id::Imdb(trakt_show.show.ids.imdb.clone());

        let tvmaze_series_info = if let Some(tvmaze_series_info) = show_lookup(imdb_id).await? {
            tvmaze_series_info
        } else {
            // Falling back to the tvdb id when imdb id fails
            let tvdb_id = Id::Tvdb(trakt_show.show.ids.tvdb);

            if let Some(tvmaze_series_info) = show_lookup(tvdb_id).await? {
                tvmaze_series_info
            } else {
                return Ok(None);
            }
        };

        let tvmaze_series_id = tvmaze_series_info.id;

        // Caching the series information
        let series_info_str = serde_json::to_string_pretty(&tvmaze_series_info)
            .expect("SeriesMainInformation should be seriealizable");
        cache_series_information(tvmaze_series_id, &series_info_str).await;

        let mut troxide_db_series =
            ManuallyDrop::new(Series::new(tvmaze_series_info.name, tvmaze_series_id));

        trakt_show.seasons.iter().for_each(|season| {
            let season_number = season.number;
            season.episodes.iter().for_each(|episode| {
                troxide_db_series.add_episode_unchecked(season_number, episode.number)
            })
        });

        Ok(Some((tvmaze_series_id, troxide_db_series)))
    }

    #[derive(Debug)]
    pub enum ProgressData {
        /// Signals Total import amount
        TotalImport(usize),
        /// Signals an import has completed
        Progressing,
    }

    /// Converts given `TraktShow`s to Series Troxide's database `Series`
    ///
    /// Since Conversion might fail, failed `TraktShow`s will be returned too
    pub async fn convert_trakt_shows_to_troxide(
        trakt_shows: Vec<TraktShow>,
        progress_sender: mpsc::Sender<ProgressData>,
    ) -> Result<(Vec<(u32, ManuallyDrop<Series>)>, Vec<TraktShow>), TvMazeApiError> {
        progress_sender
            .send(ProgressData::TotalImport(trakt_shows.len()))
            .await
            .expect("failed to send progress");

        let mut ids_and_series = Vec::with_capacity(trakt_shows.len());
        let mut failed = Vec::with_capacity(trakt_shows.len() / 2);

        let handles: Vec<_> = trakt_shows
            .iter()
            .cloned()
            .map(|show| {
                let progress_sender = progress_sender.clone();
                tokio::spawn(async move {
                    let res = trakt_show_to_troxide(&show).await;
                    if let Err(err) = progress_sender.send(ProgressData::Progressing).await {
                        tracing::warn!("failed to send import progress as: {}", err);
                    };
                    res
                })
            })
            .collect();

        for (show, handle) in trakt_shows.into_iter().zip(handles.into_iter()) {
            if let Some(id_and_series) = handle.await.expect("failed to join all the handles")? {
                ids_and_series.push(id_and_series);
            } else {
                failed.push(show)
            }
        }

        Ok((ids_and_series, failed))
    }
}

pub mod user_settings {
    //! Get user settings for a user's trakt account

    use super::trakt_data::TraktStatusCode;
    use super::{deserialize_json, get_pretty_json_from_url, ApiError};
    use reqwest::header::HeaderValue;
    use serde::Deserialize;

    const USER_SETTINGS_ADDRESS: &str = "https://api.trakt.tv/users/settings";

    #[derive(Debug, Deserialize, Clone)]
    pub struct UserSettings {
        pub user: User,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct User {
        pub username: String,
        pub ids: Ids,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct Ids {
        pub slug: String,
        pub uuid: String,
    }

    /// Retrieves the user settings from a user's trakt account
    pub async fn get_user_settings(
        client_id: &'static str,
        access_token: String,
    ) -> Result<UserSettings, ApiError> {
        let authorization = format!("Bearer {}", access_token).leak();

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("Authorization", HeaderValue::from_static(authorization));
        headers.insert("trakt-api-version", HeaderValue::from_static("2"));
        headers.insert("trakt-api-key", HeaderValue::from_static(client_id));

        let pretty_json_str = get_pretty_json_from_url(
            USER_SETTINGS_ADDRESS.to_owned(),
            headers,
            TraktStatusCode::Success,
        )
        .await?;

        deserialize_json(&pretty_json_str)
    }
}

pub mod user_credentials {
    use std::env::VarError;

    use chrono::Local;
    use directories::ProjectDirs;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;
    use tokio::fs;

    use super::{authenication::TokenResponse, user_settings::UserSettings, ApiError};

    const CREDENTIALS_FILENAME: &str = "credentials";

    #[derive(Debug, Error)]
    pub enum CredentialsError {
        #[error("the current stored token has expired")]
        ExpiredAccessToken,

        #[error("there is no available token")]
        TokenNotFound,

        #[error("there is no user details")]
        UserDetailsNotFound,

        #[error("could not read environment variable '{0}': '{1}'")]
        EnvironmentVariable(&'static str, VarError),

        #[error("api error '{0}'")]
        Api(ApiError),

        #[error("filesystem error '{0}'")]
        Io(std::io::Error),

        #[error("credentials filepath could not be determined")]
        UndeterminedCredentialsFilepath,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct Credentials {
        user: Option<User>,
        token: Option<Token>,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct User {
        pub username: String,
        pub slug: String,
    }

    impl From<UserSettings> for User {
        fn from(value: UserSettings) -> Self {
            Self {
                username: value.user.username,
                slug: value.user.ids.slug,
            }
        }
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct Client {
        pub client_id: String,
        pub client_secret: String,
    }

    impl Client {
        /// Loads the `Client` from the Environment Variables
        pub fn new() -> Result<Self, CredentialsError> {
            use std::env;

            let client_id = env::var("CLIENT-ID")
                .map_err(|err| CredentialsError::EnvironmentVariable("CLIENT-ID", err))?;
            let client_secret = env::var("CLIENT-SECRET")
                .map_err(|err| CredentialsError::EnvironmentVariable("CLIENT-SECRET", err))?;

            Ok(Self {
                client_id,
                client_secret,
            })
        }

        /// Sets the `Client ID` and `Client Secret` environment variables for the currently running process
        pub fn set_vars(client_id: &str, client_secret: &str) {
            use std::env;

            env::set_var("CLIENT-ID", client_id);
            env::set_var("CLIENT-SECRET", client_secret);
        }
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct Token {
        pub access_token: String,
        pub refresh_token: String,
        pub token_expiration_secs: i64,
        pub creation_time: chrono::DateTime<Local>,
    }

    impl Token {
        pub fn get_access_token(&self) -> Result<&str, CredentialsError> {
            let current_time = chrono::Local::now();
            let duration_since_creation = current_time - self.creation_time;

            let token_expiration_duration = chrono::Duration::seconds(self.token_expiration_secs);

            if duration_since_creation > token_expiration_duration {
                Err(CredentialsError::ExpiredAccessToken)
            } else {
                Ok(&self.access_token)
            }
        }
    }

    impl From<TokenResponse> for Token {
        fn from(value: TokenResponse) -> Self {
            Self {
                access_token: value.access_token,
                refresh_token: value.refresh_token,
                token_expiration_secs: value.expires_in,
                creation_time: chrono::Local::now(),
            }
        }
    }

    impl Credentials {
        /// Construct Credentials from it's data
        pub fn new(token: Token, user: User) -> Self {
            Self {
                token: Some(token),
                user: Some(user),
            }
        }

        /// Loads the User credentials from a  file
        pub async fn load_from_file() -> Result<Self, CredentialsError> {
            let credentials_filepath = Self::credentials_filepath()
                .ok_or(CredentialsError::UndeterminedCredentialsFilepath)?;

            fs::read_to_string(&credentials_filepath)
                .await
                .map(|file_content| {
                    serde_json::from_str(&file_content)
                        .expect("file content should be a valid json")
                })
                .map_err(CredentialsError::Io)
        }

        /// Get the filepath of the credentials file
        fn credentials_filepath() -> Option<std::path::PathBuf> {
            ProjectDirs::from("", "", env!("CARGO_PKG_NAME")).map(|proj_dir| {
                let mut credentials_filepath = std::path::PathBuf::from(&proj_dir.data_dir());
                credentials_filepath.push(CREDENTIALS_FILENAME);
                credentials_filepath
            })
        }

        /// Save the credentials to the filesystem
        pub async fn save_credentials(&self) -> Result<(), CredentialsError> {
            let credentials_filepath = Self::credentials_filepath()
                .ok_or(CredentialsError::UndeterminedCredentialsFilepath)?;
            fs::write(
                credentials_filepath,
                serde_json::to_string_pretty(self).expect("credentials should be serializable"),
            )
            .await
            .map_err(CredentialsError::Io)
        }

        /// Removes the `Credentials` by removing it's saved file
        pub async fn remove_credentials() -> Result<(), CredentialsError> {
            let credentials_filepath = Self::credentials_filepath()
                .ok_or(CredentialsError::UndeterminedCredentialsFilepath)?;
            fs::remove_file(credentials_filepath)
                .await
                .map_err(CredentialsError::Io)
        }

        /// Get the `Credential`'s data
        pub fn get_data(&self) -> Option<(&User, &Token)> {
            Some((self.user.as_ref()?, self.token.as_ref()?))
        }
    }
}

pub mod trakt_data {
    //! Data structures related to the trakt api

    use reqwest::StatusCode;
    use serde::Deserialize;

    use super::ApiError;

    #[derive(Deserialize, Debug, Clone)]
    pub struct TraktShow {
        pub show: Show,
        pub seasons: Vec<Season>,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Show {
        pub title: String,
        pub year: Option<u32>,
        pub ids: Ids,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Ids {
        pub trakt: u32,
        pub imdb: String,
        pub tvdb: u32,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Season {
        pub number: u32,
        pub episodes: Vec<Episode>,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Episode {
        pub number: u32,
        // last_watched_at: String,
    }

    /// StatusCodes returned by the Trakt api
    #[derive(Debug, PartialEq)]
    pub enum TraktStatusCode {
        Success,
        /// new resource created (POST)
        PostSuccess,
        /// no content to return (DELETE)        
        DeleteSuccess,
        /// request couldn`t be parsed        
        BadRequest,
        /// OAuth must be provided        
        Unauthorized,
        /// invalid API key or unapproved app        
        Forbidden,
        /// method exists, but no record found        
        NotFound,
        /// method doesn't exist        
        MethodNotFound,
        /// resource already created        
        Conflict,
        /// use application/json content type        
        PreconditionFailed,
        /// list count, item count, etc        
        AccountLimitExceeded,
        /// validation errors        
        UnprocessableEntity,
        /// have the user contact support        
        LockedUserAccount,
        /// user must upgrade to VIP        
        VIPOnly,
        RateLimitExceeded,
        /// please open a support ticket        
        ServerError,
        /// server overloaded (try again in 30s)        
        ServiceUnavailable502,
        /// server overloaded (try again in 30s)        
        ServiceUnavailable503,
        /// server overloaded (try again in 30s)        
        ServiceUnavailable504,
        /// Cloudflare error        
        ServiceUnavailable520,
        /// Cloudflare error        
        ServiceUnavailable521,
        /// Cloudflare error        
        ServiceUnavailable522,
        /// Manually added incase if StatusCode is unknown
        Unknown,
    }

    impl TraktStatusCode {
        /// Returns an error if the the supplied `TraktStatusCode` is different
        pub fn error_if_different(
            &self,
            trakt_status_code: TraktStatusCode,
        ) -> Result<(), ApiError> {
            if *self != trakt_status_code {
                Err(ApiError::InvalidStatusCode(trakt_status_code))
            } else {
                Ok(())
            }
        }
    }

    impl From<StatusCode> for TraktStatusCode {
        fn from(value: StatusCode) -> Self {
            match value.into() {
                200 => Self::Success,
                201 => Self::PostSuccess,
                204 => Self::DeleteSuccess,
                400 => Self::BadRequest,
                401 => Self::Unauthorized,
                403 => Self::Forbidden,
                404 => Self::NotFound,
                405 => Self::MethodNotFound,
                409 => Self::Conflict,
                412 => Self::PreconditionFailed,
                420 => Self::AccountLimitExceeded,
                422 => Self::UnprocessableEntity,
                423 => Self::LockedUserAccount,
                426 => Self::VIPOnly,
                429 => Self::RateLimitExceeded,
                500 => Self::ServerError,
                502 => Self::ServiceUnavailable502,
                503 => Self::ServiceUnavailable503,
                504 => Self::ServiceUnavailable504,
                520 => Self::ServiceUnavailable520,
                521 => Self::ServiceUnavailable521,
                522 => Self::ServiceUnavailable522,
                _ => Self::Unknown,
            }
        }
    }

    impl std::fmt::Display for TraktStatusCode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let display_str = match self {
                TraktStatusCode::Success => "Success",
                TraktStatusCode::PostSuccess => "PostSuccess",
                TraktStatusCode::DeleteSuccess => "DeleteSuccess",
                TraktStatusCode::BadRequest => "BadRequest",
                TraktStatusCode::Unauthorized => "Unauthorized",
                TraktStatusCode::Forbidden => "Forbidden",
                TraktStatusCode::NotFound => "NotFound",
                TraktStatusCode::MethodNotFound => "MethodNotFound",
                TraktStatusCode::Conflict => "Conflict",
                TraktStatusCode::PreconditionFailed => "PreconditionFailed",
                TraktStatusCode::AccountLimitExceeded => "AccountLimitExceeded",
                TraktStatusCode::UnprocessableEntity => "UnprocessableEntity",
                TraktStatusCode::LockedUserAccount => "LockedUserAccount",
                TraktStatusCode::VIPOnly => "VIPOnly",
                TraktStatusCode::RateLimitExceeded => "RateLimitExceeded",
                TraktStatusCode::ServerError => "ServerError",
                TraktStatusCode::ServiceUnavailable502 => "ServiceUnavailable502",
                TraktStatusCode::ServiceUnavailable503 => "ServiceUnavailable503",
                TraktStatusCode::ServiceUnavailable504 => "ServiceUnavailable504",
                TraktStatusCode::ServiceUnavailable520 => "ServiceUnavailable520",
                TraktStatusCode::ServiceUnavailable521 => "ServiceUnavailable521",
                TraktStatusCode::ServiceUnavailable522 => "ServiceUnavailable522",
                TraktStatusCode::Unknown => "Unknown",
            };
            write!(f, "{}", display_str)
        }
    }
}

pub mod authenication {
    //! Authenticate the program to access user's trakt account

    use super::{trakt_data::TraktStatusCode, ApiError};
    use reqwest::header::HeaderValue;
    use serde::{Deserialize, Serialize};
    use tokio::sync::mpsc;

    /// The url to retrieve device code
    const DEVICE_CODE_URL: &str = "https://api.trakt.tv/oauth/device/code";
    /// The url to retrieve user account token
    const TOKEN_URL: &str = "https://api.trakt.tv/oauth/device/token";

    /// Request body for retrieving `CodeResponse`
    #[derive(Serialize)]
    struct CodeRequestBody {
        client_id: String,
    }

    impl CodeRequestBody {
        fn new(client_id: String) -> Self {
            Self { client_id }
        }
    }

    /// Response body after retrieving `CodeResponse`
    #[derive(Deserialize, Debug, Clone)]
    pub struct CodeResponse {
        pub device_code: String,
        pub user_code: String,
        pub verification_url: String,
        pub expires_in: u32,
        pub interval: u32,
    }

    /// Request body for retrieving `TokenResponse`
    #[derive(Serialize)]
    struct TokenRequestBody {
        code: String,
        client_id: String,
        client_secret: String,
    }

    impl TokenRequestBody {
        fn new(code: String, client_id: String, client_secret: String) -> Self {
            Self {
                code,
                client_id,
                client_secret,
            }
        }
    }

    /// Response body after retrieving `TokenResponse`
    #[derive(Deserialize, Debug, Clone)]
    pub struct TokenResponse {
        pub access_token: String,
        pub token_type: String,
        pub expires_in: i64,
        pub refresh_token: String,
        // scope: String,
        // created_at: u32,
    }

    pub async fn get_token_response(
        device_code: String,
        interval: u32,
        expires_in: u32,
        client_id: String,
        client_secret: String,
        countdown_sender: mpsc::Sender<()>,
    ) -> Result<Option<TokenResponse>, ApiError> {
        let token_request_body = TokenRequestBody::new(device_code, client_id, client_secret);

        let client = reqwest::Client::new();

        let mut text = None;

        let expires_in = expires_in / interval;

        for current_second in 0..expires_in {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("Content-Type", HeaderValue::from_static("application/json"));

            let json_body = serde_json::to_string(&token_request_body)
                .expect("token reqwest body should be serializable");

            let response = client
                .post(TOKEN_URL)
                .headers(headers)
                .body(json_body)
                .send()
                .await
                .map_err(ApiError::Network)?;

            if TraktStatusCode::Success
                .error_if_different(response.status().into())
                .is_ok()
            {
                text = Some(
                    response
                        .text()
                        .await
                        .expect("failed to get text from the response"),
                );
                break;
            };

            tokio::time::sleep(std::time::Duration::from_secs(interval as u64)).await;

            println!(
                "{}/{} attempt for token request",
                current_second + 1,
                expires_in
            );

            if let Err(err) = countdown_sender.send(()).await {
                tracing::info!("stopping trakt token request as: {}", err);
                break;
            };
        }

        Ok(text.map(|text| serde_json::from_str(&text).expect("text should be valid json")))
    }

    pub async fn get_device_code_response(client_id: String) -> Result<CodeResponse, ApiError> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let json_body = serde_json::to_string(&CodeRequestBody::new(client_id)).unwrap();

        let client = reqwest::Client::new();
        let response = client
            .post(DEVICE_CODE_URL)
            .headers(headers)
            .body(json_body)
            .send()
            .await
            .map_err(ApiError::Network)?;

        TraktStatusCode::Success.error_if_different(response.status().into())?;

        let text = response
            .text()
            .await
            .expect("failed to get text from response");

        Ok(serde_json::from_str(&text).expect("text should be serializable to json"))
    }
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("network error during request")]
    Network(reqwest::Error),
    #[error("invalid status code from trakt api: '{0}'")]
    InvalidStatusCode(trakt_data::TraktStatusCode),
    #[error("trakt api error when deserializing json: unexpected '{0}'")]
    Deserialization(String, serde_json::Error),
}

pub fn deserialize_json<'a, T: serde::Deserialize<'a>>(
    prettified_json: &'a str,
) -> Result<T, ApiError> {
    serde_json::from_str::<T>(prettified_json).map_err(|err| {
        let line_number = err.line() - 1;

        let mut errored_line = String::new();
        prettified_json
            .lines()
            .skip(line_number)
            .take(1)
            .for_each(|line| errored_line = line.to_owned());
        ApiError::Deserialization(errored_line, err)
    })
}

pub async fn get_pretty_json_from_url(
    url: String,
    headers: reqwest::header::HeaderMap,
    expected_status_code: trakt_data::TraktStatusCode,
) -> Result<String, ApiError> {
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(ApiError::Network)?;

    expected_status_code.error_if_different(response.status().into())?;

    let text = response.text().await.map_err(ApiError::Network)?;

    Ok(json::stringify_pretty(
        json::parse(&text).expect("text should be valid json"),
        1,
    ))
}
