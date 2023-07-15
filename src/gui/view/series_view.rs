use crate::core::api::episodes_information::Episode;
use crate::core::api::series_information::SeriesMainInformation;
use crate::core::api::Image;
use crate::core::caching::episode_list::EpisodeReleaseTime;
use crate::core::{caching, database};
use crate::gui::assets::get_static_cow_from_asset;
use crate::gui::assets::icons::{ARROW_LEFT, CHECK_CIRCLE, CHECK_CIRCLE_FILL};
use crate::gui::styles;

use bytes::Bytes;
use cast_widget::CastWidget;
use cast_widget::Message as CastWidgetMessage;
use mini_widgets::*;
use season_widget::Message as SeasonMessage;

use iced::widget::scrollable::Properties;
use iced::widget::{button, column, container, horizontal_space, image, row, scrollable, text};
use iced::widget::{svg, vertical_space, Column, Row};
use iced::{Alignment, Command, Element, Length, Renderer};
use iced_aw::{Grid, Spinner};

mod cast_widget;
mod mini_widgets;
mod season_widget;

#[derive(PartialEq)]
pub enum SeriesStatus {
    Running,
    Ended,
    ToBeDetermined,
    InDevelopment,
    Other,
}

impl SeriesStatus {
    pub fn new(series_info: &SeriesMainInformation) -> Self {
        match series_info.status.as_ref() {
            "Running" => Self::Running,
            "Ended" => Self::Ended,
            "To Be Determined" => Self::ToBeDetermined,
            "In Development" => Self::InDevelopment,
            _ => Self::Other,
        }
    }
}

/// Generates the Series Page
pub fn series_page(
    series_information: &SeriesMainInformation,
    image_bytes: Option<Bytes>,
) -> Column<'_, Message, Renderer> {
    let content = column!();

    let mut main_info = row!().padding(5).spacing(10);

    // Putting the image to the main info
    if let Some(image_bytes) = image_bytes {
        let image_handle = image::Handle::from_memory(image_bytes);
        let image = image(image_handle).height(250);
        main_info = main_info.push(image);
    }

    let mut series_data_grid = Grid::with_columns(2);

    let status_widget = status_widget(series_information);
    let genres_widget = genres_widget(series_information);
    let language_widget = language_widget(series_information);
    let average_runtime_widget = average_runtime_widget(series_information);
    let rating_widget = rating_widget(series_information);
    let network_widget = network_widget(series_information);
    let webchannel_widget = webchannel_widget(series_information);
    let premiered_widget = premiered_widget(series_information);
    let ended_widget = ended_widget(series_information);
    let summary = summary_widget(series_information);

    series_data_grid.insert(status_widget.0);
    series_data_grid.insert(status_widget.1);

    if let Some(genres_widget) = genres_widget {
        series_data_grid.insert(genres_widget.0);
        series_data_grid.insert(genres_widget.1);
    };
    series_data_grid.insert(language_widget.0);
    series_data_grid.insert(language_widget.1);
    series_data_grid.insert(average_runtime_widget.0);
    series_data_grid.insert(average_runtime_widget.1);
    series_data_grid.insert(rating_widget.0);
    series_data_grid.insert(rating_widget.1);

    if let Some(network_widget) = network_widget {
        series_data_grid.insert(network_widget.0);
        series_data_grid.insert(network_widget.1);
    };

    if let Some(webchannel_widget) = webchannel_widget {
        series_data_grid.insert(webchannel_widget.0);
        series_data_grid.insert(webchannel_widget.1);
    };

    series_data_grid.insert(premiered_widget.0);
    series_data_grid.insert(premiered_widget.1);

    if let Some(ended_widget) = ended_widget {
        series_data_grid.insert(ended_widget.0);
        series_data_grid.insert(ended_widget.1);
    };

    let series_data = column![series_data_grid, vertical_space(10), summary,].spacing(5);

    main_info = main_info.push(series_data);

    content.push(main_info).width(Length::Fill)
}

