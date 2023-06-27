// The text size of the beginning part of a info
pub const INFO_HEADER: u16 = 18;
// The text size of the main part of a info
pub const INFO_BODY: u16 = 15;

// const INFO_BODY_HEIGHT: u16 = INFO_HEADER - (INFO_HEADER - INFO_BODY);

pub mod series_poster {

    use crate::core::api::load_image;
    use crate::core::api::series_information::SeriesMainInformation;
    use iced::widget::{column, image, mouse_area, text};
    use iced::{Command, Element, Renderer};

    #[derive(Clone, Debug)]
    pub enum Message {
        ImageLoaded(usize, Option<Vec<u8>>),
        SeriesPosterPressed(Box<SeriesMainInformation>),
    }

    impl Message {
        pub fn get_id(&self) -> Option<usize> {
            if let Self::ImageLoaded(id, _) = self {
                Some(id.to_owned())
            } else {
                None
            }
        }
    }

    pub struct SeriesPoster {
        series_information: SeriesMainInformation,
        image: Option<Vec<u8>>,
    }

    impl SeriesPoster {
        pub fn new(
            id: usize,
            series_information: SeriesMainInformation,
        ) -> (Self, Command<Message>) {
            let image_url = series_information.image.clone();

            let poster = Self {
                series_information,
                image: None,
            };

            let series_image_command = if let Some(image) = image_url {
                Command::perform(
                    async move { load_image(image.medium_image_url).await },
                    move |image| Message::ImageLoaded(id, image),
                )
            } else {
                Command::none()
            };

            (poster, series_image_command)
        }

        pub fn update(&mut self, message: Message) -> Command<Message> {
            match message {
                Message::ImageLoaded(_, image) => self.image = image,
                Message::SeriesPosterPressed(_) => {
                    unimplemented!("the series poster should not handle being pressed")
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

            mouse_area(content)
                .on_press(Message::SeriesPosterPressed(Box::new(
                    self.series_information.clone(),
                )))
                .into()
        }
    }
}
