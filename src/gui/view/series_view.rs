use crate::core::api::episodes_information::Episode;
use crate::core::api::series_information::SeriesMainInformation;
use crate::core::api::Image;
use crate::core::caching::episode_list::EpisodeReleaseTime;
use crate::core::{caching, database};
use crate::gui::assets::get_static_cow_from_asset;
use crate::gui::assets::icons::{PATCH_PLUS, PATCH_PLUS_FILL};
use crate::gui::styles;

use bytes::Bytes;
use cast_widget::CastWidget;
use cast_widget::Message as CastWidgetMessage;
use mini_widgets::*;
use season_widget::Message as SeasonMessage;

use iced::widget::{
    button, column, container, horizontal_rule, image, row, scrollable, text, Button, Space,
};
use iced::widget::{svg, vertical_space, Column};
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

/// Generates the Series Metadata
pub fn series_metadata<'a>(
    series_information: &'a SeriesMainInformation,
    image_bytes: Option<Bytes>,
    next_episode_release_time: Option<&'a (Episode, EpisodeReleaseTime)>,
) -> Element<'a, Message, Renderer> {
    let mut main_info = row!().padding(5).spacing(10);

    if let Some(image_bytes) = image_bytes {
        let image_handle = image::Handle::from_memory(image_bytes);
        let image = image(image_handle).width(180);

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

    let series_name = text(series_information.name.clone())
        .size(35)
        .style(styles::text_styles::purple_text_theme());

    let title_bar = row![
        series_name.width(Length::FillPortion(10)),
        tracking_button(series_information.id)
    ];

    let next_episode_widget = next_episode_release_time_widget(next_episode_release_time);

    let series_data = column![
        title_bar,
        rating_widget,
        horizontal_rule(1),
        series_data_grid,
        vertical_space(10),
        next_episode_widget
    ]
    .width(700)
    .spacing(5);

    main_info = main_info.push(series_data);

    let content = container(
        column![main_info, summary]
            .align_items(Alignment::Center)
            .padding(5)
            .width(Length::Fill),
    )
    .style(styles::container_styles::first_class_container_tab_theme());

    container(content)
        .width(Length::Fill)
        .padding(10)
        .center_x()
        .into()
}

fn background(background_bytes: Option<Bytes>) -> Element<'static, Message, Renderer> {
    if let Some(image_bytes) = background_bytes {
        let image_handle = image::Handle::from_memory(image_bytes);
        image(image_handle)
            .width(Length::Fill)
            .height(300)
            .content_fit(iced::ContentFit::Cover)
            .into()
    } else {
        Space::new(0, 300).into()
    }
}

fn tracking_button(series_id: u32) -> Button<'static, Message, Renderer> {
    if database::DB
        .get_series(series_id)
        .map(|series| series.is_tracked())
        .unwrap_or(false)
    {
        let tracked_icon_handle =
            svg::Handle::from_memory(get_static_cow_from_asset(PATCH_PLUS_FILL));
        let icon = svg(tracked_icon_handle)
            .width(30)
            .height(30)
            .style(styles::svg_styles::colored_svg_theme());
        button(icon).on_press(Message::UntrackSeries)
    } else {
        let tracked_icon_handle = svg::Handle::from_memory(get_static_cow_from_asset(PATCH_PLUS));
        let icon = svg(tracked_icon_handle)
            .width(30)
            .height(30)
            .style(styles::svg_styles::colored_svg_theme());
        button(icon).on_press(Message::TrackSeries)
    }
    .style(styles::button_styles::transparent_button_theme())
}

#[derive(Clone, Debug)]
pub enum Message {
    SeriesInfoObtained(Box<SeriesMainInformation>),
    SeriesImageLoaded(Option<Bytes>),
    SeriesBackgroundLoaded(Option<Bytes>),
    EpisodeListLoaded(caching::episode_list::EpisodeList),
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
    seasons_load_state: LoadState,
    series_information: Option<SeriesMainInformation>,
    series_image: Option<Bytes>,
    series_background: Option<Bytes>,
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
            seasons_load_state: LoadState::Loading,
            series_information: None,
            next_episode_release_time: None,
            series_image: None,
            series_background: None,
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
            seasons_load_state: LoadState::Loading,
            series_information: Some(series_information),
            next_episode_release_time: None,
            series_image: None,
            series_background: None,
            season_widgets: vec![],
            cast_widget,
        };

        let commands = [
            Command::batch(get_images_and_episode_list(series_image, series_id)),
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

                return Command::batch(get_images_and_episode_list(info_image, self.series_id));
            }
            Message::SeriesImageLoaded(image) => {
                self.series_image = image;
            }
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
            }
            Message::CastWidgetAction(message) => {
                return self
                    .cast_widget
                    .update(message)
                    .map(Message::CastWidgetAction)
            }
            Message::EpisodeListLoaded(episode_list) => {
                self.seasons_load_state = LoadState::Loaded;
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
            Message::SeriesBackgroundLoaded(background) => self.series_background = background,
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
                let background = background(self.series_background.clone());

                let series_metadata = series_metadata(
                    self.series_information.as_ref().unwrap(),
                    self.series_image.clone(),
                    self.next_episode_release_time.as_ref(),
                );

                let seasons_widget = self.seasons_view();

                let cast_widget = self.cast_widget.view().map(Message::CastWidgetAction);

                let content = column![
                    background,
                    series_metadata,
                    vertical_space(10),
                    seasons_widget,
                    cast_widget,
                ];

                scrollable(content).into()
            }
        }
    }

    fn seasons_view(&self) -> Element<'_, Message, Renderer> {
        let seasons_body = column![text("Seasons").size(25)]
            .align_items(Alignment::Center)
            .spacing(10);

        let content = if let LoadState::Loading = self.seasons_load_state {
            container(seasons_body.push(Spinner::new()))
                .width(700)
                .center_x()
        } else if self.season_widgets.is_empty() {
            container(seasons_body.push(text("No seasons found")))
                .width(700)
                .center_x()
        } else {
            container(
                seasons_body.push(
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
                    .align_items(Alignment::Center),
                ),
            )
        }
        .padding(10)
        .style(styles::container_styles::first_class_container_theme());

        container(content)
            .width(Length::Fill)
            .padding(10)
            .center_x()
            .center_y()
            .into()
    }
}

/// Returns two commands that requests series' image and seasons list
fn get_images_and_episode_list(
    series_info_image: Option<Image>,
    series_id: u32,
) -> [Command<Message>; 3] {
    let image_command = if let Some(image_url) = series_info_image {
        Command::perform(caching::load_image(image_url.original_image_url), |image| {
            Message::SeriesImageLoaded(image)
        })
    } else {
        Command::none()
    };

    let background_command = Command::perform(
        caching::show_images::get_recent_banner(series_id),
        Message::SeriesBackgroundLoaded,
    );

    let episode_list_command =
        Command::perform(get_episodes_list(series_id), Message::EpisodeListLoaded);

    [image_command, background_command, episode_list_command]
}

/// Returns the episodes_list of the current series
async fn get_episodes_list(series_id: u32) -> caching::episode_list::EpisodeList {
    caching::episode_list::EpisodeList::new(series_id)
        .await
        .expect("failed to get episodes list")
}
