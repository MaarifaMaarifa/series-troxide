use iced::widget::{
    button, column, container, horizontal_space, progress_bar, row, svg, text, text_input,
    vertical_space, Column,
};
use iced::{Alignment, Command, Element, Length, Renderer};
use iced_aw::Spinner;

use crate::core::api::trakt::authenication::{self, CodeResponse, TokenResponse};
use crate::core::api::trakt::trakt_data::TraktShow;
use crate::core::api::trakt::user_credentials::{self, Client, Credentials, CredentialsError};
use crate::core::api::trakt::user_settings::{self, UserSettings};
use crate::gui::assets::icons::TRAKT_ICON_RED;
use crate::gui::styles;

#[derive(Debug, Clone)]
pub enum Message {
    StartPage(StartPageMessage),
    ClientPage(ClientPageMessage),
    ProgramAuthenticationPage(ProgramAuthenticationPageMessage),
    ConfirmationPage(ConfirmationPageMessage),
    ImportPage(ImportPageMessage),
    LoadCredentials,
    CredentialsLoaded(Credentials),

    SyncTraktData,
    Cancel,
}

pub struct TraktIntegration {
    setup_page: Option<SetupStep>,
    sync_trakt_account: bool,
}

impl TraktIntegration {
    pub fn new() -> Self {
        Self {
            setup_page: None,
            sync_trakt_account: false,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        if let Some(setup_page) = self.setup_page.as_ref() {
            match setup_page {
                SetupStep::ProgramAuthentication(program_authentication_page) => {
                    program_authentication_page
                        .subscription()
                        .map(Message::ProgramAuthenticationPage)
                }
                SetupStep::Import(import_page) => {
                    import_page.subscription().map(Message::ImportPage)
                }
                _ => iced::Subscription::none(),
            }
        } else {
            iced::Subscription::none()
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        let mut next_page = None;

        let command = match message {
            Message::LoadCredentials => Self::load_credentials(),
            Message::CredentialsLoaded(credentials) => {
                let start_page_mode = if self.sync_trakt_account {
                    StartPageMode::TraktDataSync(Client::new().map_err(|err| err.to_string()))
                } else {
                    StartPageMode::AccountConfiguration
                };

                self.setup_page = Some(SetupStep::Start(StartPage::new(
                    credentials,
                    start_page_mode,
                )));
                self.sync_trakt_account = false;
                Command::none()
            }
            Message::Cancel => {
                self.setup_page = None;
                Command::none()
            }
            Message::StartPage(message) => {
                if let Some(SetupStep::Start(start_page)) = self.setup_page.as_mut() {
                    start_page
                        .update(message, &mut next_page)
                        .map(Message::StartPage)
                } else {
                    Command::none()
                }
            }
            Message::ClientPage(message) => {
                if let Some(SetupStep::Client(client_page)) = self.setup_page.as_mut() {
                    client_page
                        .update(message, &mut next_page)
                        .map(Message::ClientPage)
                } else {
                    Command::none()
                }
            }
            Message::ProgramAuthenticationPage(message) => {
                if let Some(SetupStep::ProgramAuthentication(program_authenication_page)) =
                    self.setup_page.as_mut()
                {
                    program_authenication_page
                        .update(message, &mut next_page)
                        .map(Message::ProgramAuthenticationPage)
                } else {
                    Command::none()
                }
            }
            Message::ConfirmationPage(message) => {
                if let Some(SetupStep::Confirmation(confirmation_page)) = self.setup_page.as_mut() {
                    confirmation_page
                        .update(message, &mut next_page)
                        .map(Message::ConfirmationPage)
                } else {
                    Command::none()
                }
            }
            Message::SyncTraktData => {
                self.sync_trakt_account = true;
                Self::load_credentials()
            }
            Message::ImportPage(message) => {
                if let Some(SetupStep::Import(import_page)) = self.setup_page.as_mut() {
                    import_page.update(message).map(Message::ImportPage)
                } else {
                    Command::none()
                }
            }
        };

        if let Some(next_page) = next_page {
            if let SetupStep::None = next_page {
                self.setup_page = None;
            } else {
                self.setup_page = Some(next_page);
            }
        };

        command
    }

    fn load_credentials() -> Command<Message> {
        Command::perform(Credentials::load_from_file(), |res| {
            let credentials = res.unwrap_or_else(|err| {
                tracing::warn!("failed to load the credentials from the file: {}", err);
                Credentials::default()
            });
            Message::CredentialsLoaded(credentials)
        })
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        if let Some(setup_page) = self.setup_page.as_ref() {
            let setup_page = match setup_page {
                SetupStep::Start(start_page) => start_page.view().map(Message::StartPage),
                SetupStep::Client(client_page) => client_page.view().map(Message::ClientPage),
                SetupStep::ProgramAuthentication(program_authentication_page) => {
                    program_authentication_page
                        .view()
                        .map(Message::ProgramAuthenticationPage)
                }
                SetupStep::Confirmation(confirmation_page) => {
                    confirmation_page.view().map(Message::ConfirmationPage)
                }
                SetupStep::Import(import_page) => import_page.view().map(Message::ImportPage),
                SetupStep::None => unreachable!("SetupStep::None is only used for setup pages to go to the start not to display a view"),
            };

            let content = column![setup_page, button("cancel").on_press(Message::Cancel),]
                .spacing(5)
                .width(Length::Fill)
                .align_items(Alignment::Center);

            container(content)
                .style(styles::container_styles::loading_container_theme())
                .width(Length::Fill)
                .center_x()
                .padding(10)
                .into()
        } else {
            row![
                text("Import series data from your Trakt account"),
                horizontal_space(Length::Fill),
                row![
                    button("Sync Trakt Data").on_press(Message::SyncTraktData),
                    button("Configure Trakt integration").on_press(Message::LoadCredentials),
                ]
                .spacing(5),
            ]
            .into()
        }
    }
}

enum SetupStep {
    None,
    Start(StartPage),
    Client(ClientPage),
    ProgramAuthentication(ProgramAuthenticationPage),
    Confirmation(ConfirmationPage),
    Import(ImportPage),
}

#[derive(Debug, Clone)]
pub enum StartPageMessage {
    ConnectAccount,
    RemoveAccount,
    AccountRemoved,
    ClientManualSetup,
    ImportTraktData,
}

enum StartPageMode {
    AccountConfiguration,
    TraktDataSync(Result<Client, String>),
}

struct StartPage {
    credentials: Credentials,
    page_mode: StartPageMode,
}

impl StartPage {
    pub fn new(credentials: Credentials, start_page_mode: StartPageMode) -> Self {
        Self {
            credentials,
            page_mode: start_page_mode,
        }
    }

    pub fn update(
        &mut self,
        message: StartPageMessage,
        next_page: &mut Option<SetupStep>,
    ) -> Command<StartPageMessage> {
        match message {
            StartPageMessage::ConnectAccount => {
                let client_page = ClientPage::new(ClientPageMode::AccountConfiguration);
                *next_page = Some(SetupStep::Client(client_page));
                Command::none()
            }
            StartPageMessage::RemoveAccount => {
                Command::perform(Credentials::remove_credentials(), |res| {
                    if let Err(err) = res {
                        tracing::error!("failed to remove credentials file: {}", err)
                    };
                    StartPageMessage::AccountRemoved
                })
            }
            StartPageMessage::AccountRemoved => {
                *next_page = Some(SetupStep::None);
                Command::none()
            }
            StartPageMessage::ClientManualSetup => {
                let client_page = ClientPage::new(ClientPageMode::SettingEnvironmentVariables);
                *next_page = Some(SetupStep::Client(client_page));
                Command::none()
            }
            StartPageMessage::ImportTraktData => match &self.page_mode {
                StartPageMode::AccountConfiguration => unreachable!(
                    "importing trakt data is only triggered in StartPageMode::TraktDataSync"
                ),
                StartPageMode::TraktDataSync(client) => {
                    let client = client.as_ref().expect("client should be ok at this point");
                    let slug = self.credentials.get_data().unwrap().0.slug.clone();
                    let import_page = ImportPage::new(client.client_id.clone(), slug);
                    *next_page = Some(SetupStep::Import(import_page));
                    Command::none()
                }
            },
        }
    }

    pub fn view(&self) -> Element<'_, StartPageMessage, Renderer> {
        let content = match &self.page_mode {
            StartPageMode::AccountConfiguration => self.content_when_configuring_account(),
            StartPageMode::TraktDataSync(client) => self.content_when_syncing_data(client),
        };

        let trakt_icon_handle = svg::Handle::from_memory(TRAKT_ICON_RED);
        let trakt_icon = svg(trakt_icon_handle).height(50);

        column![trakt_icon, content]
            .align_items(Alignment::Center)
            .spacing(5)
            .into()
    }

    fn content_when_configuring_account(&self) -> Element<'_, StartPageMessage, Renderer> {
        if let Some((user, token)) = self.credentials.get_data() {
            column![
                text("Trakt Account Status").size(18),
                row![
                    text("Username:"),
                    text(&user.username).style(styles::text_styles::accent_color_theme())
                ]
                .spacing(10),
                row![
                    text("Token Status:"),
                    match token.get_access_token() {
                        Ok(_) => text("Valid").style(styles::text_styles::green_text_theme()),
                        Err(_) => text("Expired").style(styles::text_styles::red_text_theme()),
                    }
                ]
                .spacing(10),
                button("Reconnect Trakt Account").on_press(StartPageMessage::ConnectAccount),
                button("Remove Trakt Account").on_press(StartPageMessage::RemoveAccount),
            ]
            .spacing(5)
            .align_items(Alignment::Center)
        } else {
            column![
                text("Trakt Account has not been connected yet"),
                button("Connect Trakt Account").on_press(StartPageMessage::ConnectAccount),
            ]
            .spacing(2)
            .align_items(Alignment::Center)
        }
        .into()
    }

