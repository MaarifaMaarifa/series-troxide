pub mod series_poster {

    use std::sync::mpsc;

    use crate::core::api::tv_maze::episodes_information::Episode;
    use crate::core::api::tv_maze::series_information::{Rating, SeriesMainInformation};
    use crate::core::api::tv_maze::Image;
    use crate::core::caching::episode_list::EpisodeReleaseTime;
    use crate::core::posters_hiding::HIDDEN_SERIES;
    use crate::core::{caching, database};
    use crate::gui::assets::icons::{EYE_SLASH_FILL, STAR_FILL};
    use crate::gui::helpers::{self, season_episode_str_gen};
    pub use crate::gui::message::IndexedMessage;
    use crate::gui::styles;

    use bytes::Bytes;
    use iced::font::Weight;
    use iced::widget::{
        button, column, container, horizontal_space, image, mouse_area, progress_bar, row, svg,
        text, vertical_space, Space,
    };
    use iced::{Command, Element, Font, Length, Renderer};

    #[derive(Clone, Debug)]
    pub enum Message {
        ImageLoaded(usize, Option<Bytes>),
        SeriesPosterPressed,
        Expand,
        Hide,
        SeriesHidden,
    }

    pub struct SeriesPoster {
        index: usize,
        series_information: SeriesMainInformation,
        image: Option<Bytes>,
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
        expanded: bool,
        hidden: bool,
    }

    impl SeriesPoster {
        pub fn new(
            index: usize,
            series_information: SeriesMainInformation,
            series_page_sender: mpsc::Sender<SeriesMainInformation>,
        ) -> (Self, Command<IndexedMessage<Message>>) {
            let image_url = series_information.image.clone();

            let poster = Self {
                index,
                series_information,
                image: None,
                series_page_sender,
                expanded: false,
                hidden: false,
            };

            let series_image_command = poster_image_command(index, image_url);

            (
                poster,
                series_image_command.map(move |message| IndexedMessage::new(index, message)),
            )
        }

        pub fn update(
            &mut self,
            message: IndexedMessage<Message>,
        ) -> Command<IndexedMessage<Message>> {
            match message.message() {
                Message::ImageLoaded(_, image) => self.image = image,
                Message::SeriesPosterPressed => {
                    self.series_page_sender
                        .send(self.series_information.clone())
                        .expect("failed to send series page info");
                }
                Message::Expand => self.expanded = !self.expanded,
                Message::Hide => {
                    let series_id = self.series_information.id;
                    let index = self.index;
                    let series_name = self.series_information.name.clone();
                    let premiered_date = self.series_information.premiered.clone();

                    return Command::perform(
                        async move {
                            let mut hidden_series = HIDDEN_SERIES.write().await;

                            hidden_series
                                .hide_series(series_id, series_name, premiered_date)
                                .await
                        },
                        |_| Message::SeriesHidden,
                    )
                    .map(move |message| IndexedMessage::new(index, message));
                }
                Message::SeriesHidden => {
                    self.hidden = true;
                }
            }
            Command::none()
        }

        /// Views the series poster widget
        ///
        /// This is the normal view of the poster, just having the image of the
        /// of the series and it's name below it
        pub fn normal_view(
            &self,
            expandable: bool,
        ) -> Element<'_, IndexedMessage<Message>, Renderer> {
            // let mut content = column![].padding(2).spacing(1);

            let poster_image: Element<'_, Message, Renderer> = {
                let image_height = if self.expanded { 170 } else { 140 };
                if let Some(image_bytes) = self.image.clone() {
                    let image_handle = image::Handle::from_memory(image_bytes);
                    image(image_handle).height(image_height).into()
                } else {
                    Space::new(image_height as f32 / 1.4, image_height).into()
                }
            };

            let content: Element<'_, Message, Renderer> = if self.expanded {
                let metadata = column![
                    text(&self.series_information.name)
                        .size(11)
                        .font(Font {
                            weight: Weight::Bold,
                            ..Default::default()
                        })
                        .style(styles::text_styles::accent_color_theme()),
                    Self::genres_widget(&self.series_information.genres),
                    Self::premier_widget(self.series_information.premiered.as_deref()),
                    Self::rating_widget(&self.series_information.rating),
                    vertical_space(5),
                    Self::hiding_button(),
                ]
                .spacing(2);

                row![poster_image, metadata]
                    .padding(2)
                    .spacing(5)
                    .width(300)
                    .into()
            } else {
                let mut content = column![].padding(2).spacing(1);
                content = content.push(poster_image);
                content = content.push(
                    text(&self.series_information.name)
                        .size(11)
                        .width(100)
                        .height(30)
                        .vertical_alignment(iced::alignment::Vertical::Center)
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                );
                content.into()
            };

            let content = container(content)
                .padding(5)
                .style(styles::container_styles::second_class_container_rounded_theme());

            let mut mouse_area = mouse_area(content).on_press(Message::SeriesPosterPressed);

            if expandable {
                mouse_area = mouse_area.on_right_press(Message::Expand);
            }

            let element: Element<'_, Message, Renderer> = mouse_area.into();
            element.map(|message| IndexedMessage::new(self.index, message))
        }

        fn rating_widget(rating: &Rating) -> Element<'_, Message, Renderer> {
            if let Some(average_rating) = rating.average {
                let star_handle = svg::Handle::from_memory(STAR_FILL);
                let star_icon = svg(star_handle)
                    .width(15)
                    .height(15)
                    .style(styles::svg_styles::colored_svg_theme());

                row![star_icon, text(average_rating).size(11)]
                    .spacing(5)
                    .into()
            } else {
                Space::new(0, 0).into()
            }
        }

        fn premier_widget(premier_date: Option<&str>) -> Element<'_, Message, Renderer> {
            if let Some(premier_date) = premier_date {
                text(format!("Premiered: {}", premier_date)).size(11).into()
            } else {
                Space::new(0, 0).into()
            }
        }

        fn genres_widget(genres: &[String]) -> Element<'_, Message, Renderer> {
            if genres.is_empty() {
                Space::new(0, 0).into()
            } else {
                text(helpers::genres_with_pipes(genres)).size(11).into()
            }
        }

        fn hiding_button() -> Element<'static, Message, Renderer> {
            let tracked_icon_handle = svg::Handle::from_memory(EYE_SLASH_FILL);
            let icon = svg(tracked_icon_handle)
                .width(15)
                .height(15)
                .style(styles::svg_styles::colored_svg_theme());

            let content = row![icon, text("Hide from Discover").size(11)].spacing(5);

            button(content)
                .on_press(Message::Hide)
                .style(styles::button_styles::transparent_button_with_rounded_border_theme())
                .into()
        }

        /// View intended for the watchlist tab
        ///
        /// Consists of the Series image to the left and it's metadata (progress bar, etc)
        /// related to stuffs left to watch (episodes, time, etc)
        pub fn watchlist_view(
            &self,
            next_episode_to_watch: Option<&Episode>,
            total_episodes: usize,
        ) -> Element<'_, IndexedMessage<Message>, Renderer> {
            let mut content = row!().padding(2).spacing(5);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).width(100);
                content = content.push(image);
            } else {
                content = content.push(Space::new(100, 140));
            };

            let mut metadata = column!().padding(2).spacing(5);

            metadata = metadata.push(
                text(&self.series_information.name)
                    .size(18)
                    .style(styles::text_styles::accent_color_theme()),
            );
            metadata = metadata.push(vertical_space(10));

            let watched_episodes = database::DB
                .get_series(self.series_information.id)
                .map(|series| series.get_total_episodes())
                .unwrap_or(0);

            let progress_bar = row![
                progress_bar(0.0..=total_episodes as f32, watched_episodes as f32,)
                    .height(10)
                    .width(500),
                text(format!(
                    "{}/{}",
                    watched_episodes as f32, total_episodes as f32
                ))
            ]
            .spacing(5);

            metadata = metadata.push(progress_bar);

            if let Some(next_episode_to_watch) = next_episode_to_watch {
                let season_number = next_episode_to_watch.season;
                let episode_number = next_episode_to_watch
                    .number
                    .expect("episode should have a valid number at this point");
                let episode_name = next_episode_to_watch.name.as_str();
                let episode_text = text(format!(
                    "{}: {}",
                    season_episode_str_gen(season_number, episode_number),
                    episode_name
                ));
                metadata = metadata.push(episode_text);
            };

            let episodes_left = total_episodes - watched_episodes;

            metadata = metadata.push(text(format!("{} episodes left", episodes_left)));

            if let Some(runtime) = self.series_information.average_runtime {
                let time = helpers::time::SaneTime::new(runtime * episodes_left as u32)
                    .get_time_plurized();
                let watchtime: String = time
                    .into_iter()
                    .rev()
                    .map(|(time_text, time_value)| format!("{} {} ", time_value, time_text))
                    .collect();
                metadata = metadata.push(text(watchtime));
            };

            content = content.push(metadata);

            let content = container(content)
                .padding(5)
                .style(styles::container_styles::first_class_container_rounded_theme())
                .width(1000);

            let element: Element<'_, Message, Renderer> = mouse_area(content)
                .on_press(Message::SeriesPosterPressed)
                .into();
            element.map(|message| IndexedMessage::new(self.index, message))
        }

        /// View intended for the upcoming releases
        ///
        /// This view is intended to be used in my_shows tab for the series whose next release
        /// episode is known
        pub fn release_series_posters_view(
            &self,
            episode_and_release_time: (&Episode, &EpisodeReleaseTime),
        ) -> Element<'_, IndexedMessage<Message>, Renderer> {
            let mut content = row!().padding(2).spacing(7);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).width(100);
                content = content.push(image);
            } else {
                content = content.push(Space::new(100, 140));
            };

            let mut metadata = column!().spacing(5);
            metadata = metadata.push(
                text(&self.series_information.name)
                    .size(18)
                    .style(styles::text_styles::accent_color_theme()),
            );
            // Some separation between series name and the rest of content
            metadata = metadata.push(vertical_space(10));

            let season_number = episode_and_release_time.0.season;
            let episode_number = episode_and_release_time
                .0
                .number
                .expect("an episode should have a valid number");

            let episode_name = &episode_and_release_time.0.name;

            metadata = metadata.push(text(format!(
                "{}: {}",
                season_episode_str_gen(season_number, episode_number),
                episode_name,
            )));

            metadata = metadata.push(text(
                episode_and_release_time.1.get_full_release_date_and_time(),
            ));

            content = content.push(metadata);

            content = content.push(horizontal_space(Length::Fill));
            let release_time_widget = container(
                container(
                    helpers::time::SaneTime::new(
                        episode_and_release_time
                            .1
                            .get_remaining_release_duration()
                            .num_minutes() as u32,
                    )
                    .get_time_plurized()
                    .into_iter()
                    .last()
                    .map(|(time_text, time_value)| {
                        column![text(time_value), text(time_text),]
                            .align_items(iced::Alignment::Center)
                    })
                    .unwrap_or(column![text("Now")]),
                )
                .width(70)
                .height(70)
                .padding(5)
                .center_x()
                .center_y()
                .style(styles::container_styles::release_time_container_theme()),
            )
            .center_x()
            .center_y()
            .height(140);

            content = content.push(release_time_widget);

            let content = container(content)
                .padding(5)
                .style(styles::container_styles::first_class_container_rounded_theme())
                .width(1000);

            let element: Element<'_, Message, Renderer> = mouse_area(content)
                .on_press(Message::SeriesPosterPressed)
                .into();
            element.map(|message| IndexedMessage::new(self.index, message))
        }
    }

    fn poster_image_command(id: usize, image: Option<Image>) -> Command<Message> {
        if let Some(image) = image {
            Command::perform(
                async move {
                    caching::load_image(image.medium_image_url, caching::ImageType::Medium).await
                },
                move |image| Message::ImageLoaded(id, image),
            )
        } else {
            Command::none()
        }
    }
}

