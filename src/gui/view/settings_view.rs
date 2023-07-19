use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, text, vertical_space,
};
use iced::{Alignment, Command, Element, Length, Renderer};

use crate::core::settings_config::{save_config, Config, Theme, ALL_THEMES};
use crate::gui::assets::icons::GEAR_WIDE_CONNECTED;
use crate::gui::{styles, troxide_widget, Message as GuiMessage, Tab};
use caching_widget::{Caching, Message as CachingMessage};
use database_widget::{Database, Message as DatabaseMessage};

#[derive(Debug, Clone)]
pub enum Message {
    ThemeSelected(Theme),
    SaveSettings,
    Caching(CachingMessage),
    Database(DatabaseMessage),
}

#[derive(Default)]
pub struct SettingsTab {
    settings_config: Config,
    unsaved_config: Option<Config>,
    caching_settings: Caching,
    database_settings: Database,
}

impl SettingsTab {
    pub fn new(settings_config: Config) -> Self {
        Self {
            settings_config,
            unsaved_config: None,
            caching_settings: Caching::default(),
            database_settings: Database::default(),
        }
    }

    pub fn get_config_settings(&self) -> &Config {
        if let Some(config) = &self.unsaved_config {
            config
        } else {
            &self.settings_config
        }
    }
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ThemeSelected(theme) => {
                if let Some(config) = &mut self.unsaved_config {
                    config.theme = theme
                } else {
                    let mut unsaved_config = self.settings_config.clone();
                    unsaved_config.theme = theme;
                    self.unsaved_config = Some(unsaved_config);
                }
            }
            Message::SaveSettings => {
                if let Some(config) = self.unsaved_config.take() {
                    self.settings_config = config;
                    save_config(&self.settings_config);
                }
            }
            Message::Caching(message) => {
                return self.caching_settings.update(message).map(Message::Caching)
            }
            Message::Database(message) => {
                return self
                    .database_settings
                    .update(message)
                    .map(Message::Database)
            }
        }
        Command::none()
    }
    pub fn view(&self) -> Element<Message, Renderer> {
        let settings_body = column![
            self.appearance_settings_view(),
            self.database_settings.view().map(Message::Database),
            self.caching_settings.view().map(Message::Caching),
        ]
        .spacing(5)
        .padding(5);

        let mut save_settings_button = button("Save Settings");

        if let Some(unsaved_settings) = &self.unsaved_config {
            if *unsaved_settings != self.settings_config {
                save_settings_button = save_settings_button.on_press(Message::SaveSettings);
            }
        };

        let save_button_bar = row!(horizontal_space(Length::Fill), save_settings_button).padding(5);

        column![settings_body, vertical_space(Length::Fill), save_button_bar]
            .align_items(Alignment::Center)
            .spacing(5)
            .padding(10)
            .into()
    }

    fn appearance_settings_view(&self) -> Element<Message, Renderer> {
        let content = column![text("Appearance")
            .size(25)
            .style(styles::text_styles::purple_text_theme())]
        .padding(5)
        .spacing(5);

        let theme_text = text("Theme");
        let theme_picklist = pick_list(
            &ALL_THEMES[..],
            Some(if let Some(config) = &self.unsaved_config {
                config.theme.clone()
            } else {
                self.settings_config.theme.clone()
            }),
            Message::ThemeSelected,
        );

        let content = content.push(
            row!(theme_text, horizontal_space(20), theme_picklist)
                .padding(5)
                .spacing(5),
        );

        container(content)
            .style(styles::container_styles::first_class_container_theme())
            .width(1000)
            .into()
    }
}

impl Tab for SettingsTab {
    type Message = GuiMessage;

    fn title(&self) -> String {
        "Settings".to_owned()
    }

    fn tab_label(&self) -> troxide_widget::tabs::TabLabel {
        troxide_widget::tabs::TabLabel::new(self.title(), GEAR_WIDE_CONNECTED)
    }

    fn content(&self) -> Element<'_, Self::Message> {
        self.view().map(GuiMessage::Settings)
    }
}