    fn content_when_syncing_data(
        &self,
        client: &Result<Client, String>,
    ) -> Element<'_, StartPageMessage, Renderer> {
        match client {
            Ok(_) => {
                if let Some((_, token)) = self.credentials.get_data() {
                    if token.get_access_token().is_ok() {
                        button("Import Trakt data")
                            .on_press(StartPageMessage::ImportTraktData)
                            .into()
                    } else {
                        text("Trakt token has expired, reconfigure your account first").into()
                    }
                } else {
                    text("No Trakt account configured").into()
                }
            }
            Err(err) => column![
                text(err).style(styles::text_styles::red_text_theme()),
                text("In order to sync your trakt account, Client information must be configured"),
                text("through environment variables"),
                vertical_space(5),
                button("Setup client information manually")
                    .on_press(StartPageMessage::ClientManualSetup),
            ]
            .align_items(Alignment::Center)
            .into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClientPageMessage {
    ClientIdChanged(String),
    ClientSecretChanged(String),
    CodeReceived(Result<CodeResponse, String>),
    ToggleClientIdView,
    ToggleClientSecretView,
    ToggleClientInformation,
    Submit,
}

enum ClientPageMode {
    AccountConfiguration,
    SettingEnvironmentVariables,
}

struct ClientPage {
    client: Result<Client, CredentialsError>,
    client_id: String,
    client_secret: String,
    show_client_id: bool,
    show_client_secret: bool,
    show_client_information: bool,
    code_loading: bool,
    response_error: Option<String>,
    client_page_mode: ClientPageMode,
}

impl ClientPage {
    fn new(client_page_mode: ClientPageMode) -> Self {
        Self {
            client: user_credentials::Client::new(),
            client_id: String::new(),
            client_secret: String::new(),
            show_client_id: false,
            show_client_secret: false,
            show_client_information: false,
            code_loading: false,
            response_error: None,
            client_page_mode,
        }
    }

    fn update(
        &mut self,
        message: ClientPageMessage,
        next_page: &mut Option<SetupStep>,
    ) -> Command<ClientPageMessage> {
        match message {
            ClientPageMessage::ClientIdChanged(text) => {
                self.client_id = text;
            }
            ClientPageMessage::ClientSecretChanged(text) => {
                self.client_secret = text;
            }
            ClientPageMessage::Submit => {
                self.code_loading = true;

                return match self.client_page_mode {
                    ClientPageMode::AccountConfiguration => match &self.client {
                        Ok(client) => Command::perform(
                            authenication::get_device_code_response(client.client_id.clone()),
                            |res| {
                                ClientPageMessage::CodeReceived(res.map_err(|err| err.to_string()))
                            },
                        ),
                        Err(_) => Command::perform(
                            authenication::get_device_code_response(self.client_id.clone()),
                            |res| {
                                ClientPageMessage::CodeReceived(res.map_err(|err| err.to_string()))
                            },
                        ),
                    },
                    ClientPageMode::SettingEnvironmentVariables => {
                        Client::set_vars(&self.client_id, &self.client_secret);
                        *next_page = Some(SetupStep::None);
                        Command::none()
                    }
                };
            }
            ClientPageMessage::CodeReceived(code_response) => match code_response {
                Ok(code_response) => {
                    let client = if let Ok(client) = self.client.as_ref() {
                        client.clone()
                    } else {
                        Client {
                            client_id: self.client_id.clone(),
                            client_secret: self.client_secret.clone(),
                        }
                    };

                    match self.client_page_mode {
                        ClientPageMode::AccountConfiguration => {
                            *next_page = Some(SetupStep::ProgramAuthentication(
                                ProgramAuthenticationPage::new(code_response, client),
                            ));
                        }
                        ClientPageMode::SettingEnvironmentVariables => {
                            *next_page = Some(SetupStep::None);
                        }
                    };
                }
                Err(err) => self.response_error = Some(err),
            },
            ClientPageMessage::ToggleClientIdView => {
                self.show_client_id = !self.show_client_id;
            }
            ClientPageMessage::ToggleClientSecretView => {
                self.show_client_secret = !self.show_client_secret;
            }
            ClientPageMessage::ToggleClientInformation => {
                self.show_client_information = !self.show_client_information
            }
        };
        Command::none()
    }

    fn view(&self) -> Element<'_, ClientPageMessage, Renderer> {
        if let Some(error_msg) = self.response_error.as_ref() {
            text(format!("Error: {}", error_msg))
                .style(styles::text_styles::red_text_theme())
                .into()
        } else if self.code_loading {
            Spinner::new().into()
        } else {
            let button_content = match self.client.is_ok() {
                true => "continue setup",
                false => "submit",
            };

            let mut submit_button = button(button_content);

            if (!self.client_id.is_empty() && !self.client_secret.is_empty()) || self.client.is_ok()
            {
                submit_button = submit_button.on_press(ClientPageMessage::Submit)
            };

            let content = match &self.client {
                Ok(client) => {
                    let (content, button_content): (
                        Element<'_, ClientPageMessage, Renderer>,
                        &str,
                    ) = if self.show_client_information {
                        (
                            column![
                                text("client information").size(18),
                                row![
                                    text("Client ID: "),
                                    text(client.client_id.as_str())
                                        .style(styles::text_styles::accent_color_theme())
                                ]
                                .spacing(5),
                                row![
                                    text("Client Secret: "),
                                    text(client.client_secret.as_str())
                                        .style(styles::text_styles::accent_color_theme())
                                ]
                                .spacing(5)
                            ]
                            .align_items(Alignment::Center)
                            .into(),
                            "hide",
                        )
                    } else {
                        (
                            text("client information has been loaded from environment variables")
                                .into(),
                            "show",
                        )
                    };

                    column![
                        content,
                        button(button_content).on_press(ClientPageMessage::ToggleClientInformation)
                    ]
                    .spacing(5)
                    .align_items(Alignment::Center)
                }
                Err(_) => column![
                    text("Trakt client information could not be loaded from environment variables"),
                    text("manually enter your Trakt client information"),
                    Self::client_field_input(
                        "Client ID",
                        &self.client_id,
                        self.show_client_id,
                        ClientPageMessage::ClientIdChanged,
                        ClientPageMessage::ToggleClientIdView
                    ),
                    Self::client_field_input(
                        "Client Secret",
                        &self.client_secret,
                        self.show_client_secret,
                        ClientPageMessage::ClientSecretChanged,
                        ClientPageMessage::ToggleClientSecretView
                    ),
                ]
                .width(500)
                .spacing(5)
                .align_items(Alignment::Center),
            };

            column![content, submit_button]
                .align_items(Alignment::Center)
                .spacing(5)
                .into()
        }
    }

    fn client_field_input<'a, F>(
        placeholder: &'a str,
        text_input_value: &'a str,
        show_client_field: bool,
        text_input_message: F,
        button_message: ClientPageMessage,
    ) -> iced::widget::Row<'a, ClientPageMessage, Renderer>
    where
        F: 'a + Fn(String) -> ClientPageMessage,
    {
        let text_input = text_input(placeholder, text_input_value).on_input(text_input_message);

        let (button_content, text_input) = match show_client_field {
            true => ("hide", text_input),
            false => ("show", text_input.password()),
        };

        row![
            text_input,
            button(button_content)
                .style(styles::button_styles::transparent_button_with_rounded_border_theme())
                .on_press(button_message)
        ]
        .spacing(5)
    }
}

