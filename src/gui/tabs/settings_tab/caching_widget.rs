use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, text, vertical_space,
    Button, Text,
};
use iced::{Command, Element, Length, Renderer};

use cache_cleaning_frequency_widget::{
    CacheCleaningFrequency, Message as CacheCleaningFrequencyMessage,
};

// use crate::core::caching::cache_cleaning;
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
    // timeouts for removing the status text
    CleanEndedCacheTimeoutComplete,
    CleanWaitingReleaseCacheTimeoutComplete,
    CleanAiredCacheTimeoutComplete,
    CacheCleaningFrequency(CacheCleaningFrequencyMessage),
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
    caching_frequency_settings_widget: CacheCleaningFrequency,
}

impl Caching {
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CleanEndedCache => {
                self.ended_series_cache_cleaning = CleaningStatus::Running;
                return Command::perform(cache_cleaning::clean_ended_cache(), |res| {
                    Message::CleanEndedCacheComplete(res.err().map(|err| err.to_string()))
                });
            }
            Message::CleanWaitingReleaseCache => {
                self.waiting_release_series_cache_cleaning = CleaningStatus::Running;
                return Command::perform(cache_cleaning::cleaning_waiting_release_cache(), |res| {
                    Message::CleanWaitingReleaseCacheComplete(res.err().map(|err| err.to_string()))
                });
            }
            Message::CleanAiredCache => {
                self.aired_series_cache_cleaning = CleaningStatus::Running;
                return Command::perform(cache_cleaning::clean_aired_cache(), |res| {
                    Message::CleanAiredCacheComplete(res.err().map(|err| err.to_string()))
                });
            }
            Message::CleanEndedCacheComplete(error_text) => {
                self.ended_series_cache_cleaning = CleaningStatus::Done(error_text);
                return Command::perform(super::status_timeout(), |_| {
                    Message::CleanEndedCacheTimeoutComplete
                });
            }
            Message::CleanWaitingReleaseCacheComplete(error_text) => {
                self.waiting_release_series_cache_cleaning = CleaningStatus::Done(error_text);
                return Command::perform(super::status_timeout(), |_| {
                    Message::CleanWaitingReleaseCacheTimeoutComplete
                });
            }
            Message::CleanAiredCacheComplete(error_text) => {
                self.aired_series_cache_cleaning = CleaningStatus::Done(error_text);
                return Command::perform(super::status_timeout(), |_| {
                    Message::CleanAiredCacheTimeoutComplete
                });
            }
            Message::CleanEndedCacheTimeoutComplete => {
                self.ended_series_cache_cleaning = CleaningStatus::Idle
            }
            Message::CleanWaitingReleaseCacheTimeoutComplete => {
                self.waiting_release_series_cache_cleaning = CleaningStatus::Idle
            }
            Message::CleanAiredCacheTimeoutComplete => {
                self.aired_series_cache_cleaning = CleaningStatus::Idle
            }
            Message::CacheCleaningFrequency(message) => {
                self.caching_frequency_settings_widget.update(message)
            }
        }
        Command::none()
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let manual_cleaning_heading_text = text("Manual Cache Cleaning").size(18);
        let manual_cleaning_explaination_text = text("Despite the fact that automated cache cleaning \
            can be configured, manual cleaning is still an option if you choose. All cache types can be manually cleaned up. \
            They will each execute a clean as soon as their separate clean buttons are pressed.").size(11);

        let content = column![
            text("Series Troxide Cache")
                .size(21)
                .style(styles::text_styles::purple_text_theme()),
            self.caching_frequency_settings_widget
                .view()
                .map(Message::CacheCleaningFrequency),
            vertical_space(10),
            horizontal_rule(1),
            vertical_space(10),
            manual_cleaning_heading_text,
            manual_cleaning_explaination_text,
            vertical_space(5),
            self.clean_aired_cache_widget(),
            self.clean_waiting_release_cache_widget(),
            self.clean_ended_cache_widget(),
        ]
        .padding(5)
        .spacing(5);

        container(content)
            .style(styles::container_styles::first_class_container_rounded_theme())
            .width(1000)
            .into()
    }

    fn clean_ended_cache_widget(&self) -> Element<'_, Message, Renderer> {
        let (status_text, button) =
            status_and_button_gen(&self.ended_series_cache_cleaning, Message::CleanEndedCache);
        row![
            text("Clean cache for the series that have ended"),
            horizontal_space(Length::Fill),
            status_text,
            button,
        ]
        .spacing(5)
        .into()
    }

    fn clean_aired_cache_widget(&self) -> Element<'_, Message, Renderer> {
        let (status_text, button) =
            status_and_button_gen(&self.aired_series_cache_cleaning, Message::CleanAiredCache);

        row![
            text("Clean cache for the series that are currently being aired"),
            horizontal_space(Length::Fill),
            status_text,
            button,
        ]
        .spacing(5)
        .into()
    }

    fn clean_waiting_release_cache_widget(&self) -> Element<'_, Message, Renderer> {
        let (status_text, button) = status_and_button_gen(
            &self.waiting_release_series_cache_cleaning,
            Message::CleanWaitingReleaseCache,
        );

        row![
            text("Clean cache for the series waiting for release date"),
            horizontal_space(Length::Fill),
            status_text,
            button,
        ]
        .spacing(5)
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

