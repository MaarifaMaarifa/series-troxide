use api::series_information;
use api::series_searching;
use iced::widget::{column, container, mouse_area, row, text, text_input, Column};
use iced::{Application, Command, Settings};

mod api;
mod cli;
mod database;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // simple_logger::init()?;
    Gui::run(Settings::default())?;
    Ok(())
}

#[derive(Debug, Clone)]
enum Message {
    SearchTermChanged(String),
    SearchTheTerm,
    SeriesResultPressed(u32),
    SeriesResultObtained(series_information::SeriesMainInformation),
    SeriesResultFailed,
    SeriesResultsObtained(Vec<series_searching::SeriesSearchResult>),
    SeriesResultsFailed,
}

#[derive(Default)]
enum Page {
    #[default]
    Search,
    Series,
    season,
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
    series_information: series_information::SeriesMainInformation,
}

#[derive(Default)]
struct Gui {
    search_term: String,
    series_result: Vec<series_searching::SeriesSearchResult>,
    search_state: SearchState,
    series_page_data: Option<SeriesPageData>,
    page: Page,
}

impl Application for Gui {
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
                        println!("Error obtaining series information: {}", err);
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
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        match &self.page {
            Page::Search => {
                let text_input = text_input("Search Series", &self.search_term)
                    .on_input(|term| Message::SearchTermChanged(term))
                    .on_submit(Message::SearchTheTerm);

                let series_results = {
                    match self.search_state {
                        SearchState::Searching => column!(text("Searching...")),
                        SearchState::Complete => {
                            let mut results = Column::new();

                            for series in &self.series_result {
                                let mut row = row!(
                                    text(&series.show.name),
                                    text(format!("{:?}", &series.show.genres)),
                                );

                                if let Some(premier) = &series.show.premiered {
                                    row = row.push(text(premier));
                                }

                                let row = mouse_area(row)
                                    .on_press(Message::SeriesResultPressed(series.show.id));

                                results = results.push(row);
                            }
                            results
                        }
                    }
                };

                let content = column!(text_input, series_results);
                container(content).into()
            }
            Page::Series => {
                let series_information =
                    &self.series_page_data.as_ref().unwrap().series_information;
                let title = text(&series_information.name);
                let summary = text(&series_information.summary);

                column!(title, summary).into()
            }
            Page::season => todo!(),
            Page::Episode => todo!(),
        }
    }
}