#[derive(Debug, Clone)]
pub enum ProgramAuthenticationPageMessage {
    AuthenticationEvent(code_authentication::Event),
    UserSettingsLoaded(UserSettings),
    CopyCodeToClipboard,
    OpenVerificationUrl,
}

struct ProgramAuthenticationPage {
    code: CodeResponse,
    client: Client,
    count_down: u32,
    token_response: Option<TokenResponse>,
    token_response_loaded: bool,
}
impl ProgramAuthenticationPage {
    fn new(code_response: CodeResponse, client: Client) -> Self {
        let count_down = code_response.expires_in;
        Self {
            code: code_response,
            client,
            count_down,
            token_response: None,
            token_response_loaded: false,
        }
    }

    fn subscription(&self) -> iced::Subscription<ProgramAuthenticationPageMessage> {
        code_authentication::authenticate_code()
            .map(ProgramAuthenticationPageMessage::AuthenticationEvent)
    }

    fn update(
        &mut self,
        message: ProgramAuthenticationPageMessage,
        next_page: &mut Option<SetupStep>,
    ) -> Command<ProgramAuthenticationPageMessage> {
        match message {
            ProgramAuthenticationPageMessage::AuthenticationEvent(event) => match event {
                code_authentication::Event::Ready(mut work_sender) => {
                    if !self.token_response_loaded {
                        work_sender
                            .try_send(code_authentication::Input::AuthenticateCode(
                                self.code.clone(),
                                self.client.clone(),
                            ))
                            .expect("failed to send code to the authenticator");
                    }
                }
                code_authentication::Event::WorkFinished(token) => {
                    self.token_response_loaded = true;
                    if let Some(token) = token {
                        let access_token = token.access_token.clone();
                        let client_id = self.client.client_id.clone();
                        self.token_response = Some(token);
                        return Command::perform(
                            user_settings::get_user_settings(client_id.leak(), access_token),
                            |res| {
                                ProgramAuthenticationPageMessage::UserSettingsLoaded(
                                    res.expect("failed to load user settings"),
                                )
                            },
                        );
                    }
                }
                code_authentication::Event::Progressing => self.count_down -= self.code.interval,
            },
            ProgramAuthenticationPageMessage::CopyCodeToClipboard => {
                return iced::clipboard::write(self.code.user_code.clone())
            }
            ProgramAuthenticationPageMessage::OpenVerificationUrl => {
                webbrowser::open(&self.code.verification_url).unwrap_or_else(|err| {
                    tracing::error!("failed to open trakt verification url: {}", err)
                });
            }
            ProgramAuthenticationPageMessage::UserSettingsLoaded(user_settings) => {
                *next_page = Some(SetupStep::Confirmation(ConfirmationPage::new(
                    self.token_response
                        .clone()
                        .expect("there should be token response at this point!"),
                    user_settings,
                )))
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, ProgramAuthenticationPageMessage, Renderer> {
        if self.token_response_loaded {
            if self.token_response.is_some() {
                column![Spinner::new(), text("Loading account settings"),]
                    .spacing(5)
                    .align_items(Alignment::Center)
                    .into()
            } else {
                text("could not retrieve authentication token")
                    .style(styles::text_styles::red_text_theme())
                    .into()
            }
        } else {
            column![
                row![
                    text("verification code:"),
                    text(&self.code.user_code),
                    button("copy to clipboard")
                        .style(
                            styles::button_styles::transparent_button_with_rounded_border_theme()
                        )
                        .on_press(ProgramAuthenticationPageMessage::CopyCodeToClipboard)
                ]
                .spacing(10),
                row![
                    text("visit this url to authenticate"),
                    button(text(&self.code.verification_url))
                        .style(
                            styles::button_styles::transparent_button_with_rounded_border_theme()
                        )
                        .on_press(ProgramAuthenticationPageMessage::OpenVerificationUrl)
                ]
                .spacing(10),
                text(format!("{} seconds to expiration", self.count_down))
            ]
            .spacing(5)
            .align_items(Alignment::Center)
            .into()
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfirmationPageMessage {
    SaveCredentials,
    CredentialsSaved,
}

struct ConfirmationPage {
    token_response: TokenResponse,
    user_settings: UserSettings,
}

impl ConfirmationPage {
    fn new(token_response: TokenResponse, user_settings: UserSettings) -> Self {
        Self {
            token_response,
            user_settings,
        }
    }

    fn update(
        &mut self,
        message: ConfirmationPageMessage,
        next_page: &mut Option<SetupStep>,
    ) -> Command<ConfirmationPageMessage> {
        match message {
            ConfirmationPageMessage::SaveCredentials => {
                let credentials = Credentials::new(
                    self.token_response.clone().into(),
                    self.user_settings.clone().into(),
                );

                Command::perform(async move { credentials.save_credentials().await }, |res| {
                    if let Err(err) = res {
                        tracing::error!("failed to save credentials file: {}", err)
                    };
                    ConfirmationPageMessage::CredentialsSaved
                })
            }
            ConfirmationPageMessage::CredentialsSaved => {
                *next_page = Some(SetupStep::None);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, ConfirmationPageMessage, Renderer> {
        column![
            text("Connected Trakt Account"),
            row![
                text("Username:"),
                text(&self.user_settings.user.username)
                    .style(styles::text_styles::accent_color_theme())
            ]
            .spacing(10),
            button("Confirm and Save Account Information")
                .on_press(ConfirmationPageMessage::SaveCredentials)
        ]
        .align_items(Alignment::Center)
        .spacing(5)
        .into()
    }
}

#[derive(Debug, Clone)]
pub enum ImportPageMessage {
    ImportEvent(trakt_data_import::Event),
}

struct ImportPage {
    client_id: String,
    slug: String,
    progress: (usize, usize),
    failed_imports: Vec<TraktShow>,
    // `bool` to indicate when complete, `Option` to indicate import err
    import_complete: (bool, Option<String>),
}
impl ImportPage {
    fn new(client_id: String, slug: String) -> Self {
        Self {
            client_id,
            slug,
            progress: (0, 0),
            failed_imports: vec![],
            import_complete: (false, None),
        }
    }

    fn subscription(&self) -> iced::Subscription<ImportPageMessage> {
        trakt_data_import::import_trakt_data().map(ImportPageMessage::ImportEvent)
    }

    fn update(&mut self, message: ImportPageMessage) -> Command<ImportPageMessage> {
        match message {
            ImportPageMessage::ImportEvent(event) => match event {
                trakt_data_import::Event::Ready(mut work_sender) => {
                    if !self.import_complete.0 {
                        work_sender
                            .try_send(trakt_data_import::Input::new(
                                self.client_id.clone(),
                                self.slug.clone(),
                            ))
                            .expect("failed to start trakt data import")
                    }
                }
                trakt_data_import::Event::WorkFinished(imports) => {
                    use crate::core::database::DB;

                    match imports {
                        Ok(imports) => {
                            imports.0.into_iter().for_each(|(series_id, mut series)| {
                                series.mark_tracked();
                                DB.add_series(series_id, &series)
                            });
                            self.failed_imports = imports.1;
                        }
                        Err(err) => self.import_complete.1 = Some(err),
                    }
                    self.import_complete.0 = true;
                }
                trakt_data_import::Event::Progressing(total_imports, current_import) => {
                    self.progress = (total_imports, current_import)
                }
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, ImportPageMessage> {
        let total_imports = self.progress.0;
        let current_import = self.progress.1;

        if (total_imports != current_import) && !self.import_complete.0 {
            row![
                text("importing").size(11),
                progress_bar(0.0..=total_imports as f32, current_import as f32,).height(13),
                text(format!("{} / {}", current_import, total_imports)).size(11)
            ]
            .spacing(10)
            .into()
        } else if !self.failed_imports.is_empty() && self.import_complete.0 {
            let failed_imports = Column::with_children(
                self.failed_imports
                    .iter()
                    .map(|trakt_show| {
                        text(format!(
                            "{} ({})",
                            trakt_show.show.title, trakt_show.show.year
                        ))
                        .into()
                    })
                    .collect::<Vec<Element<'_, ImportPageMessage, Renderer>>>(),
            )
            .align_items(Alignment::Center)
            .spacing(3);

            column![
                text("Failed imports")
                    .size(18)
                    .style(styles::text_styles::red_text_theme()),
                failed_imports,
            ]
            .align_items(Alignment::Center)
            .spacing(5)
            .into()
        } else if !self.import_complete.0 {
            text("importing...").into()
        } else {
            if let Some(err) = &self.import_complete.1 {
                text(err).style(styles::text_styles::red_text_theme())
            } else {
                text("import successful").style(styles::text_styles::green_text_theme())
            }
            .into()
        }
    }
}

mod code_authentication {
    use crate::core::api::trakt::authenication::{get_token_response, CodeResponse, TokenResponse};
    use crate::core::api::trakt::user_credentials::Client;

    use iced::futures::channel::mpsc;
    use iced::futures::sink::SinkExt;
    use iced::subscription::{self, Subscription};

    #[derive(Debug, Clone)]
    pub enum Event {
        Ready(mpsc::Sender<Input>),
        WorkFinished(Option<TokenResponse>),
        Progressing,
    }

    #[derive(Debug, Clone)]
    pub enum Input {
        AuthenticateCode(CodeResponse, Client),
    }

    enum State {
        Starting,
        Ready(mpsc::Receiver<Input>),
    }

    pub fn authenticate_code() -> Subscription<Event> {
        subscription::channel("code-authenticator", 100, |mut output| async move {
            let mut state = State::Starting;

            loop {
                match &mut state {
                    State::Starting => {
                        let (sender, receiver) = mpsc::channel(100);

                        output
                            .send(Event::Ready(sender))
                            .await
                            .expect("failed to send input sender");

                        state = State::Ready(receiver);
                    }
                    State::Ready(receiver) => {
                        use iced::futures::StreamExt;

                        let input = receiver.select_next_some().await;

                        #[allow(irrefutable_let_patterns)]
                        if let Input::AuthenticateCode(code_response, client) = input {
                            let (countdown_sender, mut countdown_receiver) =
                                tokio::sync::mpsc::channel(code_response.expires_in as usize);

                            let handle = tokio::spawn(async move {
                                get_token_response(
                                    code_response.device_code,
                                    code_response.interval,
                                    code_response.expires_in,
                                    client.client_id,
                                    client.client_secret,
                                    countdown_sender,
                                )
                                .await
                            });

                            while (countdown_receiver.recv().await).is_some() {
                                output
                                    .send(Event::Progressing)
                                    .await
                                    .expect("failed to send the progress");
                            }

                            let token_response = handle
                                .await
                                .expect("failed to await progress handle")
                                .expect("failed to get token response");

                            output
                                .send(Event::WorkFinished(token_response))
                                .await
                                .expect("failed to send work completion");
                            state = State::Starting;
                        }
                    }
                }
            }
        })
    }
}

mod trakt_data_import {
    use std::mem::ManuallyDrop;

    use crate::core::api::trakt::import_shows::{self, ProgressData};
    use crate::core::api::trakt::trakt_data::TraktShow;
    use crate::core::database::Series;

    use iced::futures::channel::mpsc;
    use iced::futures::sink::SinkExt;
    use iced::subscription::{self, Subscription};

    #[derive(Debug, Clone)]
    pub enum Event {
        Ready(mpsc::Sender<Input>),
        #[allow(clippy::type_complexity)]
        WorkFinished(Result<(Vec<(u32, ManuallyDrop<Series>)>, Vec<TraktShow>), String>),
        /// (total_import_no, current_import_no)
        Progressing(usize, usize),
    }

    #[derive(Debug, Clone)]
    pub struct Input {
        client_id: String,
        slug: String,
    }

    impl Input {
        pub fn new(client_id: String, slug: String) -> Self {
            Self { client_id, slug }
        }
    }

    enum State {
        Starting,
        Ready(mpsc::Receiver<Input>),
    }

    pub fn import_trakt_data() -> Subscription<Event> {
        subscription::channel("trakt-data-import", 100, |mut output| async move {
            let mut state = State::Starting;

            loop {
                match &mut state {
                    State::Starting => {
                        let (sender, receiver) = mpsc::channel(100);

                        output
                            .send(Event::Ready(sender))
                            .await
                            .expect("failed to send input sender");

                        state = State::Ready(receiver);
                    }
                    State::Ready(receiver) => {
                        use iced::futures::StreamExt;

                        let input = receiver.select_next_some().await;

                        let (progress_sender, mut progress_receiver) =
                            tokio::sync::mpsc::channel(100);

                        let handle = tokio::spawn(async move {
                            import_shows::import(
                                &input.slug,
                                input.client_id.leak(),
                                progress_sender,
                            )
                            .await
                        });

                        let total_import = if let Some(ProgressData::TotalImport(total_import)) =
                            progress_receiver.recv().await
                        {
                            total_import
                        } else {
                            // This condition only happens when the import fails
                            // therefore returning zero shouldn't matter
                            tracing::error!("failed to obtain import total amount");
                            0
                        };

                        for (current_import, _) in std::iter::repeat(()).enumerate() {
                            if progress_receiver.recv().await.is_some() {
                                output
                                    .send(Event::Progressing(total_import, current_import + 1))
                                    .await
                                    .expect("failed to send the progress");
                            } else {
                                break;
                            }
                        }

                        let import = handle
                            .await
                            .expect("failed to await progress handle")
                            .map_err(|err| err.to_string());

                        output
                            .send(Event::WorkFinished(import))
                            .await
                            .expect("failed to send work completion");
                        state = State::Starting;
                    }
                }
            }
        })
    }
}