mod cache_cleaning {
    use crate::core::caching::cache_cleaning::{self, CleanType, RunningStatus};

    pub async fn clean_ended_cache() -> anyhow::Result<()> {
        cache_cleaning::clean_cache(CleanType::Ended, None).await
    }

    pub async fn cleaning_waiting_release_cache() -> anyhow::Result<()> {
        cache_cleaning::clean_cache(CleanType::Running(RunningStatus::WaitingRelease), None).await
    }

    pub async fn clean_aired_cache() -> anyhow::Result<()> {
        cache_cleaning::clean_cache(CleanType::Running(RunningStatus::Aired), None).await
    }
}

mod cache_cleaning_frequency_widget {
    use iced::widget::{column, container, slider, text, vertical_space};
    use iced::{Element, Length, Renderer};

    use crate::core::settings_config::SETTINGS;

    #[derive(Clone, Debug)]
    pub enum Message {
        Aired(u32),
        Ended(u32),
        WaitingRelease(u32),
    }

    #[derive(Default)]
    pub struct CacheCleaningFrequency;

    impl CacheCleaningFrequency {
        pub fn update(&mut self, message: Message) {
            let mut settings = SETTINGS.write().unwrap();
            let cache_settigns = &mut settings.change_settings().cache;

            match message {
                Message::Aired(new_value) => cache_settigns.aired_cache_clean_frequency = new_value,
                Message::Ended(new_value) => cache_settigns.ended_cache_clean_frequency = new_value,
                Message::WaitingRelease(new_value) => {
                    cache_settigns.waiting_release_cache_clean_frequency = new_value
                }
            }
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let settings = SETTINGS.read().unwrap();
            let cache_settings = &settings.get_current_settings().cache;

            let aired_clean_frequency = container(
                slider(
                    1..=7,
                    cache_settings.aired_cache_clean_frequency,
                    Message::Aired,
                )
                .width(800),
            )
            .width(Length::Fill)
            .center_x();

            let ended_clean_frequency = container(
                slider(
                    1..=30,
                    cache_settings.ended_cache_clean_frequency,
                    Message::Ended,
                )
                .width(800),
            )
            .width(Length::Fill)
            .center_x();

            let waiting_release_frequency = container(
                slider(
                    1..=14,
                    cache_settings.waiting_release_cache_clean_frequency,
                    Message::WaitingRelease,
                )
                .width(800),
            )
            .width(Length::Fill)
            .center_x();

            let aired_clean_frequency = column![
                text(format!(
                    "Cache cleaning frequency for aired series: {} days",
                    cache_settings.aired_cache_clean_frequency
                )),
                aired_clean_frequency
            ]
            .spacing(5);

            let waiting_release_frequency = column![
                text(format!(
                    "Cache cleaning frequency for series waiting for release date: {} days",
                    cache_settings.waiting_release_cache_clean_frequency
                )),
                waiting_release_frequency
            ]
            .spacing(5);

            let ended_clean_frequency = column![
                text(format!(
                    "Cache cleaning frequency for ended series: {} days",
                    cache_settings.ended_cache_clean_frequency
                )),
                ended_clean_frequency
            ]
            .spacing(5);

            let heading_text = text("Automatic Cache Cleaning").size(18);
            let explaination_text = text("According to the settings, outdated cache is automatically \
                cleared on program starting, ensuring that the program has access to the most recent series data from the API. \
                Cache auto-clean frequency can be controlled by defining the number of days between each cache clean.").size(11);

            column![
                heading_text,
                explaination_text,
                vertical_space(10),
                aired_clean_frequency,
                waiting_release_frequency,
                ended_clean_frequency,
            ]
            .spacing(5)
            .into()
        }
    }
}
