use std::sync::mpsc;

use bytes::Bytes;
use image;

use crate::core::api::tv_maze::episodes_information::Episode;
use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::api::tv_maze::Image;
use crate::core::caching::episode_list::EpisodeReleaseTime;
use crate::core::{caching, database};
use crate::gui::assets::icons::{PATCH_PLUS, PATCH_PLUS_FILL};
use crate::gui::{helpers, styles};
use cast_widget::{CastWidget, Message as CastWidgetMessage};
use data_widgets::*;
use season_widget::{Message as SeasonsMessage, Seasons};
use series_suggestion_widget::{Message as SeriesSuggestionMessage, SeriesSuggestion};

use iced::widget::scrollable::{Id, RelativeOffset, Viewport};
use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, scrollable, text, Button,
    Space,
};
use iced::widget::{svg, vertical_space};
use iced::{Alignment, Command, Element, Length, Renderer};
use iced_aw::Grid;

mod cast_widget;
mod data_widgets;
mod season_widget;
mod series_suggestion_widget;

/// Generates the Series Metadata
pub fn series_metadata<'a>(
    series_information: &'a SeriesMainInformation,
    image_bytes: Option<Bytes>,
    next_episode_release_time: Option<(&'a Episode, EpisodeReleaseTime)>,
) -> Element<'a, Message, Renderer> {
    let mut main_info = row!().padding(5).spacing(10);

    if let Some(image_bytes) = image_bytes {
        let image_handle = iced::widget::image::Handle::from_memory(image_bytes);
        let image = iced::widget::image(image_handle).width(180);

        main_info = main_info.push(image);
    } else {
        main_info = main_info.push(helpers::empty_image::empty_image().width(180).height(253));
    };

    let mut series_data_grid = Grid::with_columns(2);

    status_widget(series_information, &mut series_data_grid);
    series_type_widget(series_information, &mut series_data_grid);
    genres_widget(series_information, &mut series_data_grid);
    language_widget(series_information, &mut series_data_grid);
    average_runtime_widget(series_information, &mut series_data_grid);
    network_widget(series_information, &mut series_data_grid);
    webchannel_widget(series_information, &mut series_data_grid);
    premiered_widget(series_information, &mut series_data_grid);
    ended_widget(series_information, &mut series_data_grid);

    let rating_widget = rating_widget(series_information);
    let summary = summary_widget(series_information);

    let series_name = text(series_information.name.clone())
        .size(31)
        .style(styles::text_styles::accent_color_theme());

    let title_bar = row![
        series_name.width(Length::FillPortion(10)),
        tracking_button(series_information.id)
    ];

    let next_episode_widget = next_episode_release_time_widget(next_episode_release_time);

    let rating_and_release_widget = row![
        rating_widget,
        horizontal_space(Length::Fill),
        next_episode_widget
    ]
    .padding(3);

    let series_data = column![
        title_bar,
        rating_and_release_widget,
        horizontal_rule(1),
        series_data_grid,
        vertical_space(10),
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
    .style(styles::container_styles::first_class_container_square_theme());

    container(content)
        .width(Length::Fill)
        .padding(10)
        .center_x()
        .into()
}

fn background(
    background_bytes: Option<Bytes>,
    series_image_blurred: Option<image::DynamicImage>,
) -> Element<'static, Message, Renderer> {
    if let Some(image_bytes) = background_bytes {
        let image_handle = iced::widget::image::Handle::from_memory(image_bytes);
        iced::widget::image(image_handle)
            .width(Length::Fill)
            .height(300)
            .content_fit(iced::ContentFit::Cover)
            .into()
    } else {
        // using the blurred series image when the background is not yet present(or still loading)
        if let Some(image) = series_image_blurred {
            let image_handle = iced::widget::image::Handle::from_pixels(
                image.width(),
                image.height(),
                image.into_rgba8().into_vec(),
            );
            return iced::widget::image(image_handle)
                .width(Length::Fill)
                .height(300)
                .content_fit(iced::ContentFit::Cover)
                .into();
        }
        Space::new(0, 300).into()
    }
}

fn tracking_button(series_id: u32) -> Button<'static, Message, Renderer> {
    if database::DB
        .get_series(series_id)
        .map(|series| series.is_tracked())
        .unwrap_or(false)
    {
        let tracked_icon_handle = svg::Handle::from_memory(PATCH_PLUS_FILL);
        let icon = svg(tracked_icon_handle)
            .width(30)
            .height(30)
            .style(styles::svg_styles::colored_svg_theme());
        button(icon).on_press(Message::UntrackSeries)
    } else {
        let tracked_icon_handle = svg::Handle::from_memory(PATCH_PLUS);
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
    SeriesImageLoaded(Option<Bytes>),
    SeriesBackgroundLoaded(Option<Bytes>),
    Seasons(SeasonsMessage),
    CastWidgetAction(CastWidgetMessage),
    SeriesSuggestion(SeriesSuggestionMessage),
    PageScrolled(Viewport),
    TrackSeries,
    UntrackSeries,
}

pub struct Series<'a> {
    series_id: u32,
    series_information: SeriesMainInformation,
    series_image: Option<Bytes>,
    series_image_blurred: Option<image::DynamicImage>,
    series_background: Option<Bytes>,
    seasons: Seasons,
    casts_widget: CastWidget,
    series_suggestion_widget: SeriesSuggestion<'a>,
    scroll_offset: RelativeOffset,
    scroller_id: Id,
}

