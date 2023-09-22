use iced::widget::{
    button, column, container, horizontal_space, progress_bar, row, text, vertical_space, Space,
    Text,
};
use iced::{Command, Element, Length, Renderer};

use crate::core::database::get_ids_from_keys_values_vec;
use crate::core::database::{self, DB};

use crate::gui::styles;

#[derive(Debug, Clone)]
pub enum Message {
    ImportDatabasePressed,
    ExportDatabasePressed,
    ImportTimeoutComplete,
    ExportTimeoutComplete,
    ImportCachingEvent(full_caching::Event),
    TraktIntegration(trakt_integration::Message),
}

pub struct Database {
    import_status: Option<anyhow::Result<()>>,
    export_status: Option<anyhow::Result<()>>,
    import_progress: (usize, usize),
    importing: bool,
    keys_values_vec: Option<database::KeysValuesVec>,
    sender: Option<iced::futures::channel::mpsc::Sender<full_caching::Input>>,
    trakt_widget: trakt_integration::TraktIntegration,
}

impl Database {
    pub fn new() -> Self {
        Self {
            import_status: None,
            export_status: None,
            import_progress: (0, 0),
            importing: false,
            keys_values_vec: None,
            sender: None,
            trakt_widget: trakt_integration::TraktIntegration::new(),
        }
    }
    pub fn subscription(&self) -> iced::Subscription<Message> {
        full_caching::import_data_cacher().map(Message::ImportCachingEvent)
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ImportDatabasePressed => match database_transfer::import_keys_values_vec() {
                Ok(keys_values_vec) => {
                    if let Some(keys_values_vec) = keys_values_vec {
                        self.keys_values_vec = Some(keys_values_vec.clone());
                        let ids: Vec<u32> = get_ids_from_keys_values_vec(keys_values_vec)
                            .into_iter()
                            .map(|series_id| {
                                series_id
                                    .parse()
                                    .expect("series id should be parsable to u32")
                            })
                            .collect();
                        self.import_progress.1 = ids.len();
                        self.importing = true;
                        self.sender
                            .as_mut()
                            .expect("there should be a work sender at this point")
                            .try_send(full_caching::Input::CacheSeries(ids))
                            .expect("full caching receiver disconnected");
                    }
                    Command::none()
                }
                Err(err) => {
                    self.import_status = Some(Err(anyhow::anyhow!(err)));
                    Command::perform(super::status_timeout(), |_| Message::ImportTimeoutComplete)
                }
            },
            Message::ExportDatabasePressed => {
                self.export_status = Some(database_transfer::export());
                Command::perform(super::status_timeout(), |_| Message::ExportTimeoutComplete)
            }
            Message::ImportTimeoutComplete => {
                self.import_status = None;
                Command::none()
            }
            Message::ExportTimeoutComplete => {
                self.export_status = None;
                Command::none()
            }
            Message::ImportCachingEvent(event) => {
                match event {
                    full_caching::Event::Ready(work_sender) => {
                        self.sender = Some(work_sender);
                    }
                    full_caching::Event::WorkFinished => {
                        self.import_progress = (0, 0);
                        self.importing = false;
                        DB.import_keys_value_vec(
                            self.keys_values_vec
                                .take()
                                .expect("there should be keys_values_vec at this point"),
                        )
                        .expect("failed to import keys_values_vec");

                        self.import_status = Some(Ok(()));
                        return Command::perform(super::status_timeout(), |_| {
                            Message::ImportTimeoutComplete
                        });
                    }
                    full_caching::Event::Progressing => {
                        self.import_progress.0 += 1;
                    }
                }
                Command::none()
            }
            Message::TraktIntegration(message) => self
                .trakt_widget
                .update(message)
                .map(Message::TraktIntegration),
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let import_progress: Element<'_, Message, Renderer> = if self.importing {
            row![
                text("caching the import").size(11),
                progress_bar(
                    0.0..=self.import_progress.1 as f32,
                    self.import_progress.0 as f32,
                )
                .height(13),
                text(format!(
                    "{} / {}",
                    self.import_progress.0, self.import_progress.1
                ))
                .size(11)
            ]
            .spacing(10)
            .into()
        } else {
            Space::new(0, 0).into()
        };

        let import_widget = column![
            text("Import Data").size(18),
            row![
                text("Import your series tracking data into Series Troxide").size(11),
                horizontal_space(Length::Fill),
                get_status_text(self.import_status.as_ref()),
                {
                    let mut button = button("Import");
                    if !self.importing {
                        button = button.on_press(Message::ImportDatabasePressed);
                    }
                    button
                },
            ]
            .spacing(5),
            vertical_space(5),
            import_progress,
        ];

