use std::sync::mpsc;

use iced::widget::{container, text};
use iced::{Command, Element, Length, Renderer};
use iced_aw::{Spinner, Wrap};

use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::caching;
use crate::gui::styles;
use crate::gui::troxide_widget::series_poster::{
    IndexedMessage, Message as SeriesPosterMessage, SeriesPoster,
};

#[derive(Debug, Clone)]
pub enum Message {
    SeriesPosters(IndexedMessage<usize, SeriesPosterMessage>),
    SeriesInformationReceived(Option<Vec<SeriesMainInformation>>),
}

#[derive(Default)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

pub struct MyShows<'a> {
    load_state: LoadState,
    series_posters: Vec<SeriesPoster<'a>>,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
}

impl<'a> MyShows<'a> {
    pub fn new_as_ended_tracked_series(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Command<Message>) {
        (
            Self {
                load_state: LoadState::default(),
                series_posters: vec![],
                series_page_sender,
            },
            Command::perform(
                async {
                    caching::series_list::SeriesList::new()
                        .get_ended_tracked_series_informations()
                        .await
                },
                move |res| Message::SeriesInformationReceived(res.ok()),
            ),
        )
    }

    pub fn new_as_waiting_release_series(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Command<Message>) {
        (
            Self {
                load_state: LoadState::default(),
                series_posters: vec![],
                series_page_sender,
            },
            Command::perform(
                async {
                    caching::series_list::SeriesList::new()
                        .get_waiting_release_series_informations()
                        .await
                },
                |res| Message::SeriesInformationReceived(res.ok()),
            ),
        )
    }

    pub fn new_as_untracked_series(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Command<Message>) {
        (
            Self {
                load_state: LoadState::default(),
                series_posters: vec![],
                series_page_sender,
            },
            Command::perform(
                async {
                    caching::series_list::SeriesList::new()
                        .get_untracked_series_informations()
                        .await
                },
                |res| Message::SeriesInformationReceived(res.ok()),
            ),
        )
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SeriesInformationReceived(series_infos) => {
                self.load_state = LoadState::Loaded;
                let mut series_infos = series_infos.unwrap();

                // sorting the list according to name
                series_infos.sort_by_key(|series_info| series_info.name.clone());

                let mut series_posters_commands = Vec::with_capacity(series_infos.len());
                let mut series_posters = Vec::with_capacity(series_infos.len());

                for (index, series_info) in series_infos.into_iter().enumerate() {
                    let (poster, command) = SeriesPoster::new(
                        index,
                        std::borrow::Cow::Owned(series_info),
                        self.series_page_sender.clone(),
                    );
                    series_posters.push(poster);
                    series_posters_commands.push(command);
                }
                self.series_posters = series_posters;
                Command::batch(series_posters_commands).map(Message::SeriesPosters)
            }
            Message::SeriesPosters(message) => self.series_posters[message.index()]
                .update(message)
                .map(Message::SeriesPosters),
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        if let LoadState::Loading = self.load_state {
            return container(Spinner::new())
                .center_x()
                .center_y()
                .height(100)
                .width(Length::Fill)
                .into();
        }
        if self.series_posters.is_empty() {
            container(text("Nothing to show"))
                .style(styles::container_styles::first_class_container_square_theme())
                .center_x()
                .center_y()
                .height(200)
                .width(Length::Fill)
                .into()
        } else {
            Wrap::with_elements(
                self.series_posters
                    .iter()
                    .map(|poster| poster.view(false).map(Message::SeriesPosters))
                    .collect(),
            )
            .line_spacing(5.0)
            .spacing(5.0)
            .into()
        }
    }
}
