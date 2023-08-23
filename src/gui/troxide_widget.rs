pub mod series_poster {

    use crate::core::api::episodes_information::Episode;
    use crate::core::api::series_information::SeriesMainInformation;
    use crate::core::api::Image;
    use crate::core::caching::episode_list::EpisodeReleaseTime;
    use crate::core::{caching, database};
    use crate::gui::helpers::{self, season_episode_str_gen};
    use crate::gui::styles;

    use bytes::Bytes;
    use iced::widget::{
        column, container, horizontal_space, image, mouse_area, progress_bar, row, text,
        vertical_space, Space,
    };
    use iced::{Command, Element, Length, Renderer};

    #[derive(Clone, Debug)]
    pub enum Message {
        ImageLoaded(usize, Option<Bytes>),
        SeriesPosterPressed(Box<SeriesMainInformation>),
    }

    impl Message {
        /// Returns the index given to the `Series Poster` when it was created
        ///
        /// Since `Series Posters` are normaly stored in a `Vec` and run Commands
        /// this method provide a simple way of sending back the Command Message
        /// to the appropriate `Series Poster` in a `Vec`
        pub fn get_index(&self) -> Option<usize> {
            if let Self::ImageLoaded(id, _) = self {
                return Some(id.to_owned());
            }
            None
        }
    }

    #[derive(PartialEq, Eq, Hash)]
    pub struct SeriesPoster {
        series_information: SeriesMainInformation,
        image: Option<Bytes>,
    }

    impl SeriesPoster {
        pub fn new(
            index: usize,
            series_information: SeriesMainInformation,
        ) -> (Self, Command<Message>) {
            let image_url = series_information.image.clone();

            let poster = Self {
                series_information,
                image: None,
            };

            let series_image_command = poster_image_command(index, image_url);

            (poster, series_image_command)
        }

        pub fn update(&mut self, message: Message) -> Command<Message> {
            match message {
                Message::ImageLoaded(_, image) => self.image = image,
                Message::SeriesPosterPressed(_) => {
                    unreachable!("the series poster should not handle being pressed")
                }
            }
            Command::none()
        }

        /// Views the series poster widget
        ///
        /// This is the normal view of the poster, just having the image of the
        /// of the series and it's name below it
        pub fn normal_view(&self) -> Element<'_, Message, Renderer> {
            let mut content = column!().padding(2).spacing(1);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).width(100);
                content = content.push(image);
            } else {
                content = content.push(Space::new(100, 140));
            };

            content = content.push(
                text(&self.series_information.name)
                    .size(11)
                    .width(100)
                    .height(30)
                    .vertical_alignment(iced::alignment::Vertical::Center)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
            );

            let content = container(content)
                .padding(5)
                .style(styles::container_styles::second_class_container_rounded_theme());

            mouse_area(content)
                .on_press(Message::SeriesPosterPressed(Box::new(
                    self.series_information.clone(),
                )))
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
        ) -> Element<'_, Message, Renderer> {
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
                    .style(styles::text_styles::purple_text_theme()),
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

            mouse_area(content)
                .on_press(Message::SeriesPosterPressed(Box::new(
                    self.series_information.clone(),
                )))
                .into()
        }

        /// View intended for the upcoming releases
        ///
        /// This view is intended to be used in my_shows tab for the series whose next release
        /// episode is known
        pub fn release_series_posters_view(
            &self,
            episode_and_release_time: (&Episode, &EpisodeReleaseTime),
        ) -> Element<'_, Message, Renderer> {
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
                    .style(styles::text_styles::purple_text_theme()),
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

            mouse_area(content)
                .on_press(Message::SeriesPosterPressed(Box::new(
                    self.series_information.clone(),
                )))
                .into()
        }
    }

    fn poster_image_command(id: usize, image: Option<Image>) -> Command<Message> {
        if let Some(image) = image {
            Command::perform(
                async move { caching::load_image(image.medium_image_url).await },
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

    use crate::gui::assets::get_static_cow_from_asset;
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
                    let svg_handle =
                        svg::Handle::from_memory(get_static_cow_from_asset(tab_label.icon));
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
                let back_button_icon_handle =
                    svg::Handle::from_memory(get_static_cow_from_asset(CARET_LEFT_FILL));
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