        let export_widget = column![
            text("Export Data").size(18),
            row![
                text("Export your series tracking data from Series Troxide").size(11),
                horizontal_space(Length::Fill),
                get_status_text(self.export_status.as_ref()),
                button("Export").on_press(Message::ExportDatabasePressed)
            ]
            .spacing(5)
        ];

        let series_troxide_data = column![
            text("Series Troxide Data").size(18),
            import_widget,
            export_widget,
        ]
        .spacing(5);

        let trakt_data = column![
            text("Trakt Data").size(18),
            self.trakt_widget.view().map(Message::TraktIntegration)
        ]
        .spacing(5);

        let content = column![
            text("Series Tracking Data")
                .size(21)
                .style(styles::text_styles::purple_text_theme()),
            series_troxide_data,
            trakt_data
        ]
        .padding(5);

        container(content)
            .style(styles::container_styles::first_class_container_rounded_theme())
            .width(1000)
            .into()
    }
}

fn get_status_text(status: Option<&anyhow::Result<()>>) -> Text {
    if let Some(res) = status {
        if let Err(err) = res {
            text(err.to_string()).style(styles::text_styles::red_text_theme())
        } else {
            text("Done!").style(styles::text_styles::green_text_theme())
        }
    } else {
        text("")
    }
}

mod full_caching {
    use crate::core::caching::series_info_and_episode_list::SeriesInfoAndEpisodeList;

    use iced::futures::channel::mpsc;
    use iced::futures::sink::SinkExt;
    use iced::subscription::{self, Subscription};

    #[derive(Debug, Clone)]
    pub enum Event {
        Ready(mpsc::Sender<Input>),
        WorkFinished,
        Progressing,
    }

    #[derive(Debug, Clone)]
    pub enum Input {
        CacheSeries(Vec<u32>),
    }

    enum State {
        Starting,
        Ready(mpsc::Receiver<Input>),
    }

