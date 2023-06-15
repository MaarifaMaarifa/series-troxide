use iced::widget::{checkbox, progress_bar, row, text};
use iced::{Command, Element, Renderer};

use crate::core::api::seasons_list::Season as SeasonInfo;
use crate::gui::Message as GuiMessage;

#[derive(Clone, Debug)]
pub enum Message {
    CheckboxPressed(bool),
}

pub struct Season {
    season: SeasonInfo,
    is_tracked: bool,
}

impl Season {
    pub fn new(season_info: SeasonInfo) -> Self {
        Self {
            season: season_info,
            is_tracked: false,
        }
    }
    pub fn update(&mut self, message: Message) -> Command<GuiMessage> {
        match message {
            Message::CheckboxPressed(tracking_status) => {
                if let Some(_) = self.season.episode_order {
                    self.is_tracked = tracking_status;
                }
            }
        }
        Command::none()
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let track_checkbox = checkbox("", self.is_tracked, |tracking| {
            Message::CheckboxPressed(tracking)
        });
        let season_name = text(format!("Season {}", self.season.number));
        let season_progress = if let Some(episodes_number) = self.season.episode_order {
            progress_bar(0.0..=episodes_number as f32, 0.0)
                .height(10)
                .width(500)
        } else {
            progress_bar(0.0..=0.0, 0.0).height(10).width(500)
        };
        let episodes_progress = text(format!("{}/{}", 0, self.season.episode_order.unwrap_or(0)));

        row!(
            track_checkbox,
            season_name,
            season_progress,
            episodes_progress
        )
        .into()
    }
}
