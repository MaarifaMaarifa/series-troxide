use std::collections::HashSet;

use crate::core::caching;
use crate::core::{api::series_information::SeriesMainInformation, database};
use crate::gui::troxide_widget::series_poster::{Message as SeriesPosterMessage, SeriesPoster};
use crate::gui::troxide_widget::{GREEN_THEME, RED_THEME};
use crate::gui::{Message as GuiMessage, Tab};
use iced::widget::{container, scrollable};
use iced_aw::{Spinner, Wrap};

use iced::Length;
use iced::{
    widget::{column, text},
    Command, Element, Renderer,
};

use super::series_view::SeriesStatus;

#[derive(Debug, Clone)]
pub enum Message {
    SeriesInformationsReceived(Vec<SeriesMainInformation>),
    SeriesSelected(Box<SeriesMainInformation>),
    SeriesPosterAction(usize, SeriesPosterMessage),
}

#[derive(Default)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Default)]
pub struct MyShowsTab {
    load_state: LoadState,
    series_ids: Vec<String>,
    series: Vec<SeriesPoster>,
}

impl MyShowsTab {
    pub fn refresh(&mut self) -> Command<Message> {
        let series_ids = database::DB.get_series_id_collection();
        let fresh_series_ids: HashSet<String> = series_ids.iter().cloned().collect();

        let current_series_ids: HashSet<String> = self.series_ids.iter().cloned().collect();

        // Preventing my_shows page from reloading when no series updates has occured in the database
        if (self.series.is_empty() && !series_ids.is_empty())
            || (fresh_series_ids != current_series_ids)
        {
            self.load_state = LoadState::Loading;
            self.series_ids = series_ids.clone();

            let series_information =
                caching::series_information::get_series_main_info_with_ids(series_ids);

            return Command::perform(series_information, |series_infos| {
                Message::SeriesInformationsReceived(series_infos)
            });
        } else {
            Command::none()
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SeriesSelected(_) => {
                unimplemented!("My shows page should not handle selecting a series poster")
            }
            Message::SeriesPosterAction(index, message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_info) = message {
                    return Command::perform(async {}, |_| Message::SeriesSelected(series_info));
                }
                return self.series[index]
                    .update(message)
                    .map(move |message| Message::SeriesPosterAction(index, message));
            }
            Message::SeriesInformationsReceived(series_infos) => {
                self.load_state = LoadState::Loaded;

                let mut series_posters = Vec::with_capacity(series_infos.len());
                let mut series_posters_commands = Vec::with_capacity(series_infos.len());

                for (index, series_info) in series_infos.into_iter().enumerate() {
                    let (series_poster, series_poster_command) =
                        SeriesPoster::new(index, series_info);
                    series_posters.push(series_poster);
                    series_posters_commands.push(series_poster_command);
                }
                self.series = series_posters;
                Command::batch(series_posters_commands).map(|message| {
                    Message::SeriesPosterAction(message.get_id().unwrap_or(0), message)
                })
            }
        }
    }

    pub fn view(&self) -> Element<Message, Renderer> {
        match self.load_state {
            LoadState::Loading => container(Spinner::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into(),
            LoadState::Loaded => {
                let running_shows =
                    Wrap::with_elements(Self::filter_posters(&self.series, SeriesStatus::Running))
                        .spacing(5.0)
                        .padding(5.0);
                let ended_shows =
                    Wrap::with_elements(Self::filter_posters(&self.series, SeriesStatus::Ended))
                        .spacing(5.0)
                        .padding(5.0);

                let content = column!(
                    text("Running").size(20).style(GREEN_THEME),
                    running_shows,
                    text("Ended").size(20).style(RED_THEME),
                    ended_shows
                )
                .spacing(5.0)
                .width(Length::Fill)
                .padding(5.0);

                scrollable(content).into()
            }
        }
    }

    fn filter_posters(
        posters: &Vec<SeriesPoster>,
        status: SeriesStatus,
    ) -> Vec<Element<'_, Message, Renderer>> {
        posters
            .iter()
            .filter(|poster| poster.get_status().unwrap() == status)
            .map(|poster| {
                poster.view().map(|message| {
                    Message::SeriesPosterAction(message.get_id().unwrap_or(0), message)
                })
            })
            .collect()
    }
}

impl Tab for MyShowsTab {
    type Message = GuiMessage;

    fn title(&self) -> String {
        "My Shows".to_owned()
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        iced_aw::TabLabel::Text("My Shows icon".to_owned())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        self.view().map(GuiMessage::MyShows)
    }
}
