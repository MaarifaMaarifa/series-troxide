use std::sync::mpsc;

use iced::widget::{column, container, scrollable, text, text_input, vertical_space, Column};
use iced::{Command, Element, Length, Renderer};
use iced_aw::Spinner;
use search_result::{
    IndexedMessage as SearchResultIndexedMessage, Message as SearchResultMessage, SearchResult,
};

use crate::core::api::tv_maze::series_information::SeriesMainInformation;
use crate::core::api::tv_maze::series_searching;
use crate::gui::styles;

#[derive(Default)]
pub enum LoadState {
    Loaded,
    Loading,
    #[default]
    NotLoaded,
}

#[derive(Clone, Debug)]
pub enum Message {
    TermChanged(String),
    TermSearched,
    SearchSuccess(Vec<series_searching::SeriesSearchResult>),
    SearchFail,
    SearchResult(SearchResultIndexedMessage<SearchResultMessage>),
    EscapeKeyPressed,
}

pub struct Search {
    search_term: String,
    search_results: Vec<SearchResult>,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
    pub load_state: LoadState,
}

impl Search {
    pub fn new(series_page_sender: mpsc::Sender<SeriesMainInformation>) -> Self {
        Self {
            search_term: String::new(),
            search_results: vec![],
            load_state: LoadState::NotLoaded,
            series_page_sender,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::subscription::events_with(|event, _| {
            if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key_code,
                modifiers,
            }) = event
            {
                if key_code == iced::keyboard::KeyCode::Escape && modifiers.is_empty() {
                    return Some(Message::EscapeKeyPressed);
                }
            }
            None
        })
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::TermChanged(term) => {
                self.search_term = term;
                self.load_state = LoadState::NotLoaded;
            }
            Message::TermSearched => {
                self.load_state = LoadState::Loading;

                let series_result = series_searching::search_series(self.search_term.clone());

                return Command::perform(series_result, |res| match res {
                    Ok(res) => Message::SearchSuccess(res),
                    Err(_) => Message::SearchFail,
                });
            }
            Message::SearchSuccess(results) => {
                self.load_state = LoadState::Loaded;

                let mut search_results = Vec::with_capacity(results.len());
                let mut search_results_commands = Vec::with_capacity(results.len());
                results.into_iter().enumerate().for_each(|(index, result)| {
                    let (search_result, search_result_command) =
                        SearchResult::new(index, result, self.series_page_sender.clone());
                    search_results.push(search_result);
                    search_results_commands.push(search_result_command.map(Message::SearchResult));
                });

                self.search_results = search_results;

                return Command::batch(search_results_commands);
            }
            Message::SearchFail => panic!("Series Search Failed"),
            Message::SearchResult(message) => {
                if let SearchResultMessage::SeriesResultPressed = message.clone().message() {
                    self.load_state = LoadState::NotLoaded;
                }
                self.search_results[message.index()].update(message)
            }
            Message::EscapeKeyPressed => self.load_state = LoadState::NotLoaded,
        }
        Command::none()
    }

    pub fn view(
        &self,
    ) -> (
        Element<'_, Message, Renderer>,
        Option<Element<'_, Message, Renderer>>,
    ) {
        let search_bar = column!(
            vertical_space(10),
            text_input("Search Series", &self.search_term)
                .width(300)
                .on_input(Message::TermChanged)
                .on_submit(Message::TermSearched)
        )
        .width(Length::Fill)
        .align_items(iced::Alignment::Center);

        let search_results: Option<Element<'_, Message, Renderer>> = match self.load_state {
            LoadState::Loaded => {
                let result_items: Vec<_> = self
                    .search_results
                    .iter()
                    .map(|result| result.view().map(Message::SearchResult))
                    .collect();

                Some(if result_items.is_empty() {
                    container(text("No results"))
                        .padding(10)
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .center_x()
                        .center_y()
                        .into()
                } else {
                    Column::with_children(result_items)
                        .padding(20)
                        .spacing(5)
                        .into()
                })
            }
            LoadState::Loading => Some(Spinner::new().into()),
            LoadState::NotLoaded => None,
        };

        let search_results = search_results.map(|search_results| {
            container(
                scrollable(search_results)
                    .direction(styles::scrollable_styles::vertical_direction()),
            )
            .padding(5)
            .style(styles::container_styles::first_class_container_rounded_theme())
            .into()
        });

        (search_bar.into(), search_results)
    }
}

