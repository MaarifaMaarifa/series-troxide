use thiserror::Error;

pub mod import_shows {
    //! Import user shows from Trakt api

    use super::trakt_data::{TraktShow, TraktStatusCode};
    use reqwest::header::HeaderValue;

    const USER_WATCHED_SHOWS_ADDRESS: &str = "https://api.trakt.tv/users/SLUG/watched/shows";

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
}

pub mod convert {
    //! Convert Trakt Shows to SerieTroxides' database Shows

    use std::sync::mpsc;

    use super::trakt_data::TraktShow;
    use crate::core::api::tv_maze::show_lookup::{show_lookup, Id};
    use crate::core::api::tv_maze::ApiError as TvMazeApiError;
    use crate::core::caching::series_information::cache_series_information;
    use crate::core::database::Series;

    async fn trakt_show_to_troxide(
        trakt_show: &TraktShow,
    ) -> Result<Option<(u32, Series)>, TvMazeApiError> {
        let imdb_id = Id::Imdb(trakt_show.show.ids.imdb.clone());

        let tvmaze_series_info = if let Some(tvmaze_series_info) = show_lookup(imdb_id).await? {
            tvmaze_series_info
        } else {
            return Ok(None);
        };

        let tvmaze_series_id = tvmaze_series_info.id;

        // Caching the series information
        let series_info_str = serde_json::to_string_pretty(&tvmaze_series_info)
            .expect("SeriesMainInformation should be seriealizable");
        cache_series_information(tvmaze_series_id, &series_info_str).await;

        let mut troxide_db_series = Series::new(tvmaze_series_info.name, tvmaze_series_id);

        trakt_show.seasons.iter().for_each(|season| {
            let season_number = season.number;
            season.episodes.iter().for_each(|episode| {
                troxide_db_series.add_episode_unchecked(season_number, episode.number)
            })
        });

        Ok(Some((tvmaze_series_id, troxide_db_series)))
    }

