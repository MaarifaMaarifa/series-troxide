use std::sync::mpsc;

use bytes::Bytes;
use image;

use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::api::tv_maze::Image;
use crate::core::program_state::ProgramState;
use crate::core::{caching, database};
use crate::gui::styles;
use data_widgets::*;
use people_widget::{Message as PeopleWidgetMessage, PeopleWidget};
use season_widget::{Message as SeasonsMessage, Seasons};
use series_suggestion_widget::{Message as SeriesSuggestionMessage, SeriesSuggestion};

use iced::widget::scrollable::{Id, RelativeOffset, Viewport};
use iced::widget::Space;
use iced::widget::{column, scrollable};
use iced::{Element, Task};

mod data_widgets;
mod people_widget;
mod season_widget;
mod series_suggestion_widget;

#[derive(Clone, Debug)]
pub enum Message {
    SeriesImageLoaded(Option<Bytes>),
    SeriesBackgroundLoaded(Option<Bytes>),
    Seasons(SeasonsMessage),
    PeopleWidget(PeopleWidgetMessage),
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
    people_widget: PeopleWidget,
    series_suggestion_widget: SeriesSuggestion<'a>,
    scroll_offset: RelativeOffset,
    scroller_id: Id,
    program_state: ProgramState,
}

impl<'a> Series<'a> {
    /// Counstruct the series page by providing it with SeriesMainInformation
    pub fn new(
        program_state: ProgramState,
        series_information: SeriesMainInformation,
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    ) -> (Self, Task<Message>) {
        let series_id = series_information.id;
        let (people_widget, people_widget_command) = PeopleWidget::new(series_id);
        let (seasons, seasons_command) = Seasons::new(
            program_state.clone(),
            series_id,
            series_information.name.clone(),
        );

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
            people_widget,
            series_suggestion_widget,
            scroll_offset: RelativeOffset::default(),
            scroller_id: scroller_id.clone(),
            program_state,
        };

        let scroller_command = scrollable::snap_to(scroller_id, RelativeOffset::START);

        let commands = [
            load_images(series_image, series_id),
            seasons_command.map(Message::Seasons),
            people_widget_command.map(Message::PeopleWidget),
            series_suggestion_widget_command.map(Message::SeriesSuggestion),
            scroller_command,
        ];

        (series, Task::batch(commands))
    }

    pub fn get_series_main_information(&self) -> &SeriesMainInformation {
        &self.series_information
    }

    /// Restores the last `RelativeOffset` of the series page scroller.
    pub fn restore_scroller_relative_offset(&self) -> Task<Message> {
        scrollable::snap_to(self.scroller_id.clone(), self.scroll_offset)
    }

    /// Sets the `RelativeOffset` of the series page scroller to the start.
    pub fn set_relative_offset_to_start(&self) -> Task<Message> {
        scrollable::snap_to(self.scroller_id.clone(), RelativeOffset::START)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
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

                if let Some(mut series) =
                    database::series_tree::get_series(self.program_state.get_db(), series_id)
                {
                    series.mark_tracked();

                    database::series_tree::add_series(
                        self.program_state.get_db(),
                        self.series_information.id,
                        &series,
                    );
                } else {
                    let mut series = database::db_models::Series::new(
                        self.series_information.name.to_owned(),
                        self.series_id,
                    );
                    series.mark_tracked();
                    database::series_tree::add_series(
                        self.program_state.get_db(),
                        self.series_information.id,
                        &series,
                    );
                }
            }
            Message::UntrackSeries => {
                let series_id = self.series_information.id;
                if let Some(mut series) =
                    database::series_tree::get_series(self.program_state.get_db(), series_id)
                {
                    series.mark_untracked();
                    database::series_tree::add_series(
                        self.program_state.get_db(),
                        series_id,
                        &series,
                    );
                }
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
            Message::PeopleWidget(message) => {
                return self
                    .people_widget
                    .update(message)
                    .map(Message::PeopleWidget)
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let background = background(
            self.series_background.clone(),
            self.series_image_blurred.clone(),
        );

        let series_metadata = series_metadata(
            self.program_state.get_db(),
            &self.series_information,
            self.series_image.clone(),
            self.seasons.get_next_episode_to_air(),
        );

        let seasons_widget = self.seasons.view().map(Message::Seasons);

        let people_widget = self.people_widget.view().map(Message::PeopleWidget);

        let series_suggestion_widget = self
            .series_suggestion_widget
            .view()
            .map(Message::SeriesSuggestion);

        let content = column![
            background,
            series_metadata,
            Space::with_height(10),
            seasons_widget,
            people_widget,
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
fn load_images(series_info_image: Option<Image>, series_id: u32) -> Task<Message> {
    let image_command = if let Some(image_url) = series_info_image {
        Task::perform(
            caching::load_image(
                image_url.original_image_url,
                caching::ImageResolution::Original(caching::ImageKind::Poster),
            ),
            Message::SeriesImageLoaded,
        )
    } else {
        Task::none()
    };

    let background_command = Task::perform(
        caching::show_images::get_recent_banner(series_id),
        Message::SeriesBackgroundLoaded,
    );

    Task::batch([image_command, background_command])
}
