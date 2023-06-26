use crate::core::api::series_information;
use crate::core::{api::series_information::SeriesMainInformation, database};
use crate::gui::Message as GuiMessage;
use iced::widget::container;
use iced_aw::{Spinner, Wrap};

use iced::Length;
use iced::{
    widget::{column, text, Column},
    Command, Element, Renderer,
};
use series_poster::Message as SeriesPosterMessage;

#[derive(Debug, Clone)]
pub enum Message {
    SeriesInformationsReceived(Vec<SeriesMainInformation>),
    SeriesSelected(Box<SeriesMainInformation>),
    SeriesPosterAction(usize, Box<SeriesPosterMessage>),
}

#[derive(Default)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Default)]
pub struct MyShows {
    load_state: LoadState,
    series: Vec<series_poster::SeriesPoster>,
}

impl MyShows {
    pub fn new() -> (Self, Command<GuiMessage>) {
        let series_id = database::DB.get_series_id_collection();
        let series_information = series_information::get_series_main_info_with_ids(series_id);

        (
            Self {
                load_state: LoadState::Loading,
                series: vec![],
            },
            Command::perform(series_information, |series_infos| {
                GuiMessage::MyShowsAction(Message::SeriesInformationsReceived(series_infos))
            }),
        )
    }

    pub fn update(&mut self, message: Message) -> Command<GuiMessage> {
        match message {
            Message::SeriesSelected(_) => {
                unimplemented!("My shows page should not handle selecting a series poster")
            }
            Message::SeriesPosterAction(index, message) => {
                return self.series[index]
                    .update(*message)
                    .map(GuiMessage::MyShowsAction)
            }
            Message::SeriesInformationsReceived(series_infos) => {
                self.load_state = LoadState::Loaded;

                let mut series_posters = Vec::with_capacity(series_infos.len());
                let mut series_posters_commands = Vec::with_capacity(series_infos.len());

                for (index, series_info) in series_infos.into_iter().enumerate() {
                    let (series_poster, series_poster_command) =
                        series_poster::SeriesPoster::new(index, series_info);
                    series_posters.push(series_poster);
                    series_posters_commands.push(series_poster_command);
                }
                self.series = series_posters;
                Command::batch(series_posters_commands).map(GuiMessage::MyShowsAction)
            }
        }
    }

    pub fn view(&self) -> Element<Message, Renderer> {
        let title = text("Tracked Shows").size(30);

        match self.load_state {
            LoadState::Loading => container(Spinner::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into(),
            LoadState::Loaded => column!(
                title,
                Wrap::with_elements(
                    self.series
                        .iter()
                        .enumerate()
                        .map(|(index, poster)| poster.view().map(move |message| {
                            Message::SeriesPosterAction(index, Box::new(message))
                        }))
                        .collect()
                )
                .spacing(5.0)
                .padding(5.0)
            )
            .padding(5)
            .into(),
        }
    }
}

mod series_poster {

    use crate::core::api::load_image;
    use crate::core::api::series_information::SeriesMainInformation;
    use iced::widget::{column, image, mouse_area, text};
    use iced::{Command, Element, Renderer};

    use super::Message as MyShowsMessage;

    #[derive(Clone, Debug)]
    pub enum Message {
        ImageLoaded(Option<Vec<u8>>),
        SeriesPosterPressed(Box<SeriesMainInformation>),
    }

    pub struct SeriesPoster {
        series_information: SeriesMainInformation,
        image: Option<Vec<u8>>,
    }

    impl SeriesPoster {
        pub fn new(
            index: usize,
            series_information: SeriesMainInformation,
        ) -> (Self, Command<MyShowsMessage>) {
            let image_url = series_information.image.clone();

            let poster = Self {
                series_information,
                image: None,
            };

            let series_image_command = if let Some(image) = image_url {
                Command::perform(
                    async move { load_image(image.medium_image_url).await },
                    move |image| {
                        MyShowsMessage::SeriesPosterAction(
                            index,
                            Box::new(Message::ImageLoaded(image)),
                        )
                    },
                )
            } else {
                Command::none()
            };

            (poster, series_image_command)
        }

        pub fn update(&mut self, message: Message) -> Command<MyShowsMessage> {
            match message {
                Message::ImageLoaded(image) => self.image = image,
                Message::SeriesPosterPressed(series_information) => {
                    return Command::perform(async {}, move |_| {
                        MyShowsMessage::SeriesSelected(series_information)
                    })
                }
            }
            Command::none()
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let mut content = column!().padding(2).spacing(1);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).width(100);
                content = content.push(image);
            };

            content = content.push(
                text(&self.series_information.name)
                    .size(15)
                    .width(100)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
            );

            // content.push(text(&self.episode.name)).into()
            mouse_area(content)
                .on_press(Message::SeriesPosterPressed(Box::new(
                    self.series_information.clone(),
                )))
                .into()
        }
    }
}