    /// Converts given `TraktShow`s to Series Troxide's database `Series`
    ///
    /// Since Conversion might fail, failed `TraktShow`s will be returned too
    pub async fn convert_trakt_shows_to_troxide(
        trakt_shows: Vec<TraktShow>,
        mut progress_sender: Option<mpsc::Sender<()>>,
    ) -> Result<(Vec<(u32, Series)>, Vec<TraktShow>), TvMazeApiError> {
        let mut ids_and_series = Vec::with_capacity(trakt_shows.len());
        let mut failed = Vec::with_capacity(trakt_shows.len() / 2);

        for show in trakt_shows {
            // Avoiding using tokio::spawn to avoid overwhelming the tvmaze api, as trakt import can be very huge
            if let Some(id_and_series) = trakt_show_to_troxide(&show).await? {
                ids_and_series.push(id_and_series);
            } else {
                failed.push(show)
            }
            if let Some(progress_sender) = progress_sender.as_mut() {
                progress_sender.send(()).expect("failed to send progress");
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

    #[derive(Debug, Deserialize)]
    pub struct UserSettings {
        pub user: User,
    }

    #[derive(Debug, Deserialize)]
    pub struct User {
        pub username: String,
        pub ids: Ids,
    }

    #[derive(Debug, Deserialize)]
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

mod user_credentials {
    use std::env::VarError;

    use chrono::Local;
    use directories::ProjectDirs;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;
    use tokio::fs;

    use super::ApiError;

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
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Credentials {
        user: Option<User>,
        client: Option<Client>,
        token: Option<Token>,
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    struct User {
        username: String,
        slug: String,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    struct Client {
        client_id: String,
        client_secret: String,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct Token {
        pub access_token: String,
        pub refresh_token: String,
        pub token_expiration_secs: i64,
        pub creation_time: chrono::DateTime<Local>,
    }

    impl Credentials {
        async fn new() -> anyhow::Result<Self> {
            if let Some(proj_dir) = ProjectDirs::from("", "", env!("CARGO_PKG_NAME")) {
                let mut credentials_filepath = std::path::PathBuf::from(&proj_dir.data_dir());
                credentials_filepath.push(CREDENTIALS_FILENAME);

                match fs::read_to_string(&credentials_filepath).await {
                    Ok(file_content) => {
                        return Ok(serde_json::from_str(&file_content)
                            .expect("file content should be a valid json"));
                    }
                    Err(err) => {
                        if let std::io::ErrorKind::NotFound = err.kind() {
                            let credentials = Credentials::default();
                            fs::write(
                                credentials_filepath,
                                serde_json::to_string_pretty(&credentials)
                                    .expect("credentials should be serializable"),
                            )
                            .await?;
                            return Ok(credentials);
                        } else {
                            anyhow::bail!(err)
                        }
                    }
                }
            }
            anyhow::bail!("could not get credentials path");
        }

        /// Loads all the fields in the `Credentials` in a proper sequence without panic
        pub async fn full_load(&mut self) -> Result<(), CredentialsError> {
            self.load_client().await?;
            self.load_token().await?;
            self.load_user_details().await?;
            Ok(())
        }

        /// Loads the `User` into the `Credentials`
        ///
        /// # Panics
        /// panics if the `Client` and `Token` in the `Credentials` are`None`.
        /// To prevent panic, perform `Credentials::load_client` and `Credentials::load_token` first.
        async fn load_user_details(&mut self) -> Result<(), CredentialsError> {
            use super::user_settings;

            let client_id = self.client.clone().unwrap().client_id.leak();
            let access_token = self.token.clone().unwrap().access_token;
            let user_settings = user_settings::get_user_settings(client_id, access_token)
                .await
                .map_err(CredentialsError::Api)?;

            self.user = Some(User {
                username: user_settings.user.username,
                slug: user_settings.user.ids.slug,
            });
            Ok(())
        }

        /// Loads the `Token` into the `Credentials`
        ///
        /// # Panics
        /// panics if the `Client` in the `Credentials` is `None`.
        /// To prevent panic, perform `Credentials::load_client` first.
        async fn load_token(&mut self) -> Result<(), CredentialsError> {
            use super::authenication;

            let client = self.client.clone().unwrap();

            self.token = authenication::authenticate(client.client_id, client.client_secret)
                .await
                .map_err(CredentialsError::Api)?;

            Ok(())
        }

        /// Loads the `Client` into the `Credentials`
        async fn load_client(&mut self) -> Result<(), CredentialsError> {
            use std::env;

            let client_id = env::var("CLIENT-ID")
                .map_err(|err| CredentialsError::EnvironmentVariable("CLIENT-ID", err))?;
            let client_secret = env::var("CLIENT-SECRET")
                .map_err(|err| CredentialsError::EnvironmentVariable("CLIENT-SECRET", err))?;

            self.client = Some(Client {
                client_id,
                client_secret,
            });
            Ok(())
        }

        fn get_access_token(&self) -> Result<&str, CredentialsError> {
            let current_time = chrono::Local::now();
            let token = self.token.as_ref().ok_or(CredentialsError::TokenNotFound)?;
            let duration = current_time - token.creation_time;

            let token_expiration_duration = chrono::Duration::seconds(token.token_expiration_secs);

            if duration < token_expiration_duration {
                Err(CredentialsError::ExpiredAccessToken)
            } else {
                Ok(&token.access_token)
            }
        }

        fn get_client_id(&self) -> Option<&str> {
            self.client.as_ref().map(|client| client.client_id.as_str())
        }

        fn get_user_slug(&self) -> Option<&str> {
            self.user.as_ref().map(|user| user.slug.as_str())
        }
    }
}

pub mod trakt_data {
    //! Data structures related to the trakt api

    use reqwest::StatusCode;
    use serde::Deserialize;

    use super::ApiError;

    #[derive(Deserialize, Debug)]
    pub struct TraktShow {
        pub show: Show,
        pub seasons: Vec<Season>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Show {
        title: String,
        // year: u32,
        pub ids: Ids,
    }

    #[derive(Deserialize, Debug)]
    pub struct Ids {
        pub trakt: u32,
        pub imdb: String,
        pub tvdb: u32,
    }

    #[derive(Deserialize, Debug)]
    pub struct Season {
        pub number: u32,
        pub episodes: Vec<Episode>,
    }

    #[derive(Deserialize, Debug)]
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

mod authenication {
    //! Authenticate the program to access user's trakt account

    use super::user_credentials::{self, Token};
    use super::{trakt_data::TraktStatusCode, ApiError};
    use reqwest::header::HeaderValue;
    use serde::{Deserialize, Serialize};

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
    #[derive(Deserialize, Debug)]
    struct CodeResponse {
        device_code: String,
        user_code: String,
        verification_url: String,
        expires_in: u32,
        interval: u32,
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
    #[derive(Deserialize, Debug)]
    struct TokenResponse {
        pub access_token: String,
        pub token_type: String,
        pub expires_in: i64,
        pub refresh_token: String,
        // scope: String,
        // created_at: u32,
    }

    async fn get_token_response(
        device_code: String,
        interval: u32,
        expires_in: u32,
        client_id: String,
        client_secret: String,
    ) -> Result<Option<TokenResponse>, ApiError> {
        let token_request_body = TokenRequestBody::new(device_code, client_id, client_secret);

        let client = reqwest::Client::new();

        let mut text = None;

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
                text = Some(response.text().await.unwrap());
                break;
            };

            println!("{}/{} attempt", current_second + 1, expires_in);

            tokio::time::sleep(std::time::Duration::from_secs(interval as u64)).await;
        }

        Ok(text.map(|text| serde_json::from_str(&text).expect("text should be valid json")))
    }

    async fn get_device_code_response(client_id: String) -> CodeResponse {
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
            .unwrap();

        let text = response.text().await.unwrap();

        serde_json::from_str(&text).unwrap()
    }

    /// Authenticates the program returning `Token` facilitating import and export of user tracked shows in trakt
    pub async fn authenticate(
        client_id: String,
        client_secret: String,
    ) -> Result<Option<Token>, ApiError> {
        let code_response = get_device_code_response(client_id.clone()).await;

        println!("code response:\n{:#?}", code_response);

        let token = get_token_response(
            code_response.device_code,
            code_response.interval,
            code_response.expires_in,
            client_id,
            client_secret,
        )
        .await?
        .map(|token_response| user_credentials::Token {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            token_expiration_secs: token_response.expires_in,
            creation_time: chrono::Local::now(),
        });

        Ok(token)
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
