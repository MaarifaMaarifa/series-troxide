use cast_poster::{CastPoster, Message as CastMessage};
use iced::widget::{button, column, container, horizontal_space, row, svg, text, Space};
use iced::{Command, Element, Length, Renderer};
use iced_aw::{Spinner, Wrap};

use crate::core::{api::show_cast::Cast, caching};
use crate::gui::assets::get_static_cow_from_asset;
use crate::gui::assets::icons::{CHEVRON_DOWN, CHEVRON_UP};
use crate::gui::styles;

const INITIAL_CAST_NUMBER: usize = 20;

#[derive(Clone, Debug)]
pub enum Message {
    CastReceived(Vec<Cast>),
    CastAction(usize, CastMessage),
    Expand,
    Shrink,
}

enum LoadState {
    Loading,
    Loaded,
}

pub struct CastWidget {
    load_state: LoadState,
    cast: Vec<CastPoster>,
    is_expanded: bool,
}

impl CastWidget {
    pub fn new(series_id: u32) -> (Self, Command<Message>) {
        let cast_widget = Self {
            load_state: LoadState::Loading,
            cast: vec![],
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
                self.cast = cast_posters;
                Command::batch(posters_commands)
                    .map(|message| Message::CastAction(message.get_id(), message))
            }
            Message::CastAction(index, message) => {
                self.cast[index].update(message);
                Command::none()
            }
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
                if self.cast.is_empty() {
                    Space::new(0, 0).into()
                } else {
                    let cast_posters: Vec<_> = self
                        .cast
                        .iter()
                        .enumerate()
                        .take_while(|(index, _)| self.is_expanded || *index < INITIAL_CAST_NUMBER)
                        .map(|(_, poster)| {
                            poster
                                .view()
                                .map(|message| Message::CastAction(message.get_id(), message))
                        })
                        .collect();

                    column![
                        text("Top Casts").size(21),
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
        if self.cast.len() > INITIAL_CAST_NUMBER {
            let (info, expansion_icon, message) = if self.is_expanded {
                let svg_handle = svg::Handle::from_memory(get_static_cow_from_asset(CHEVRON_UP));
                let up_icon = svg(svg_handle)
                    .width(Length::Shrink)
                    .style(styles::svg_styles::colored_svg_theme());
                (text("less casts"), up_icon, Message::Shrink)
            } else {
                let svg_handle = svg::Handle::from_memory(get_static_cow_from_asset(CHEVRON_DOWN));
                let down_icon = svg(svg_handle)
                    .width(Length::Shrink)
                    .style(styles::svg_styles::colored_svg_theme());
                (text("more casts"), down_icon, Message::Expand)
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
        widget::{column, container, horizontal_space, image, row, text, Column, Row, Space},
        Command, Element, Font, Renderer,
    };

    use crate::{
        core::{api::show_cast::Cast, caching},
        gui::{helpers, styles},
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
            let mut content = Row::new().spacing(10);

            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);

                let image = image(image_handle).width(100);
                content = content.push(image);
            } else {
                content = content.push(Space::new(100, 140));
            };

            let mut cast_info = Column::new().width(150).spacing(3);

            cast_info = cast_info.push(column![
                text(&self.cast.person.name)
                    .style(styles::text_styles::purple_text_theme())
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

            let content = content.push(cast_info);

            container(content)
                .style(styles::container_styles::first_class_container_square_theme())
                .padding(7)
                .into()
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