/// A function that sleeps for 3 seconds designed to provide timeout
/// for status texts in widgets like the database and caching widget.
async fn status_timeout() {
    tokio::time::sleep(std::time::Duration::from_secs(3)).await
}

mod database_widget {
    use iced::widget::{button, column, container, horizontal_space, row, text, Text};
    use iced::{Command, Element, Length, Renderer};

    use crate::gui::styles;

    #[derive(Debug, Clone)]
    pub enum Message {
        ImportDatabasePressed,
        ExportDatabasePressed,
        ImportTimeoutComplete,
        ExportTimeoutComplete,
    }

    #[derive(Default)]
    pub struct Database {
        import_status: Option<anyhow::Result<()>>,
        export_status: Option<anyhow::Result<()>>,
    }

    impl Database {
        pub fn update(&mut self, message: Message) -> Command<Message> {
            match message {
                Message::ImportDatabasePressed => {
                    self.import_status = Some(database_transfer::import());

                    Command::perform(super::status_timeout(), |_| Message::ImportTimeoutComplete)
                }
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
            }
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let import_widget = column![
                text("Import Data").size(22),
                row![
                    text("Import your series tracking data into Series Troxide").size(15),
                    horizontal_space(Length::Fill),
                    get_status_text(self.import_status.as_ref()),
                    button("Import").on_press(Message::ImportDatabasePressed)
                ]
                .spacing(5)
            ];

            let export_widget = column![
                text("Export Data").size(22),
                row![
                    text("Export your series tracking data from Series Troxide").size(15),
                    horizontal_space(Length::Fill),
                    get_status_text(self.export_status.as_ref()),
                    button("Export").on_press(Message::ExportDatabasePressed)
                ]
                .spacing(5)
            ];

            let content = column![
                text("Series Troxide Data")
                    .size(25)
                    .style(styles::text_styles::purple_text_theme()),
                import_widget,
                export_widget,
            ]
            .padding(5)
            .spacing(5);

            container(content)
                .style(styles::container_styles::first_class_container_theme())
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

    mod database_transfer {
        use directories::UserDirs;
        use std::path;

        use crate::core::database::database_transfer;
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

        pub fn import() -> anyhow::Result<()> {
            let chosen_path = FileDialog::new()
                .set_directory(get_home_directory()?)
                .pick_file();

            if let Some(chosen_path) = chosen_path {
                database_transfer::read_database_from_path(path::Path::new(&chosen_path))?;
            }

            Ok(())
        }

        pub fn get_home_directory() -> anyhow::Result<path::PathBuf> {
            let user_dirs =
                UserDirs::new().ok_or(anyhow::anyhow!("could not get user directory"))?;
            Ok(user_dirs.home_dir().to_path_buf())
        }
    }
}

mod caching_widget {
    use iced::widget::{button, column, container, horizontal_space, row, text, Button, Text};
    use iced::{Command, Element, Length, Renderer};

    use crate::core::caching::cache_cleaning;
    use crate::gui::styles;

    #[allow(clippy::enum_variant_names)] // Removing the word clean makes the message not make sense
    #[derive(Clone, Debug)]
    pub enum Message {
        CleanEndedCache,
        CleanWaitingReleaseCache,
        CleanAiredCache,
        CleanEndedCacheComplete(Option<String>),
        CleanWaitingReleaseCacheComplete(Option<String>),
        CleanAiredCacheComplete(Option<String>),
    }

    #[derive(Default)]
    enum CleaningStatus {
        #[default]
        Idle,
        Running,
        Done(Option<String>),
    }

    #[derive(Default)]
    pub struct Caching {
        ended_series_cache_cleaning: CleaningStatus,
        aired_series_cache_cleaning: CleaningStatus,
        waiting_release_series_cache_cleaning: CleaningStatus,
    }