mod search_result {
    use std::sync::mpsc;

    use bytes::Bytes;
    use iced::widget::{column, image, mouse_area, row, svg, text, Space};
    use iced::{Command, Element, Renderer};

    use crate::core::api::tv_maze::series_information::SeriesMainInformation;
    use crate::core::api::tv_maze::Rating;
    use crate::core::{api::tv_maze::series_searching, caching};
    use crate::gui::assets::icons::STAR_FILL;
    pub use crate::gui::message::IndexedMessage;
    use crate::gui::{helpers, styles};

    #[derive(Debug, Clone)]
    pub enum Message {
        ImageLoaded(Option<Bytes>),
        SeriesResultPressed,
    }

    pub struct SearchResult {
        index: usize,
        search_result: series_searching::SeriesSearchResult,
        image: Option<Bytes>,
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
    }

    impl SearchResult {
        pub fn new(
            index: usize,
            search_result: series_searching::SeriesSearchResult,
            series_page_sender: mpsc::Sender<SeriesMainInformation>,
        ) -> (Self, Command<IndexedMessage<Message>>) {
            let image_url = search_result.show.image.clone();
            (
                Self {
                    index,
                    search_result,
                    image: None,
                    series_page_sender,
                },
                image_url
                    .map(|url| {
                        Command::perform(
                            caching::load_image(url.medium_image_url, caching::ImageType::Medium),
                            Message::ImageLoaded,
                        )
                        .map(move |message| IndexedMessage::new(index, message))
                    })
                    .unwrap_or(Command::none()),
            )
        }

        pub fn update(&mut self, message: IndexedMessage<Message>) {
            match message.message() {
                Message::ImageLoaded(image) => self.image = image,
                Message::SeriesResultPressed => {
                    self.series_page_sender
                        .send(self.search_result.show.clone())
                        .expect("failed to send series page info");
                }
            }
        }

        pub fn view(&self) -> Element<'_, IndexedMessage<Message>, Renderer> {
            let mut row = row!().spacing(5).padding(5);

            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                row = row.push(image(image_handle).height(60))
            } else {
                row = row.push(Space::new(43, 60))
            };

            // Getting the series genres
            let genres: Element<'_, Message, Renderer> =
                if !self.search_result.show.genres.is_empty() {
                    text(helpers::genres_with_pipes(&self.search_result.show.genres))
                        .size(11)
                        .into()
                } else {
                    Space::new(0, 0).into()
                };

            let mut column = column![
                text(&self.search_result.show.name)
                    .size(16)
                    .style(styles::text_styles::accent_color_theme()),
                genres
            ];

            if let Some(premier) = &self.search_result.show.premiered {
                column = column.push(text(format!("Premiered: {}", premier)).size(9));
            }

            column = column.push(Self::rating_widget(&self.search_result.show.rating));

            let element: Element<'_, Message, Renderer> = mouse_area(row.push(column))
                .on_press(Message::SeriesResultPressed)
                .into();
            element.map(|message| IndexedMessage::new(self.index, message))
        }

        fn rating_widget(rating: &Rating) -> Element<'_, Message, Renderer> {
            if let Some(average_rating) = rating.average {
                let star_handle = svg::Handle::from_memory(STAR_FILL);
                let star_icon = svg(star_handle)
                    .width(12)
                    .height(12)
                    .style(styles::svg_styles::colored_svg_theme());

                row![star_icon, text(average_rating).size(11)]
                    .spacing(5)
                    .into()
            } else {
                Space::new(0, 0).into()
            }
        }
    }
}