pub mod title_bar {
    use iced::widget::{
        button, container, horizontal_space, mouse_area, row, svg, text, Row, Space,
    };
    use iced::{Element, Length, Renderer};

    use crate::gui::assets::icons::CARET_LEFT_FILL;
    use crate::gui::styles;
    use crate::gui::tabs::TabLabel;

    #[derive(Clone, Debug)]
    pub enum Message {
        TabSelected(usize),
        BackButtonPressed,
    }

    pub struct TitleBar {
        active_tab: usize,
    }

    impl TitleBar {
        pub fn new() -> Self {
            Self {
                active_tab: usize::default(),
            }
        }

        pub fn update(&mut self, message: Message) {
            #[allow(irrefutable_let_patterns)]
            if let Message::TabSelected(new_active_tab) = message {
                self.active_tab = new_active_tab
            }
        }

        pub fn view(
            &self,
            tab_labels: &[TabLabel],
            show_back_button: bool,
        ) -> iced::Element<'_, Message, Renderer> {
            let tab_views = tab_labels
                .iter()
                .enumerate()
                .map(|(index, tab_label)| {
                    let svg_handle = svg::Handle::from_memory(tab_label.icon);
                    let icon = svg(svg_handle)
                        .width(Length::Shrink)
                        .style(styles::svg_styles::colored_svg_theme());
                    let text_label = text(&tab_label.text);
                    let mut tab = container(
                        mouse_area(row![icon, text_label].spacing(5))
                            .on_press(Message::TabSelected(index)),
                    )
                    .padding(5);

                    // Highlighting the tab if is active
                    if index == self.active_tab {
                        tab = tab
                            .style(styles::container_styles::second_class_container_square_theme())
                    };
                    tab.into()
                })
                .collect();

            let tab_views = Row::with_children(tab_views).spacing(10);

            let back_button: Element<'_, Message, Renderer> = if show_back_button {
                let back_button_icon_handle = svg::Handle::from_memory(CARET_LEFT_FILL);
                let icon = svg(back_button_icon_handle)
                    .width(20)
                    .style(styles::svg_styles::colored_svg_theme());
                button(icon)
                    .on_press(Message::BackButtonPressed)
                    .style(styles::button_styles::transparent_button_theme())
                    .into()
            } else {
                Space::new(0, 0).into()
            };

            container(row![
                back_button,
                horizontal_space(Length::Fill),
                tab_views,
                horizontal_space(Length::Fill)
            ])
            .style(styles::container_styles::first_class_container_square_theme())
            .into()
        }
    }
}