    impl Caching {
        pub fn update(&mut self, message: Message) -> Command<Message> {
            match message {
                Message::CleanEndedCache => {
                    self.ended_series_cache_cleaning = CleaningStatus::Running;
                    return Command::perform(cache_cleaning::clean_ended_series_cache(), |res| {
                        Message::CleanEndedCacheComplete(res.err().map(|err| err.to_string()))
                    });
                }
                Message::CleanWaitingReleaseCache => {
                    self.waiting_release_series_cache_cleaning = CleaningStatus::Running;
                    return Command::perform(
                        cache_cleaning::clean_running_cache(
                            cache_cleaning::RunningStatus::WaitingRelease,
                        ),
                        |res| {
                            Message::CleanWaitingReleaseCacheComplete(
                                res.err().map(|err| err.to_string()),
                            )
                        },
                    );
                }
                Message::CleanAiredCache => {
                    self.aired_series_cache_cleaning = CleaningStatus::Running;
                    return Command::perform(
                        cache_cleaning::clean_running_cache(cache_cleaning::RunningStatus::Aired),
                        |res| {
                            Message::CleanAiredCacheComplete(res.err().map(|err| err.to_string()))
                        },
                    );
                }
                Message::CleanEndedCacheComplete(error_text) => {
                    self.ended_series_cache_cleaning = CleaningStatus::Done(error_text)
                }
                Message::CleanWaitingReleaseCacheComplete(error_text) => {
                    self.waiting_release_series_cache_cleaning = CleaningStatus::Done(error_text)
                }
                Message::CleanAiredCacheComplete(error_text) => {
                    self.aired_series_cache_cleaning = CleaningStatus::Done(error_text)
                }
            }
            Command::none()
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let content = column![
                text("Series Troxide Cache")
                    .size(25)
                    .style(styles::text_styles::purple_text_theme()),
                self.clean_aired_cache_widget(),
                self.clean_waiting_release_cache_widget(),
                self.clean_ended_cache_widget(),
            ]
            .padding(5)
            .spacing(5);

            container(content)
                .style(styles::container_styles::first_class_container_theme())
                .width(1000)
                .into()
        }

        fn clean_ended_cache_widget(&self) -> Element<'_, Message, Renderer> {
            let (status_text, button) =
                status_and_button_gen(&self.ended_series_cache_cleaning, Message::CleanEndedCache);
            column![
                text("Ended Cache Cleaning").size(22),
                row![
                    text("clean cache for the series that have ended").size(15),
                    horizontal_space(Length::Fill),
                    status_text,
                    button,
                ]
                .spacing(5)
            ]
            .into()
        }

        fn clean_aired_cache_widget(&self) -> Element<'_, Message, Renderer> {
            let (status_text, button) =
                status_and_button_gen(&self.aired_series_cache_cleaning, Message::CleanAiredCache);

            column![
                text("Aired Cache Cleaning").size(22),
                row![
                    text("clean cache for the series that are currently being aired").size(15),
                    horizontal_space(Length::Fill),
                    status_text,
                    button,
                ]
                .spacing(5)
            ]
            .into()
        }

        fn clean_waiting_release_cache_widget(&self) -> Element<'_, Message, Renderer> {
            let (status_text, button) = status_and_button_gen(
                &self.waiting_release_series_cache_cleaning,
                Message::CleanWaitingReleaseCache,
            );

            column![
                text("Waiting Release Cache Cleaning").size(22),
                row![
                    text("clean cache for the series waiting for their release date").size(15),
                    horizontal_space(Length::Fill),
                    status_text,
                    button,
                ]
                .spacing(5)
            ]
            .into()
        }
    }

    /// Generates the status text and the button for each type of cleaning
    fn status_and_button_gen(
        cleaning_status: &CleaningStatus,
        button_message: Message,
    ) -> (Text, Button<Message, Renderer>) {
        let mut button = button("clean");
        let status_text = match cleaning_status {
            CleaningStatus::Idle => {
                button = button.on_press(button_message);
                text("")
            }
            CleaningStatus::Running => text("Running"),
            CleaningStatus::Done(error_message) => {
                button = button.on_press(button_message);
                if let Some(error_message) = error_message {
                    text(error_message.as_str()).style(styles::text_styles::red_text_theme())
                } else {
                    text("Done!").style(styles::text_styles::green_text_theme())
                }
            }
        };

        (status_text, button)
    }
}
