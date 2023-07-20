use iced::widget::{button, column, container, horizontal_space, row, text, Button, Text};
use iced::{Command, Element, Length, Renderer};

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

mod cache_cleaning {
    use crate::core::caching::cache_cleaning::{CacheCleaner, CleanType, RunningStatus};

    pub async fn clean_ended_cache() -> anyhow::Result<()> {
        let mut cleaner = CacheCleaner::new()?;
        cleaner.clean_cache(CleanType::Ended).await
    }

    pub async fn cleaning_waiting_release_cache() -> anyhow::Result<()> {
        let mut cleaner = CacheCleaner::new()?;
        cleaner
            .clean_cache(CleanType::Running(RunningStatus::WaitingRelease))
            .await
    }

    pub async fn clean_aired_cache() -> anyhow::Result<()> {
        let mut cleaner = CacheCleaner::new()?;
        cleaner
            .clean_cache(CleanType::Running(RunningStatus::Aired))
            .await
    }
}
