use std::sync::mpsc;

use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::program_state::ProgramState;
use crate::gui::assets::icons::FILM;
use crate::gui::styles;

use iced::widget::scrollable::{RelativeOffset, Viewport};
use iced::widget::{column, scrollable, text};
use iced::{Element, Length, Task};

use my_shows_widget::{Message as MyShowsMessage, MyShows};
use upcoming_releases_widget::{Message as UpcomingReleasesMessage, UpcomingReleases};

use super::tab_searching::{Message as SearcherMessage, Searchable, Searcher};
use super::Tab;

mod my_shows_widget;
mod upcoming_releases_widget;

#[derive(Debug, Clone)]
pub enum Message {
    Ended(MyShowsMessage),
    Waiting(MyShowsMessage),
    Upcoming(UpcomingReleasesMessage),
    Untracked(MyShowsMessage),
    PageScrolled(Viewport),
    Searcher(SearcherMessage),
}

pub struct MyShowsTab<'a> {
    waiting_releases: MyShows<'a>,
    upcoming_releases: UpcomingReleases<'a>,
    ended_releases: MyShows<'a>,
    untracked_releases: MyShows<'a>,
    scrollable_offset: RelativeOffset,
    searcher: Searcher,
}

impl<'a> MyShowsTab<'a> {
    pub fn new(
        program_state: ProgramState,
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
        scrollable_offset: Option<RelativeOffset>,
    ) -> (Self, Task<Message>) {
        let (untracked_releases, untracked_releases_commands) =
            MyShows::new_as_untracked_series(program_state.clone(), series_page_sender.clone());
        let (ended_releases, ended_releases_commands) =
            MyShows::new_as_ended_tracked_series(program_state.clone(), series_page_sender.clone());
        let (upcoming_releases, upcoming_releases_commands) =
            UpcomingReleases::new(program_state.clone(), series_page_sender.clone());
        let (waiting_releases, waiting_releases_commands) =
            MyShows::new_as_waiting_release_series(program_state.clone(), series_page_sender);

        (
            Self {
                ended_releases,
                untracked_releases,
                waiting_releases,
                upcoming_releases,
                scrollable_offset: scrollable_offset.unwrap_or(RelativeOffset::START),
                searcher: Searcher::new("Search My Shows".to_owned()),
            },
            Task::batch([
                untracked_releases_commands.map(Message::Untracked),
                ended_releases_commands.map(Message::Ended),
                waiting_releases_commands.map(Message::Waiting),
                upcoming_releases_commands.map(Message::Upcoming),
            ]),
        )
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        self.upcoming_releases.subscription().map(Message::Upcoming)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Ended(message) => self.ended_releases.update(message).map(Message::Ended),
            Message::Waiting(message) => {
                self.waiting_releases.update(message).map(Message::Waiting)
            }
            Message::Upcoming(message) => self
                .upcoming_releases
                .update(message)
                .map(Message::Upcoming),
            Message::Untracked(message) => self
                .untracked_releases
                .update(message)
                .map(Message::Untracked),
            Message::PageScrolled(view_port) => {
                self.scrollable_offset = view_port.relative_offset();
                Task::none()
            }
            Message::Searcher(message) => {
                self.searcher.update(message);
                let current_search_term = self.searcher.current_search_term().to_owned();

                self.waiting_releases.update_matches(&current_search_term);
                self.upcoming_releases.update_matches(&current_search_term);
                self.ended_releases.update_matches(&current_search_term);
                self.untracked_releases.update_matches(&current_search_term);

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let upcoming_releases = self.upcoming_releases.view().map(Message::Upcoming);

        let waiting_releases: Element<'_, Message> = column![
            text("Waiting for release date")
                .size(21)
                .style(styles::text_styles::green_text_theme),
            self.waiting_releases.view().map(Message::Waiting)
        ]
        .spacing(5)
        .into();

        let ended_releases: Element<'_, Message> = column![
            text("Ended")
                .size(21)
                .style(styles::text_styles::red_text_theme),
            self.ended_releases.view().map(Message::Ended)
        ]
        .spacing(5)
        .into();

        let untracked_releases: Element<'_, Message> = column![
            text("Untracked").size(21),
            self.untracked_releases.view().map(Message::Untracked)
        ]
        .spacing(5)
        .into();

        let searcher = self.searcher.view().map(Message::Searcher);

        let content = scrollable(
            column![
                upcoming_releases,
                waiting_releases,
                ended_releases,
                untracked_releases,
            ]
            .padding(10)
            .spacing(50)
            .width(Length::Fill)
            .align_x(iced::Alignment::Start),
        )
        .direction(styles::scrollable_styles::vertical_direction())
        .id(Self::scrollable_id())
        .on_scroll(Message::PageScrolled);

        column![searcher, content].spacing(10).padding(5).into()
    }
}

impl<'a> Tab for MyShowsTab<'a> {
    type Message = Message;

    fn title() -> &'static str {
        "My Shows"
    }

    fn icon_bytes() -> &'static [u8] {
        FILM
    }

    fn get_scrollable_offset(&self) -> scrollable::RelativeOffset {
        self.scrollable_offset
    }
}
