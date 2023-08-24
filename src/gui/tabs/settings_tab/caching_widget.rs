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
        let manual_cleaning_explaination_text = text("Sometimes, you may want to perform manual cache clean up for some reasons. \
            All of the cache type can be cleaned up manually. When their individual clean buttons are pressed, a clean will be performed immediately and the last clean \
            record for that type of clean will be update.").size(11);

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
        column![
            text("Ended Cache Cleaning"),
            row![
                text("clean cache for the series that have ended").size(11),
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
            text("Aired Cache Cleaning"),
            row![
                text("clean cache for the series that are currently being aired").size(11),
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
            text("Waiting Release Cache Cleaning"),
            row![
                text("clean cache for the series waiting for their release date").size(11),
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
    use iced::widget::{column, text, vertical_space};
    use iced::{Element, Renderer};
    use iced_aw::{Grid, NumberInput};

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
            let cache_settigns = &settings.get_current_settings().cache;

            let aired_clean_frequency = NumberInput::new(
                cache_settigns.aired_cache_clean_frequency,
                u32::MAX,
                Message::Aired,
            );
            let ended_clean_frequency = NumberInput::new(
                cache_settigns.ended_cache_clean_frequency,
                u32::MAX,
                Message::Ended,
            );
            let waiting_release_frequency = NumberInput::new(
                cache_settigns.waiting_release_cache_clean_frequency,
                u32::MAX,
                Message::WaitingRelease,
            );

            let mut grid = Grid::with_columns(2);

            grid.insert(text("Aired series cache cleaning frequency(days)"));
            grid.insert(aired_clean_frequency);

            grid.insert(text(
                "Waiting release series cache cleaning frequency(days)    ",
            )); // This being the longest text, a tab is added at the end to keep spacing between columns in the grid
            grid.insert(waiting_release_frequency);

            grid.insert(text("Ended series cache cleaning frequency(days)"));
            grid.insert(ended_clean_frequency);

            let heading_text = text("Automatic Cache Cleaning").size(18);
            let explaination_text = text("Outdated cache are cleaned up automatically during \
                program startup based on the settings, this makes the program have the latest series data from the API. \
                You can manage cache auto-clean frequency by setting after how many days each cache clean should be performed.").size(11);

            column![heading_text, explaination_text, vertical_space(10), grid,]
                .spacing(5)
                .into()
        }
    }
}