fn top_bar(series_info: &SeriesMainInformation) -> Row<'_, Message, Renderer> {
    let back_icon_handle = svg::Handle::from_memory(get_static_cow_from_asset(ARROW_LEFT));
    let back_icon = svg(back_icon_handle)
        .width(Length::Shrink)
        .style(styles::svg_styles::colored_svg_theme());

    let track_button = if database::DB
        .get_series(series_info.id)
        .map(|series| series.is_tracked())
        .unwrap_or(false)
    {
        let tracked_icon_handle =
            svg::Handle::from_memory(get_static_cow_from_asset(CHECK_CIRCLE_FILL));
        let icon = svg(tracked_icon_handle)
            .width(Length::Shrink)
            .style(styles::svg_styles::colored_svg_theme());
        button(icon)
            .on_press(Message::UntrackSeries)
            .style(styles::button_styles::transparent_button_theme())
    } else {
        let tracked_icon_handle = svg::Handle::from_memory(get_static_cow_from_asset(CHECK_CIRCLE));
        let icon = svg(tracked_icon_handle)
            .width(Length::Shrink)
            .style(styles::svg_styles::colored_svg_theme());
        button(icon)
            .on_press(Message::TrackSeries)
            .style(styles::button_styles::transparent_button_theme())
    };

    row!(
        button(back_icon)
            .on_press(Message::GoBack)
            .style(styles::button_styles::transparent_button_theme()),
        horizontal_space(Length::Fill),
        text(&series_info.name).size(30),
        horizontal_space(Length::Fill),
        track_button,
    )
}

#[derive(Clone, Debug)]
pub enum Message {
    SeriesInfoObtained(Box<SeriesMainInformation>),
    SeriesImageLoaded(Option<Bytes>),
    EpisodeListLoaded(caching::episode_list::EpisodeList),
    GoBack,
    SeasonAction(usize, Box<SeasonMessage>),
    CastWidgetAction(CastWidgetMessage),
    TrackSeries,
    UntrackSeries,
}

enum LoadState {
    Loading,
    Loaded,
}

pub struct Series {
    series_id: u32,
    load_state: LoadState,
    series_information: Option<SeriesMainInformation>,
    series_image: Option<Bytes>,
    next_episode_release_time: Option<(Episode, EpisodeReleaseTime)>,
    season_widgets: Vec<season_widget::Season>,
    cast_widget: CastWidget,
}

impl Series {
    /// Counstruct the series page by providing it with id
    pub fn from_series_id(series_id: u32) -> (Self, Command<Message>) {
        let (cast_widget, cast_widget_command) = CastWidget::new(series_id);
        let series = Self {
            series_id,
            load_state: LoadState::Loading,
            series_information: None,
            next_episode_release_time: None,
            series_image: None,
            season_widgets: vec![],
            cast_widget,
        };

        let series_command = Command::perform(
            caching::series_information::get_series_main_info_with_id(series_id),
            |info| {
                Message::SeriesInfoObtained(Box::new(
                    info.expect("Failed to load series information"),
                ))
            },
        );

        (
            series,
            Command::batch([
                series_command,
                cast_widget_command.map(Message::CastWidgetAction),
            ]),
        )
    }

