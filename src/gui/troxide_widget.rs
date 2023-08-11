pub mod series_poster {

    use crate::core::api::episodes_information::Episode;
    use crate::core::api::series_information::SeriesMainInformation;
    use crate::core::api::Image;
    use crate::core::caching::episode_list::EpisodeReleaseTime;
    use crate::core::{caching, database};
    use crate::gui::helpers::season_episode_str_gen;
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
        pub fn get_id(&self) -> Option<usize> {
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
            id: usize,
            series_information: SeriesMainInformation,
        ) -> (Self, Command<Message>) {
            let image_url = series_information.image.clone();

            let poster = Self {
                series_information,
                image: None,
            };

            let series_image_command = poster_image_command(id, image_url);

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
        pub fn view(&self) -> Element<'_, Message, Renderer> {
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
        pub fn watchlist_view(&self, total_episodes: usize) -> Element<'_, Message, Renderer> {
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

            let last_episode_watched = if let Some(series) =
                database::DB.get_series(self.series_information.id)
            {
                if let Some((season_num, last_watched_season)) = series.get_last_season() {
                    last_watched_season.get_last_episode();
                    text(format!("{} {}","Last watched", season_episode_str_gen(season_num, last_watched_season.get_last_episode().expect("the season should have atleast one episode for it to be the last watched"))))
                } else {
                    text("No Episode Watched")
                }
            } else {
                text("No Episode Watched")
            };

            metadata = metadata.push(last_episode_watched);

            let episodes_left = total_episodes - watched_episodes;

            metadata = metadata.push(text(format!("{} episodes left", episodes_left)));

            if let Some(runtime) = self.series_information.average_runtime {
                let watchtime = format!(
                    "Average time left to complete, {} minutes",
                    runtime * episodes_left as u32
                );
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

        pub fn release_series_posters_view(
            &self,
            episode_and_release_time: (&Episode, EpisodeReleaseTime),
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
                    text(
                        &episode_and_release_time
                            .1
                            .get_remaining_release_time()
                            .unwrap(),
                    )
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
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

pub mod tabs {
    use iced::widget::{column, container, horizontal_space, mouse_area, row, svg, text, Row};
    use iced::{Element, Length, Renderer};

    use crate::gui::assets::get_static_cow_from_asset;
    use crate::gui::styles;

    pub struct TabLabel {
        pub text: String,
        pub icon: &'static [u8],
    }

    impl TabLabel {
        pub fn new(text: String, icon: &'static [u8]) -> Self {
            Self { text, icon }
        }
    }

    pub struct Tabs<'a, Message> {
        active_tab: usize,
        on_select: Box<dyn Fn(usize) -> Message>,
        tab_labels: Vec<TabLabel>,
        current_tab_view: Element<'a, Message, Renderer>,
    }

    impl<'a, Message> Tabs<'a, Message>
    where
        Message: Clone + 'a,
    {
        pub fn with_labels<F>(
            tab_labels: Vec<TabLabel>,
            current_tab_view: Element<'a, Message, Renderer>,
            on_select: F,
        ) -> Self
        where
            F: 'static + Fn(usize) -> Message,
        {
            Self {
                active_tab: usize::default(),
                tab_labels,
                on_select: Box::new(on_select),
                current_tab_view,
            }
        }

        pub fn set_active_tab(mut self, tab_id: usize) -> Self {
            self.active_tab = tab_id;
            self
        }

        fn tab_view(&self) -> iced::Element<'a, Message, Renderer> {
            let tab_views = self
                .tab_labels
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
                            .on_press((self.on_select)(index)),
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

            container(row![
                horizontal_space(Length::Fill),
                tab_views,
                horizontal_space(Length::Fill)
            ])
            .style(styles::container_styles::first_class_container_square_theme())
            .into()
        }

        pub fn view(self) -> Element<'a, Message, Renderer> {
            let tab_view = self.tab_view();
            column![tab_view, self.current_tab_view].into()
        }
    }
}