    pub fn import_data_cacher() -> Subscription<Event> {
        subscription::channel("settings-data-importer", 100, |mut output| async move {
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
                        if let Input::CacheSeries(ids) = input {
                            let (series_info_and_episode_list, mut progress_receiver) =
                                SeriesInfoAndEpisodeList::new(ids);

                            let handle = tokio::spawn(async move {
                                let series_info_and_episode_list = series_info_and_episode_list;
                                series_info_and_episode_list.run_full_caching(true).await
                            });

                            while let Some(result) = progress_receiver.recv().await {
                                if let Err(err) = result {
                                    tracing::error!("progress error: {}", err);
                                }

                                output
                                    .send(Event::Progressing)
                                    .await
                                    .expect("failed to send the progress");
                            }
                            handle
                                .await
                                .expect("failed to await progress handle")
                                .unwrap();
                            output
                                .send(Event::WorkFinished)
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

mod database_transfer {
    use directories::UserDirs;
    use std::path;

    use crate::core::database::database_transfer;
    use crate::core::database::KeysValuesVec;
    use rfd::FileDialog;

    pub fn export() -> anyhow::Result<()> {
        let chosen_path = FileDialog::new()
            .set_directory(get_home_directory()?)
            .save_file();

        if let Some(mut chosen_path) = chosen_path {
            let file_name = chosen_path.file_name().map(std::ffi::OsString::from);
            chosen_path.pop();
            database_transfer::write_database_to_path(&chosen_path, file_name.as_deref())?;
        }

        Ok(())
    }

    // TODO: Uncomment when needed
    // pub fn import() -> anyhow::Result<()> {
    //     let chosen_path = FileDialog::new()
    //         .set_directory(get_home_directory()?)
    //         .pick_file();

    //     if let Some(chosen_path) = chosen_path {
    //         database_transfer::read_database_from_path(path::Path::new(&chosen_path))?;
    //     }

    //     Ok(())
    // }

    pub fn import_keys_values_vec() -> anyhow::Result<Option<KeysValuesVec>> {
        let chosen_path = FileDialog::new()
            .set_directory(get_home_directory()?)
            .pick_file();

        if let Some(chosen_path) = chosen_path {
            let data = database_transfer::read_database_from_path_as_keys_value_vec(
                path::Path::new(&chosen_path),
            )?;
            return Ok(Some(data));
        }

        Ok(None)
    }

    pub fn get_home_directory() -> anyhow::Result<path::PathBuf> {
        let user_dirs = UserDirs::new().ok_or(anyhow::anyhow!("could not get user directory"))?;
        Ok(user_dirs.home_dir().to_path_buf())
    }
}

mod trakt_integration {
    use iced::widget::{button, column, horizontal_space, row, svg, text, text_input};
    use iced::{Alignment, Command, Element, Length, Renderer};
    use iced_aw::Spinner;

    use crate::core::api::trakt::authenication::{self, CodeResponse, TokenResponse};
    use crate::core::api::trakt::user_credentials::{self, Client, Credentials, CredentialsError};
    use crate::gui::assets::{get_static_cow_from_asset, icons::TRAKT_ICON_RED};

    #[derive(Debug, Clone)]
    pub enum Message {
        StartPage(StartPageMessage),
        ClientPage(ClientPageMessage),
        ProgramAuthenticationPage(ProgramAuthenticationPageMessage),
        LoadCredentials,
        CredentialsLoaded(Credentials),
        Cancel,
    }

    pub struct TraktIntegration {
        setup_page: Option<SetupPage>,
    }

    impl TraktIntegration {
        pub fn new() -> Self {
            Self { setup_page: None }
        }

        pub fn update(&mut self, message: Message) -> Command<Message> {
            let mut next_page = None;
            let command = match message {
                Message::LoadCredentials => Command::perform(Credentials::new(), |res| {
                    Message::CredentialsLoaded(res.expect("failed to load the client"))
                }),
                Message::CredentialsLoaded(credentials) => {
                    self.setup_page = Some(SetupPage::StartPage(StartPage::new(credentials)));
                    Command::none()
                }
                Message::Cancel => {
                    self.setup_page = None;
                    Command::none()
                }
                Message::StartPage(message) => {
                    if let Some(SetupPage::StartPage(start_page)) = self.setup_page.as_mut() {
                        start_page
                            .update(message, &mut next_page)
                            .map(Message::StartPage)
                    } else {
                        Command::none()
                    }
                }
                Message::ClientPage(message) => {
                    if let Some(SetupPage::ClientPage(client_page)) = self.setup_page.as_mut() {
                        client_page
                            .update(message, &mut next_page)
                            .map(Message::ClientPage)
                    } else {
                        Command::none()
                    }
                }
                Message::ProgramAuthenticationPage(message) => {
                    if let Some(SetupPage::ProgramAuthenticationPage(program_authenication_page)) =
                        self.setup_page.as_mut()
                    {
                        program_authenication_page
                            .update(message, &mut next_page)
                            .map(Message::ProgramAuthenticationPage)
                    } else {
                        Command::none()
                    }
                }
            };

            if let Some(next_page) = next_page {
                self.setup_page = Some(next_page)
            };

            command
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            if let Some(setup_page) = self.setup_page.as_ref() {
                let setup_page = match setup_page {
                    SetupPage::StartPage(start_page) => start_page.view().map(Message::StartPage),
                    SetupPage::ClientPage(client_page) => {
                        client_page.view().map(Message::ClientPage)
                    }
                    SetupPage::ProgramAuthenticationPage(program_authentication_page) => {
                        program_authentication_page
                            .view()
                            .map(Message::ProgramAuthenticationPage)
                    }
                    SetupPage::Confirmation(_) => todo!(),
                };
                column![setup_page, button("cancel").on_press(Message::Cancel),]
                    .spacing(5)
                    .width(Length::Fill)
                    .align_items(Alignment::Center)
                    .into()
            } else {
                row![
                    text("Import series data from your Trakt account"),
                    horizontal_space(Length::Fill),
                    button("Configure Trakt integration").on_press(Message::LoadCredentials),
                ]
                .into()
            }
        }
    }

    enum SetupPage {
        StartPage(StartPage),
        ClientPage(ClientPage),
        ProgramAuthenticationPage(ProgramAuthenticationPage),
        Confirmation(TokenResponse),
    }

    #[derive(Debug, Clone)]
    pub enum StartPageMessage {
        ConnectAccount,
    }

    struct StartPage {
        credentials: Credentials,
    }

    impl StartPage {
        pub fn new(credentials: Credentials) -> Self {
            Self { credentials }
        }

        pub fn update(
            &mut self,
            message: StartPageMessage,
            next_page: &mut Option<SetupPage>,
        ) -> Command<StartPageMessage> {
            match message {
                StartPageMessage::ConnectAccount => {
                    let client_page = ClientPage::new();
                    *next_page = Some(SetupPage::ClientPage(client_page));
                    Command::none()
                }
            }
        }

        pub fn view(&self) -> Element<'_, StartPageMessage, Renderer> {
            let content = if let Some((user, token)) = self.credentials.payload() {
                column![
                    text("Current Authentication Status"),
                    row![text("Username: "), text(&user.username)],
                    row![
                        text("Token Status: "),
                        text(match token.get_access_token() {
                            Ok(_) => "Valid",
                            Err(_) => "Expired",
                        })
                    ],
                    button("Reconnect Trakt Account").on_press(StartPageMessage::ConnectAccount),
                ]
            } else {
                column![
                    text("Trakt Account has not been connected yet"),
                    button("Connect Trakt Account").on_press(StartPageMessage::ConnectAccount),
                ]
                .spacing(2)
                .align_items(Alignment::Center)
            };

            let trakt_icon_handle =
                svg::Handle::from_memory(get_static_cow_from_asset(TRAKT_ICON_RED));
            let trakt_icon = svg(trakt_icon_handle).height(50);

            column![trakt_icon, content]
                .align_items(Alignment::Center)
                .spacing(5)
                .into()
        }
    }

    #[derive(Debug, Clone)]
    pub enum ClientPageMessage {
        ClientIdChanged(String),
        ClientSecretChanged(String),
        CodeReceived(CodeResponse),
        Submit,
    }

    struct ClientPage {
        client: Result<Client, CredentialsError>,
        client_id: String,
        client_secret: String,
        can_submit: bool,
        code_loading: bool,
    }

    impl ClientPage {
        fn new() -> Self {
            let client = user_credentials::load_client();
            let can_submit = client.is_ok();
            Self {
                client,
                client_id: String::new(),
                client_secret: String::new(),
                can_submit,
                code_loading: false,
            }
        }

        fn update(
            &mut self,
            message: ClientPageMessage,
            next_page: &mut Option<SetupPage>,
        ) -> Command<ClientPageMessage> {
            let command = match message {
                ClientPageMessage::ClientIdChanged(text) => {
                    self.client_id = text;
                    Command::none()
                }
                ClientPageMessage::ClientSecretChanged(text) => {
                    self.client_secret = text;
                    Command::none()
                }
                ClientPageMessage::Submit => {
                    self.code_loading = true;
                    match &self.client {
                        Ok(client) => Command::perform(
                            authenication::get_device_code_response(client.client_id.clone()),
                            ClientPageMessage::CodeReceived,
                        ),
                        Err(_) => Command::perform(
                            authenication::get_device_code_response(self.client_id.clone()),
                            ClientPageMessage::CodeReceived,
                        ),
                    }
                }
                ClientPageMessage::CodeReceived(code_response) => {
                    *next_page = Some(SetupPage::ProgramAuthenticationPage(
                        ProgramAuthenticationPage::new(code_response),
                    ));
                    Command::none()
                }
            };

            self.can_submit = !self.client_id.is_empty() && !self.client_secret.is_empty();

            command
        }

        fn view(&self) -> Element<'_, ClientPageMessage, Renderer> {
            if self.code_loading {
                Spinner::new().into()
            } else {
                let mut submit_button = button("Submit");
                if self.can_submit {
                    submit_button = submit_button.on_press(ClientPageMessage::Submit)
                };

                let content = match &self.client {
                    Ok(client) => column![
                        text("Current client information"),
                        row![text("Client ID: "), text(client.client_id.as_str())],
                        row![text("Client Secret: "), text(client.client_secret.as_str())],
                    ],
                    Err(_) => column![
                        text("Enter your Trakt client information"),
                        text_input("Client ID", &self.client_id)
                            .on_input(ClientPageMessage::ClientIdChanged),
                        text_input("Client Secret", &self.client_secret)
                            .on_input(ClientPageMessage::ClientSecretChanged),
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
    }

    #[derive(Debug, Clone)]
    pub enum ProgramAuthenticationPageMessage {}

    struct ProgramAuthenticationPage {
        code: CodeResponse,
    }
    impl ProgramAuthenticationPage {
        fn new(code_response: CodeResponse) -> Self {
            Self {
                code: code_response,
            }
        }

        fn update(
            &mut self,
            message: ProgramAuthenticationPageMessage,
            next_page: &mut Option<SetupPage>,
        ) -> Command<ProgramAuthenticationPageMessage> {
            Command::none()
        }

        fn view(&self) -> Element<'_, ProgramAuthenticationPageMessage, Renderer> {
            text(format!("{:#?}", self.code)).into()
        }
    }
}