impl<'a> Series<'a> {
    /// Counstruct the series page by providing it with SeriesMainInformation
    pub fn new(
        series_information: SeriesMainInformation,
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Command<Message>) {
        let series_id = series_information.id;
        let (casts_widget, casts_widget_command) = CastWidget::new(series_id);
        let (seasons, seasons_command) = Seasons::new(series_id, series_information.name.clone());

        let (series_suggestion_widget, series_suggestion_widget_command) = SeriesSuggestion::new(
            series_id,
            series_information.get_genres(),
            series_page_sender,
        );
        let scroller_id = Id::new(format!("series-page-scroller-{}", series_id));

        let series_image = series_information.image.clone();
        let series = Self {
            series_id,
            series_information,
            series_image: None,
            series_image_blurred: None,
            series_background: None,
            seasons,
            casts_widget,
            series_suggestion_widget,
            scroll_offset: RelativeOffset::default(),
            scroller_id: scroller_id.clone(),
        };

        let scroller_command = scrollable::snap_to(scroller_id, RelativeOffset::START);

        let commands = [
            Command::batch(load_images(series_image, series_id)),
            seasons_command.map(Message::Seasons),
            casts_widget_command.map(Message::CastWidgetAction),
            series_suggestion_widget_command.map(Message::SeriesSuggestion),
            scroller_command,
        ];

        (series, Command::batch(commands))
    }

    /// Restores the last `RelativeOffset` of the series page scroller.
    pub fn restore_scroller_relative_offset(&self) -> Command<Message> {
        scrollable::snap_to(self.scroller_id.clone(), self.scroll_offset)
    }

    /// Sets the `RelativeOffset` of the series page scroller to the start.
    pub fn set_relative_offset_to_start(&self) -> Command<Message> {
        scrollable::snap_to(self.scroller_id.clone(), RelativeOffset::START)
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SeriesImageLoaded(image) => {
                // This blurred series image is going to be used when the background is loading or missing
                if self.series_background.is_none() {
                    self.series_image_blurred = image.as_ref().map(|image| {
                        image::load_from_memory(image)
                            .unwrap()
                            /*
                            creating a thumbnail out of it as this is going to make blurring
                            process more faster
                            */
                            .thumbnail(100, 100)
                            .blur(5.0)
                    });
                }
                self.series_image = image;
            }
            Message::Seasons(message) => return self.seasons.update(message).map(Message::Seasons),
            Message::TrackSeries => {
                let series_id = self.series_information.id;

                if let Some(mut series) = database::DB.get_series(series_id) {
                    series.mark_tracked();
                } else {
                    let mut series = database::Series::new(
                        self.series_information.name.to_owned(),
                        self.series_id,
                    );
                    series.mark_tracked();
                    database::DB.add_series(self.series_information.id, &series);
                }
            }
            Message::UntrackSeries => {
                let series_id = self.series_information.id;
                if let Some(mut series) = database::DB.get_series(series_id) {
                    series.mark_untracked();
                }
            }
            Message::CastWidgetAction(message) => {
                return self
                    .casts_widget
                    .update(message)
                    .map(Message::CastWidgetAction)
            }
            Message::SeriesBackgroundLoaded(background) => self.series_background = background,
            Message::SeriesSuggestion(message) => {
                return self
                    .series_suggestion_widget
                    .update(message)
                    .map(Message::SeriesSuggestion)
            }
            Message::PageScrolled(view_port) => {
                self.scroll_offset = view_port.relative_offset();
            }
        }
        Command::none()
    }

    pub fn view(&self) -> Element<Message, Renderer> {
        let background = background(
            self.series_background.clone(),
            self.series_image_blurred.clone(),
        );

        let series_metadata = series_metadata(
            &self.series_information,
            self.series_image.clone(),
            self.seasons.get_next_episode_and_release_time(),
        );

        let seasons_widget = self.seasons.view().map(Message::Seasons);

        let casts_widget = self.casts_widget.view().map(Message::CastWidgetAction);
        let series_suggestion_widget = self
            .series_suggestion_widget
            .view()
            .map(Message::SeriesSuggestion);

        let content = column![
            background,
            series_metadata,
            vertical_space(10),
            seasons_widget,
            casts_widget,
            series_suggestion_widget
        ];

        scrollable(content)
            .direction(styles::scrollable_styles::vertical_direction())
            .id(self.scroller_id.clone())
            .on_scroll(Message::PageScrolled)
            .into()
    }
}

/// Returns two commands that requests series' image and seasons list
fn load_images(series_info_image: Option<Image>, series_id: u32) -> [Command<Message>; 2] {
    let image_command = if let Some(image_url) = series_info_image {
        Command::perform(
            caching::load_image(
                image_url.original_image_url,
                caching::ImageType::Original(caching::OriginalType::Poster),
            ),
            Message::SeriesImageLoaded,
        )
    } else {
        Command::none()
    };

    let background_command = Command::perform(
        caching::show_images::get_recent_banner(series_id),
        Message::SeriesBackgroundLoaded,
    );

    [image_command, background_command]
}
