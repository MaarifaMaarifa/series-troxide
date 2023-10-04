use std::sync::mpsc;

use crate::core::api::tv_maze::series_information::{Genre, SeriesMainInformation};
use crate::core::caching::tv_schedule::full_schedule;
use crate::gui::troxide_widget::series_poster::{
    IndexedMessage as SeriesPosterIndexedMessage, Message as SeriesPosterMessage, SeriesPoster,
};

use iced::widget::{column, container, text, Space};
use iced::{Command, Element, Length, Renderer};
use iced_aw::{Spinner, Wrap};

#[derive(Debug, Clone)]
pub enum Message {
    FullScheduleLoaded(full_schedule::FullSchedule),
    SeriesPoster(SeriesPosterIndexedMessage<SeriesPosterMessage>),
}

enum LoadState {
    Loading,
    Loaded,
}
pub struct SeriesSuggestion {
    series_id: u32,
    genres: Vec<Genre>,
    load_state: LoadState,
    suggested_series: Vec<SeriesPoster>,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
}

impl SeriesSuggestion {
    pub fn new(
        series_id: u32,
        genres: Vec<Genre>,
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Command<Message>) {
        (
            Self {
                genres,
                load_state: LoadState::Loading,
                series_id,
                suggested_series: vec![],
                series_page_sender,
            },
            Command::perform(full_schedule::FullSchedule::new(), |schedule| {
                Message::FullScheduleLoaded(schedule.expect("failed to load the full schedule"))
            }),
        )
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FullScheduleLoaded(full_schedule) => {
                self.load_state = LoadState::Loaded;

                let mut series_infos = full_schedule.get_series_by_genres(20, &self.genres);

                let poster_index_to_remove = series_infos
                    .iter()
                    .enumerate()
                    .find(|(_, series_info)| series_info.id == self.series_id)
                    .map(|(index, _)| index);

                // preventing the parent series from appearing in the suggestions
                if let Some(index) = poster_index_to_remove {
                    series_infos.remove(index);
                }

                let mut posters = Vec::with_capacity(series_infos.len());
                let mut posters_commands = Vec::with_capacity(series_infos.len());
                for (index, series_info) in series_infos.into_iter().enumerate() {
                    let (poster, poster_command) =
                        SeriesPoster::new(index, series_info, self.series_page_sender.clone());
                    posters.push(poster);
                    posters_commands.push(poster_command);
                }

                self.suggested_series = posters;
                Command::batch(posters_commands).map(Message::SeriesPoster)
            }
            Message::SeriesPoster(message) => self.suggested_series[message.index()]
                .update(message)
                .map(Message::SeriesPoster),
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        match self.load_state {
            LoadState::Loading => container(Spinner::new())
                .width(Length::Fill)
                .center_x()
                .into(),
            LoadState::Loaded => {
                if self.suggested_series.is_empty() {
                    Space::new(0, 0).into()
                } else {
                    column![
                        text("You may also like").size(21),
                        Wrap::with_elements(
                            self.suggested_series
                                .iter()
                                .map(|poster| poster.normal_view(false).map(Message::SeriesPoster))
                                .collect(),
                        )
                        .line_spacing(5.0)
                        .spacing(5.0)
                    ]
                    .padding(10)
                    .spacing(5)
                    .into()
                }
            }
        }
    }
}
