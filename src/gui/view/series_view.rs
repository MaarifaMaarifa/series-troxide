use crate::core::api::seasons_list::{get_seasons_list, Season as SeasonInfo};
use crate::core::api::series_information::SeriesMainInformation;
use crate::core::api::Image;
use crate::core::{caching, database};
use crate::gui::assets::get_static_cow_from_asset;
use crate::gui::assets::icons::{ARROW_LEFT, CHECK_CIRCLE, CHECK_CIRCLE_FILL};

use cast_widget::CastWidget;
use cast_widget::Message as CastWidgetMessage;
use mini_widgets::*;
use season_widget::Message as SeasonMessage;

use iced::widget::scrollable::Properties;
use iced::widget::{button, column, container, horizontal_space, image, row, scrollable, text};
use iced::widget::{svg, vertical_space, Column, Row};
use iced::{Command, Element, Length, Renderer};
use iced_aw::Spinner;

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
    image_bytes: Option<Vec<u8>>,
) -> container::Container<'_, Message, Renderer> {
    let mut content = column!();

    let mut main_info = row!().padding(5);

    // Putting the image to the main info
    if let Some(image_bytes) = image_bytes {
        let image_handle = image::Handle::from_memory(image_bytes);
        let image = image(image_handle).height(250);
        main_info = main_info.push(image);
    }

    // Getting genres
    // Putting series information to the main info
    let series_data = column!(
        // text(format!("Status: {}", series_information.status)),
        status_widget(series_information),
        genres_widget(series_information),
        language_widget(series_information),
        average_runtime_widget(series_information),
        rating_widget(series_information),
        network_widget(series_information),
        webchannel_widget(series_information),
        premiered_widget(series_information),
        ended_widget(series_information),
        summary_widget(series_information),
    )
    .spacing(3)
    .padding(5);

    main_info = main_info.push(series_data);

    content = content.push(main_info);

    container(scrollable(content))
}

fn top_bar(series_info: &SeriesMainInformation) -> Row<'_, Message, Renderer> {
    let back_icon_handle = svg::Handle::from_memory(get_static_cow_from_asset(ARROW_LEFT));
    let back_icon = svg(back_icon_handle).width(Length::Shrink);

    let track_button = if database::DB.get_series(series_info.id).is_some() {
        let tracked_icon_handle =
            svg::Handle::from_memory(get_static_cow_from_asset(CHECK_CIRCLE_FILL));
        let icon = svg(tracked_icon_handle).width(Length::Shrink);
        button(icon).on_press(Message::UntrackSeries)
    } else {
        let tracked_icon_handle = svg::Handle::from_memory(get_static_cow_from_asset(CHECK_CIRCLE));
        let icon = svg(tracked_icon_handle).width(Length::Shrink);
        button(icon).on_press(Message::TrackSeries)
    };

    row!(
        button(back_icon).on_press(Message::GoBack),
        horizontal_space(Length::Fill),
        text(&series_info.name).size(30),
        horizontal_space(Length::Fill),
        track_button,
    )
}

#[derive(Clone, Debug)]
pub enum Message {
    SeriesInfoObtained(Box<SeriesMainInformation>),
    SeriesImageLoaded(Option<Vec<u8>>),
    GoBack,
    SeasonsLoaded(Vec<SeasonInfo>),
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
    series_image: Option<Vec<u8>>,
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
            series_image: None,
            season_widgets: vec![],
            cast_widget,
        };

        let commands = [
            Command::batch(get_image_and_seasons(series_image, series_id)),
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

                return Command::batch(get_image_and_seasons(info_image, self.series_id));
            }
            Message::SeriesImageLoaded(image) => {
                self.series_image = image;
            }
            Message::GoBack => return Command::perform(async {}, |_| Message::GoBack),
            Message::SeasonsLoaded(season_list) => {
                self.season_widgets = season_list
                    .into_iter()
                    .enumerate()
                    .map(|(index, season)| {
                        season_widget::Season::new(index, season, self.series_id)
                    })
                    .collect()
            }
            Message::SeasonAction(index, message) => {
                return self.season_widgets[index].update(*message);
            }
            Message::TrackSeries => {
                let series = database::Series::new(
                    self.series_information.as_ref().unwrap().name.to_owned(),
                    self.series_id,
                );
                database::DB.track_series(self.series_information.as_ref().unwrap().id, &series);
            }
            Message::UntrackSeries => {
                database::DB.untrack_series(self.series_information.as_ref().unwrap().id);
            }
            Message::CastWidgetAction(message) => {
                return self
                    .cast_widget
                    .update(message)
                    .map(Message::CastWidgetAction)
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
                let seasons_widget = column!(
                    text("Seasons").size(25),
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
                    .spacing(5),
                    vertical_space(10),
                    text("Top Cast").size(25),
                    self.cast_widget.view().map(Message::CastWidgetAction),
                )
                .padding(10);

                let content = scrollable(column!(main_body, seasons_widget))
                    .vertical_scroll(Properties::new().scroller_width(5).width(1));
                column!(top_bar(self.series_information.as_ref().unwrap()), content).into()
            }
        }
    }
}

/// Returns two commands that requests series' image and seasons list
fn get_image_and_seasons(
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

    let seasons_list_command = Command::perform(get_seasons_list(series_id), |seasons_list| {
        Message::SeasonsLoaded(seasons_list.expect("Failed to load seasons"))
    });

    [image_command, seasons_list_command]
}
