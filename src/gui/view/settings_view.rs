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
            database_settings: Database,
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
            Message::Database(message) => self.database_settings.update(message),
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
        let content = column![text("Appearance").size(25)].padding(5).spacing(5);

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

mod database_widget {
    use iced::widget::{button, column, container, horizontal_space, row, text};
    use iced::{Element, Length, Renderer};

    use crate::gui::styles;

    #[derive(Debug, Clone)]
    pub enum Message {
        ImportDatabasePressed,
        ExportDatabasePressed,
    }

    #[derive(Default)]
    pub struct Database;

    impl Database {
        pub fn update(&mut self, message: Message) {
            match message {
                Message::ImportDatabasePressed => database_transfer::import(),
                Message::ExportDatabasePressed => database_transfer::export(),
            }
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let import_widget = column![
                text("Import Data").size(22),
                row![
                    text("Import your series tracking data into Series Troxide").size(15),
                    horizontal_space(Length::Fill),
                    button("Import").on_press(Message::ImportDatabasePressed)
                ]
            ];

            let export_widget = column![
                text("Export Data").size(22),
                row![
                    text("Export your series tracking data from Series Troxide").size(15),
                    horizontal_space(Length::Fill),
                    button("Export").on_press(Message::ExportDatabasePressed)
                ]
            ];

            let content = column![
                text("Series Troxide Data").size(25),
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

    mod database_transfer {
        use directories::UserDirs;
        use std::path;

        use crate::core::database::database_transfer;
        use rfd::FileDialog;

        pub fn export() {
            let chosen_path = FileDialog::new()
                .set_directory(get_home_directory())
                .save_file();

            if let Some(mut chosen_path) = chosen_path {
                let file_name = chosen_path.file_name().map(std::ffi::OsString::from);
                chosen_path.pop();
                database_transfer::write_database_to_path(&chosen_path, file_name.as_deref())
                    .unwrap();
            }
        }

        pub fn import() {
            let chosen_path = FileDialog::new()
                .set_directory(get_home_directory())
                .pick_file();

            if let Some(chosen_path) = chosen_path {
                database_transfer::read_database_from_path(path::Path::new(&chosen_path)).unwrap()
            }
        }

        pub fn get_home_directory() -> path::PathBuf {
            let user_dirs = UserDirs::new().unwrap();
            user_dirs.home_dir().to_path_buf()
        }
    }
}

mod caching_widget {
    use iced::widget::{button, column, container, horizontal_space, row, text, Button};
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
                text("Series Troxide Cache").size(25),
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
                    text(status_text),
                    button,
                ]
                .spacing(3)
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
                    text(status_text),
                    button,
                ]
                .spacing(3)
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
                    text(status_text),
                    button,
                ]
                .spacing(3)
            ]
            .into()
        }
    }

    /// Generates the status text and the button for each type of cleaning
    fn status_and_button_gen(
        cleaning_status: &CleaningStatus,
        button_message: Message,
    ) -> (&str, Button<Message, Renderer>) {
        let mut button = button("clean");
        let status_text = match cleaning_status {
            CleaningStatus::Idle => {
                button = button.on_press(button_message);
                ""
            }
            CleaningStatus::Running => "Running",
            CleaningStatus::Done(error_message) => {
                button = button.on_press(button_message);
                if let Some(error_message) = error_message {
                    error_message.as_str()
                } else {
                    "Done"
                }
            }
        };

        (status_text, button)
    }
}
