use cast_poster::{CastPoster, IndexedMessage, Message as CastMessage};
use iced::widget::{button, column, container, horizontal_space, row, svg, text, Space};
use iced::{Command, Element, Length, Renderer};
use iced_aw::{Spinner, Wrap};

use crate::core::{api::tv_maze::show_cast::Cast, caching};
use crate::gui::assets::icons::{CHEVRON_DOWN, CHEVRON_UP};
use crate::gui::styles;

const INITIAL_CAST_NUMBER: usize = 20;

#[derive(Clone, Debug)]
pub enum Message {
    CastReceived(Vec<Cast>),
    Cast(IndexedMessage<usize, CastMessage>),
    Expand,
    Shrink,
}

enum LoadState {
    Loading,
    Loaded,
}

pub struct CastWidget {
    load_state: LoadState,
    casts: Vec<CastPoster>,
    is_expanded: bool,
}

impl CastWidget {
    pub fn new(series_id: u32) -> (Self, Command<Message>) {
        let cast_widget = Self {
            load_state: LoadState::Loading,
            casts: vec![],
            is_expanded: false,
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
                self.casts = cast_posters;
                Command::batch(posters_commands).map(Message::Cast)
            }
            Message::Cast(message) => self.casts[message.index()]
                .update(message)
                .map(Message::Cast),
            Message::Expand => {
                self.is_expanded = true;
                Command::none()
            }
            Message::Shrink => {
                self.is_expanded = false;
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
                if self.casts.is_empty() {
                    Space::new(0, 0).into()
                } else {
                    let cast_posters: Vec<_> = self
                        .casts
                        .iter()
                        .enumerate()
                        .take_while(|(index, _)| self.is_expanded || *index < INITIAL_CAST_NUMBER)
                        .map(|(_, poster)| poster.view().map(Message::Cast))
                        .collect();

                    column![
                        text("Cast").size(21),
                        Wrap::with_elements(cast_posters)
                            .padding(5.0)
                            .line_spacing(10.0)
                            .spacing(10.0),
                        self.expansion_widget(),
                    ]
                    .padding(5)
                    .into()
                }
            }
        }
    }

    fn expansion_widget(&self) -> Element<'_, Message, Renderer> {
        if self.casts.len() > INITIAL_CAST_NUMBER {
            let (info, expansion_icon, message) = if self.is_expanded {
                let svg_handle = svg::Handle::from_memory(CHEVRON_UP);
                let up_icon = svg(svg_handle)
                    .width(Length::Shrink)
                    .style(styles::svg_styles::colored_svg_theme());
                (text("show less"), up_icon, Message::Shrink)
            } else {
                let svg_handle = svg::Handle::from_memory(CHEVRON_DOWN);
                let down_icon = svg(svg_handle)
                    .width(Length::Shrink)
                    .style(styles::svg_styles::colored_svg_theme());
                (text("show more"), down_icon, Message::Expand)
            };

            let content = row![
                horizontal_space(5),
                info,
                expansion_icon,
                horizontal_space(5),
            ]
            .spacing(10)
            .align_items(iced::Alignment::Center);

            let content = button(content)
                .on_press(message)
                .style(styles::button_styles::transparent_button_theme());

            container(
                container(content)
                    .style(styles::container_styles::first_class_container_square_theme()),
            )
            .center_x()
            .width(Length::Fill)
            .padding(20)
            .into()
        } else {
            Space::new(0, 0).into()
        }
    }
}

mod cast_poster {
    use bytes::Bytes;
    use iced::{
        font::Weight,
        widget::{
            button, column, container, horizontal_space, image, row, svg, text, Column, Row, Space,
        },
        Command, Element, Font, Renderer,
    };

    pub use crate::gui::message::IndexedMessage;
    use crate::{
        core::{
            api::tv_maze::{show_cast::Cast, Image},
            caching,
        },
        gui::{assets::icons::ARROW_REPEAT, helpers, styles},
    };

    #[derive(Debug, Clone)]
    pub enum Message {
        PersonImageLoaded(Option<Bytes>),
        CharacterImageLoaded(Option<Bytes>),
        SwitchDisplayImage,
    }

    enum DisplayImage {
        Person,
        Character,
    }

    pub struct CastPoster {
        index: usize,
        cast: Cast,
        person_image: Option<Bytes>,
        character_image: Option<Bytes>,
        character_image_loading: bool,
        current_display_image: DisplayImage,
    }

    impl CastPoster {
        pub fn new(id: usize, cast: Cast) -> (Self, Command<IndexedMessage<usize, Message>>) {
            let image = cast.person.image.clone();
            let poster = Self {
                index: id,
                cast,
                person_image: None,
                character_image: None,
                character_image_loading: false,
                current_display_image: DisplayImage::Person,
            };
            let poster_command = Self::load_person_image(image);
            (
                poster,
                poster_command.map(move |message| IndexedMessage::new(id, message)),
            )
        }

