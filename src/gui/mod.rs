mod troxide_widget;
mod view;

use crate::core::api::series_information;
use crate::core::api::series_searching;

use view::menu_view::Message as MenuMessage;

use iced::widget::row;
use iced::widget::{
    column, container, mouse_area, scrollable, text, text_input, vertical_space, Column,
};
use iced::Alignment;
use iced::{Application, Command, Length};

#[derive(Debug, Clone)]
pub enum Message {
    SearchTermChanged(String),
    SearchTheTerm,
    SeriesResultPressed(u32),
    SeriesResultObtained((series_information::SeriesMainInformation, Option<Vec<u8>>)),
    SeriesResultFailed,
    SeriesResultsObtained(Vec<(series_searching::SeriesSearchResult, Option<Vec<u8>>)>),
    SeriesResultsFailed,
    TrackSeries,
    GoToSearchPage,
    MenuAction(MenuMessage),
}

#[derive(Default)]
enum Page {
    #[default]
    Search,
    Series,
    Season,
    Episode,
}

#[derive(Default)]
enum SearchState {
    Searching,
    #[default]
    Complete,
}

#[derive(Debug)]
struct SeriesPageData {
    series_information: (series_information::SeriesMainInformation, Option<Vec<u8>>),
}

#[derive(Default)]
pub struct TroxideGui {
    menu_view: view::menu_view::Menu,
    search_term: String,
    series_result: Vec<(series_searching::SeriesSearchResult, Option<Vec<u8>>)>,
    search_state: SearchState,
    series_page_data: Option<SeriesPageData>,
    page: Page,
}

impl Application for TroxideGui {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        "Series Troxide".to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::SearchTermChanged(search_term) => {
                if let SearchState::Complete = self.search_state {
                    self.search_term = search_term;
                }
                Command::none()
            }
            Message::SearchTheTerm => {
                self.search_state = SearchState::Searching;

                let series_result = series_searching::search_series(self.search_term.clone());

                Command::perform(series_result, |res| match res {
                    Ok(res) => Message::SeriesResultsObtained(res),
                    Err(err) => {
                        println!("{:?}", err);
                        Message::SeriesResultsFailed
                    }
                })
            }
            Message::SeriesResultsObtained(series_results) => {
                self.series_result = series_results;
                self.search_state = SearchState::Complete;
                Command::none()
            }
            Message::SeriesResultsFailed => {
                // log::error!("Failed to obtain series search results");
                println!("Failed to obtain series search results");
                self.search_state = SearchState::Complete;
                Command::none()
            }
            Message::SeriesResultPressed(series_id) => {
                let series_information = series_information::get_series_main_info(series_id);

                Command::perform(series_information, |res| match res {
                    Ok(res) => Message::SeriesResultObtained(res),
                    Err(err) => {
                        println!("Error obtaining series information: {:?}", err);
                        Message::SeriesResultFailed
                    }
                })
            }
            Message::SeriesResultObtained(series_information) => {
                self.series_page_data = Some(SeriesPageData { series_information });
                self.page = Page::Series;
                Command::none()
            }
            Message::SeriesResultFailed => {
                println!("Failed to obtain Series Information");
                Command::none()
            }
            Message::TrackSeries => {
                println!("Added series to tracking");
                Command::none()
            }
            Message::GoToSearchPage => {
                self.page = Page::Search;
                Command::none()
            }
            Message::MenuAction(message) => {
                self.menu_view.update(message);
                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let main_view = match &self.page {
            Page::Search => {
                let text_input = text_input("Search Series", &self.search_term)
                    .on_input(|term| Message::SearchTermChanged(term))
                    .on_submit(Message::SearchTheTerm)
                    .padding(5)
                    .width(500);

                let series_results = {
                    match self.search_state {
                        SearchState::Searching => column!(text("Searching...")),
                        SearchState::Complete => {
                            let mut results = Column::new();

                            for (series, image_bytes) in &self.series_result {
                                let row =
                                    troxide_widget::series_result(series, image_bytes.to_owned());
                                let row = mouse_area(row)
                                    .on_press(Message::SeriesResultPressed(series.show.id));
                                results = results.push(row).push(vertical_space(7));
                            }
                            results
                        }
                    }
                };

                let content = column!(
                    vertical_space(15),
                    text_input,
                    vertical_space(10),
                    scrollable(series_results).width(Length::Fill)
                )
                .align_items(Alignment::Center);
                container(content)
            }
            Page::Series => {
                let series_information =
                    &self.series_page_data.as_ref().unwrap().series_information;

                view::series_view::series_page(
                    &series_information.0,
                    series_information.1.to_owned(),
                )
                // let title = text(&series_information.name);
                // let summary = text(&series_information.summary);

                // column!(title, summary).into()
            }
            Page::Season => todo!(),
            Page::Episode => todo!(),
        };

        let menu_view = self.menu_view.view().map(Message::MenuAction);

        row!(menu_view, main_view).into()
    }
}
