use std::sync::mpsc;

use iced::{Command, Element, Renderer};
use iced_aw::Wrap;

use crate::core::api::series_information::SeriesMainInformation;
use crate::core::caching;
use crate::gui::troxide_widget::series_poster::{Message as SeriesPosterMessage, SeriesPoster};
use crate::gui::view::series_view;

#[derive(Debug, Clone)]
pub enum Message {
    SeriesPosters(usize, SeriesPosterMessage),
    SeriesInformationReceived(Option<Vec<SeriesMainInformation>>),
}

pub struct MyShows {
    series_posters: Vec<SeriesPoster>,
    series_page_sender: mpsc::Sender<(series_view::Series, Command<series_view::Message>)>,
}

impl MyShows {
    pub fn new_as_ended_tracked_series(
        series_page_sender: mpsc::Sender<(series_view::Series, Command<series_view::Message>)>,
    ) -> (Self, Command<Message>) {
        (
            Self {
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
        series_page_sender: mpsc::Sender<(series_view::Series, Command<series_view::Message>)>,
    ) -> (Self, Command<Message>) {
        (
            Self {
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
        series_page_sender: mpsc::Sender<(series_view::Series, Command<series_view::Message>)>,
    ) -> (Self, Command<Message>) {
        (
            Self {
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
                let series_infos = series_infos.unwrap();

                let mut series_posters_commands = Vec::with_capacity(series_infos.len());
                let mut series_posters = Vec::with_capacity(series_infos.len());

                for (index, series_info) in series_infos.into_iter().enumerate() {
                    let (poster, command) = SeriesPoster::new(index, series_info);
                    series_posters.push(poster);
                    series_posters_commands.push(command);
                }
                self.series_posters = series_posters;
                Command::batch(series_posters_commands)
                    .map(|message| Message::SeriesPosters(message.get_id().unwrap_or(0), message))
            }
            Message::SeriesPosters(index, message) => {
                if let SeriesPosterMessage::SeriesPosterPressed(series_info) = message.clone() {
                    self.series_page_sender
                        .send(series_view::Series::from_series_information(*series_info))
                        .expect("failed to send the series page");
                    return Command::none();
                }
                self.series_posters[index]
                    .update(message)
                    .map(|message| Message::SeriesPosters(message.get_id().unwrap_or(0), message))
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        Wrap::with_elements(
            self.series_posters
                .iter()
                .map(|poster| {
                    poster.view().map(|message| {
                        Message::SeriesPosters(message.get_id().unwrap_or(0), message)
                    })
                })
                .collect(),
        )
        .line_spacing(5.0)
        .spacing(5.0)
        .into()
    }
}