        pub fn update(
            &mut self,
            message: IndexedMessage<usize, Message>,
        ) -> Command<IndexedMessage<usize, Message>> {
            let command = match message.message() {
                Message::PersonImageLoaded(image) => {
                    self.person_image = image;
                    Command::none()
                }
                Message::CharacterImageLoaded(image) => {
                    self.character_image = image;
                    self.character_image_loading = false;
                    Command::none()
                }
                Message::SwitchDisplayImage => match self.current_display_image {
                    DisplayImage::Person => {
                        if self.character_image.is_none() && !self.character_image_loading {
                            self.current_display_image = DisplayImage::Character;
                            self.character_image_loading = true;
                            Self::load_character_image(self.cast.character.image.clone())
                        } else {
                            self.current_display_image = DisplayImage::Character;
                            Command::none()
                        }
                    }
                    DisplayImage::Character => {
                        self.current_display_image = DisplayImage::Person;
                        Command::none()
                    }
                },
            };
            let index = self.index;
            command.map(move |message| IndexedMessage::new(index, message))
        }

        pub fn view(&self) -> Element<'_, IndexedMessage<usize, Message>, Renderer> {
            let mut content = Row::new().spacing(10);

            let empty_image = helpers::empty_image::empty_image().width(100).height(140);

            match self.current_display_image {
                DisplayImage::Person => {
                    if let Some(image_bytes) = self.person_image.clone() {
                        let image_handle = image::Handle::from_memory(image_bytes);

                        let image = image(image_handle).width(100);
                        content = content.push(image);
                    } else {
                        content = content.push(empty_image);
                    };
                }
                DisplayImage::Character => {
                    if let Some(image_bytes) = self.character_image.clone() {
                        let image_handle = image::Handle::from_memory(image_bytes);

                        let image = image(image_handle).width(100);
                        content = content.push(image);
                    } else {
                        content = content.push(empty_image);
                    };
                }
            }

            let mut cast_info = Column::new().width(150).spacing(3);

            cast_info = cast_info.push(column![
                text(&self.cast.person.name)
                    .style(styles::text_styles::accent_color_theme())
                    .size(15),
                text(format!("as {}", &self.cast.character.name)).size(11)
            ]);

            // A little bit of space between cast name and other informations
            cast_info = cast_info.push(horizontal_space(20));

            if let Some(gender) = self.cast.person.gender.as_ref() {
                cast_info = cast_info.push(cast_info_field("Gender: ", gender));
            }

            if self.cast.person.deathday.is_none() {
                if let Ok(duration_since_birth) = self.cast.duration_since_birth() {
                    if let Some(age) =
                        helpers::time::SaneTime::new(duration_since_birth.num_minutes() as u32)
                            .get_time_plurized()
                            .last()
                    {
                        cast_info = cast_info
                            .push(cast_info_field("Age: ", format!("{} {}", age.1, age.0)));
                    } else {
                        cast_info = cast_info.push(text("Just born"));
                    }
                }
            }

            if let Some(birthday) = self.cast.person.birthday.as_ref() {
                cast_info = cast_info.push(cast_info_field("Birthday: ", birthday));
            }

            if let Some(deathday) = self.cast.person.deathday.as_ref() {
                cast_info = cast_info.push(cast_info_field("Deathday: ", deathday));
            }

            if let Some(country) = self.cast.person.country.as_ref() {
                cast_info = cast_info.push(cast_info_field("Born in: ", &country.name));
            }

            cast_info = cast_info.push(self.image_switch_button());

            let content = content.push(cast_info);

            let element: Element<'_, Message, Renderer> = container(content)
                .style(styles::container_styles::first_class_container_square_theme())
                .padding(7)
                .into();
            element.map(|message| IndexedMessage::new(self.index, message))
        }

        fn image_switch_button(&self) -> Element<'_, Message, Renderer> {
            if self.cast.character.image.is_some() {
                let image_switch_button_handle = svg::Handle::from_memory(ARROW_REPEAT);
                let icon =
                    svg(image_switch_button_handle).style(styles::svg_styles::colored_svg_theme());

                let mut button =
                    button(icon).style(styles::button_styles::transparent_button_theme());

                if !self.character_image_loading {
                    button = button.on_press(Message::SwitchDisplayImage);
                }
                button.into()
            } else {
                Space::new(0, 0).into()
            }
        }

        fn load_person_image(image: Option<Image>) -> Command<Message> {
            if let Some(image) = image {
                Command::perform(
                    caching::load_image(image.medium_image_url, caching::ImageType::Medium),
                    Message::PersonImageLoaded,
                )
            } else {
                Command::none()
            }
        }

        fn load_character_image(image: Option<Image>) -> Command<Message> {
            if let Some(image) = image {
                Command::perform(
                    caching::load_image(image.medium_image_url, caching::ImageType::Medium),
                    Message::CharacterImageLoaded,
                )
            } else {
                Command::none()
            }
        }
    }

    fn cast_info_field(
        title: &str,
        value: impl std::fmt::Display,
    ) -> Element<'_, Message, Renderer> {
        row![
            text(title)
                .font(Font {
                    weight: Weight::Bold,
                    ..Default::default()
                })
                .size(12),
            text(value).size(12)
        ]
        .into()
    }
}