    /// Counstruct the series page by providing it with SeriesMainInformation
    pub fn from_series_information(
        series_information: SeriesMainInformation,
    ) -> (Self, Command<Message>) {
        let series_id = series_information.id;
        let (cast_widget, cast_widget_command) = CastWidget::new(series_id);
        let series_image = series_information.image.clone();
        let series = Self {
            series_id,
            load_state: LoadState::Loaded,
            series_information: Some(series_information),
            next_episode_release_time: None,
            series_image: None,
            season_widgets: vec![],
            cast_widget,
        };

        let commands = [
            Command::batch(get_image_and_episode_list(series_image, series_id)),
            cast_widget_command.map(Message::CastWidgetAction),
        ];

        (series, Command::batch(commands))
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SeriesInfoObtained(info) => {
                self.load_state = LoadState::Loaded;
                let info_image = info.image.clone();
                self.series_information = Some(*info);

                return Command::batch(get_image_and_episode_list(info_image, self.series_id));
            }
            Message::SeriesImageLoaded(image) => {
                self.series_image = image;
            }
            Message::GoBack => return Command::perform(async {}, |_| Message::GoBack),
            Message::SeasonAction(index, message) => {
                return self.season_widgets[index].update(*message);
            }
            Message::TrackSeries => {
                let series_id = self.series_information.as_ref().unwrap().id;

                if let Some(mut series) = database::DB.get_series(series_id) {
                    series.mark_tracked();
                } else {
                    let mut series = database::Series::new(
                        self.series_information.as_ref().unwrap().name.to_owned(),
                        self.series_id,
                    );
                    series.mark_tracked();
                    database::DB.add_series(self.series_information.as_ref().unwrap().id, &series);
                }
            }
            Message::UntrackSeries => {
                let series_id = self.series_information.as_ref().unwrap().id;
                if let Some(mut series) = database::DB.get_series(series_id) {
                    series.mark_untracked();
                }
                // database::DB.untrack_series(self.series_information.as_ref().unwrap().id);
            }
            Message::CastWidgetAction(message) => {
                return self
                    .cast_widget
                    .update(message)
                    .map(Message::CastWidgetAction)
            }
            Message::EpisodeListLoaded(episode_list) => {
                let season_and_total_episodes =
                    episode_list.get_season_numbers_with_total_episode();

                self.season_widgets = season_and_total_episodes
                    .into_iter()
                    .enumerate()
                    .map(|(index, season)| {
                        season_widget::Season::new(
                            index,
                            self.series_id,
                            self.series_information.as_ref().unwrap().clone().name,
                            season.0,
                            season.1,
                        )
                    })
                    .collect();

                self.next_episode_release_time = episode_list
                    .get_next_episode_and_time()
                    .map(|(episode, release_time)| (episode.clone(), release_time))
            }
        }
        Command::none()
    }

    pub fn view(&self) -> Element<Message, Renderer> {
        match self.load_state {
            LoadState::Loading => container(Spinner::new())
                .height(Length::Fill)
                .width(Length::Fill)
                .center_x()
                .center_y()
                .into(),
            LoadState::Loaded => {
                let main_body = series_page(
                    self.series_information.as_ref().unwrap(),
                    self.series_image.clone(),
                );

                let seasons_widget = container(
                    container(
                        column![
                            text("Seasons").size(25),
                            vertical_space(10),
                            Column::with_children(
                                self.season_widgets
                                    .iter()
                                    .enumerate()
                                    .map(|(index, widget)| {
                                        widget
                                            .view()
                                            .map(move |m| Message::SeasonAction(index, Box::new(m)))
                                    })
                                    .collect(),
                            )
                            .padding(5)
                            .spacing(5)
                            .align_items(Alignment::Center)
                        ]
                        .align_items(Alignment::Center),
                    )
                    .padding(10)
                    .style(styles::container_styles::first_class_container_theme()),
                )
                .width(Length::Fill)
                .center_x()
                .center_y();

                let seasons_widget = column![
                    next_episode_release_time_widget(self),
                    seasons_widget,
                    vertical_space(10),
                    text("Top Cast").size(25),
                    self.cast_widget.view().map(Message::CastWidgetAction),
                ]
                .padding(10);

                let content = scrollable(column!(main_body, seasons_widget))
                    .vertical_scroll(Properties::new().scroller_width(5).width(1));
                column!(top_bar(self.series_information.as_ref().unwrap()), content).into()
            }
        }
    }
}

/// Returns two commands that requests series' image and seasons list
fn get_image_and_episode_list(
    series_info_image: Option<Image>,
    series_id: u32,
) -> [Command<Message>; 2] {
    let image_command = if let Some(image_url) = series_info_image {
        Command::perform(caching::load_image(image_url.original_image_url), |image| {
            Message::SeriesImageLoaded(image)
        })
    } else {
        Command::none()
    };

    let episode_list_command =
        Command::perform(get_episodes_list(series_id), Message::EpisodeListLoaded);

    [image_command, episode_list_command]
}

/// Returns the episodes_list of the current series
async fn get_episodes_list(series_id: u32) -> caching::episode_list::EpisodeList {
    caching::episode_list::EpisodeList::new(series_id)
        .await
        .expect("failed to get episodes list")
}
