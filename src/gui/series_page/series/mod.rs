use std::sync::mpsc;

use crate::core::api::tv_maze::episodes_information::Episode;
use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::api::tv_maze::Image;
use crate::core::caching::episode_list::EpisodeReleaseTime;
use crate::core::{caching, database};
use crate::gui::assets::get_static_cow_from_asset;
use crate::gui::assets::icons::{PATCH_PLUS, PATCH_PLUS_FILL};
use crate::gui::styles;

use bytes::Bytes;
use cast_widget::{CastWidget, Message as CastWidgetMessage};
use data_widgets::*;
use iced::widget::scrollable::{Id, RelativeOffset, Viewport};
use image;
use season_widget::{IndexedMessage as SeasonIndexedMessage, Message as SeasonMessage};
use series_suggestion_widget::{Message as SeriesSuggestionMessage, SeriesSuggestion};

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, scrollable, text, Button,
    Space,
};
use iced::widget::{svg, vertical_space, Column};
use iced::{Alignment, Command, Element, Length, Renderer};
use iced_aw::{Grid, Spinner};

mod cast_widget;
mod data_widgets;
mod season_widget;
mod series_suggestion_widget;

/// Generates the Series Metadata
pub fn series_metadata<'a>(
    series_information: &'a SeriesMainInformation,
    image_bytes: Option<Bytes>,
    next_episode_release_time: Option<&'a (Episode, EpisodeReleaseTime)>,
) -> Element<'a, Message, Renderer> {
    let mut main_info = row!().padding(5).spacing(10);

    if let Some(image_bytes) = image_bytes {
        let image_handle = iced::widget::image::Handle::from_memory(image_bytes);
        let image = iced::widget::image(image_handle).width(180);

        main_info = main_info.push(image);
    } else {
        main_info = main_info.push(Space::new(180, 253));
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
        // next_episode_widget
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

/// This Series Message is useful to make sure that the appropiate
/// series page receives it's exact series message. Since the series
/// page can be switched rapidly by the user, some of the commands
/// might be running in the background and my complete when a new instance
/// of series page has been opened updating it with wrong data.
#[derive(Clone, Debug)]
pub struct IdentifiableMessage {
    pub id: u32,
    pub message: Message,
}

impl IdentifiableMessage {
    pub fn new(id: u32, message: Message) -> Self {
        Self { id, message }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_message(self) -> Message {
        self.message
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    SeriesImageLoaded(Option<Bytes>),
    SeriesBackgroundLoaded(Option<Bytes>),
    EpisodeListLoaded(caching::episode_list::EpisodeList),
    Season(SeasonIndexedMessage<SeasonMessage>),
    CastWidgetAction(CastWidgetMessage),
    SeriesSuggestion(SeriesSuggestionMessage),
    PageScrolled(Viewport),
    TrackSeries,
    UntrackSeries,
}

enum LoadState {
    Loading,
    Loaded,
}

pub struct Series {
    series_id: u32,
    seasons_load_state: LoadState,
    series_information: SeriesMainInformation,
    series_image: Option<Bytes>,
    series_image_blurred: Option<image::DynamicImage>,
    series_background: Option<Bytes>,
    next_episode_release_time: Option<(Episode, EpisodeReleaseTime)>,
    season_widgets: Vec<season_widget::Season>,
    casts_widget: CastWidget,
    series_suggestion_widget: SeriesSuggestion,
    scroll_offset: RelativeOffset,
    scroller_id: Id,
}

impl Series {
    /// Counstruct the series page by providing it with SeriesMainInformation
    pub fn new(
        series_information: SeriesMainInformation,
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Command<Message>) {
        let series_id = series_information.id;
        let (casts_widget, casts_widget_command) = CastWidget::new(series_id);
        let (series_suggestion_widget, series_suggestion_widget_command) = SeriesSuggestion::new(
            series_id,
            series_information.get_genres(),
            series_page_sender,
        );
        let scroller_id = Id::new(format!("series-page-scroller-{}", series_id));

        let series_image = series_information.image.clone();
        let series = Self {
            series_id,
            seasons_load_state: LoadState::Loading,
            series_information,
            next_episode_release_time: None,
            series_image: None,
            series_image_blurred: None,
            series_background: None,
            season_widgets: vec![],
            casts_widget,
            series_suggestion_widget,
            scroll_offset: RelativeOffset::default(),
            scroller_id: scroller_id.clone(),
        };

        let scroller_command = scrollable::snap_to(scroller_id, RelativeOffset::START);

        let commands = [
            Command::batch(get_images_and_episode_list(series_image, series_id)),
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
            Message::Season(message) => {
                return self.season_widgets[message.index()]
                    .update(message)
                    .map(Message::Season);
            }
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
                            self.series_information.clone().name,
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
            self.next_episode_release_time.as_ref(),
        );

        let seasons_widget = self.seasons_view();

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
            .id(self.scroller_id.clone())
            .on_scroll(Message::PageScrolled)
            .into()
    }

    fn seasons_view(&self) -> Element<'_, Message, Renderer> {
        let seasons_body = column![text("Seasons").size(21)]
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
                            .map(|widget| widget.view().map(Message::Season))
                            .collect(),
                    )
                    .padding(5)
                    .spacing(5)
                    .align_items(Alignment::Center),
                ),
            )
        }
        .padding(10)
        .style(styles::container_styles::first_class_container_rounded_theme());

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
