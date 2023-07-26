use cast_poster::{CastPoster, Message as CastMessage};
use iced::widget::{column, container, text, Space};
use iced::{Command, Element, Length, Renderer};
use iced_aw::{Spinner, Wrap};

use crate::core::{api::show_cast::Cast, caching};

#[derive(Clone, Debug)]
pub enum Message {
    CastReceived(Vec<Cast>),
    CastAction(usize, CastMessage),
}

enum LoadState {
    Loading,
    Loaded,
}

pub struct CastWidget {
    load_state: LoadState,
    cast: Vec<CastPoster>,
}

impl CastWidget {
    pub fn new(series_id: u32) -> (Self, Command<Message>) {
        let cast_widget = Self {
            load_state: LoadState::Loading,
            cast: vec![],
        };

        let cast_command = Command::perform(caching::show_cast::get_show_cast(series_id), |cast| {
            Message::CastReceived(cast.expect("Failed to get show cast"))
        });

        (cast_widget, cast_command)
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CastReceived(cast) => {
                self.load_state = LoadState::Loaded;
                let mut cast_posters = Vec::with_capacity(cast.len());
                let mut posters_commands = Vec::with_capacity(cast.len());
                for (index, person) in cast.into_iter().enumerate() {
                    let (cast_poster, poster_command) = CastPoster::new(index, person);
                    cast_posters.push(cast_poster);
                    posters_commands.push(poster_command);
                }
                self.cast = cast_posters;
                Command::batch(posters_commands)
                    .map(|message| Message::CastAction(message.get_id(), message))
            }
            Message::CastAction(index, message) => {
                self.cast[index].update(message);
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        match self.load_state {
            LoadState::Loading => {
                return container(Spinner::new())
                    .center_x()
                    .center_y()
                    .height(100)
                    .width(Length::Fill)
                    .into()
            }
            LoadState::Loaded => {
                if self.cast.is_empty() {
                    Space::new(0, 0).into()
                } else {
                    column![
                        text("Top Cast").size(25),
                        Wrap::with_elements(
                            self.cast
                                .iter()
                                .map(|poster| {
                                    poster.view().map(|message| {
                                        Message::CastAction(message.get_id(), message)
                                    })
                                })
                                .collect(),
                        )
                        .padding(5.0)
                        .line_spacing(5.0)
                        .spacing(5.0)
                    ]
                    .into()
                }
            }
        }
    }
}
mod cast_poster {
    use bytes::Bytes;
    use iced::{
        alignment,
        widget::{container, image, text, Column, Space},
        Command, Element, Renderer,
    };

    use crate::{
        core::{api::show_cast::Cast, caching},
        gui::styles,
    };

    #[derive(Debug, Clone)]
    pub enum Message {
        ImageReceived(usize, Option<Bytes>),
    }

    impl Message {
        pub fn get_id(&self) -> usize {
            match self {
                Message::ImageReceived(id, _) => id.to_owned(),
            }
        }
    }

    pub struct CastPoster {
        cast: Cast,
        image: Option<Bytes>,
    }

    impl CastPoster {
        pub fn new(id: usize, cast: Cast) -> (Self, Command<Message>) {
            let image = cast.person.image.clone();
            let poster = Self { cast, image: None };
            let poster_command = if let Some(image) = image {
                Command::perform(
                    caching::load_image(image.medium_image_url),
                    move |image_bytes| Message::ImageReceived(id, image_bytes),
                )
            } else {
                Command::none()
            };

            (poster, poster_command)
        }

        pub fn update(&mut self, message: Message) {
            match message {
                Message::ImageReceived(_, image) => self.image = image,
            }
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let mut content = Column::new();

            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);

                let image = image(image_handle).width(100);
                content = content.push(image);
            } else {
                content = content.push(Space::new(100, 140));
            };

            let name = text(format!(
                "{}\nas {}",
                self.cast.person.name, self.cast.character.name
            ))
            .horizontal_alignment(alignment::Horizontal::Center)
            .width(100)
            .height(45)
            .size(15);

            let content = content.push(name);

            container(content)
                .style(styles::container_styles::second_class_container_rounded_theme())
                .padding(7)
                .into()
        }
    }
}
