use bytes::Bytes;
use iced::widget::{
    column, container, scrollable, text, text_input, vertical_space, Column, Space,
};
use iced::{Command, Element, Length, Renderer};
use iced_aw::Spinner;
use search_result::{Message as SearchResultMessage, SearchResult};

use super::Message as DiscoverMessage;
use crate::core::api::series_searching;
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
    SearchTermChanged(String),
    SearchTermSearched,
    SearchSuccess(Vec<series_searching::SeriesSearchResult>),
    SearchFail,
    SeriesResultPressed(/*series id*/ u32),
    SearchResult(SearchResultMessage),
}

#[derive(Default)]
pub struct Search {
    search_term: String,
    search_results: Vec<SearchResult>,
    series_search_results_images: Vec<Option<Bytes>>,
    pub load_state: LoadState,
}

impl Search {
    pub fn update(&mut self, message: Message) -> Command<DiscoverMessage> {
        match message {
            Message::SearchTermChanged(term) => {
                self.search_term = term;
                return if self.search_term.is_empty() {
                    self.load_state = LoadState::NotLoaded;
                    Command::perform(async {}, |_| DiscoverMessage::HideSearchResults)
                } else {
                    Command::none()
                };
            }
            Message::SearchTermSearched => {
                self.load_state = LoadState::Loading;

                let series_result = series_searching::search_series(self.search_term.clone());

                let search_status_command = Command::perform(series_result, |res| match res {
                    Ok(res) => DiscoverMessage::Search(Message::SearchSuccess(res)),
                    Err(_) => DiscoverMessage::Search(Message::SearchFail),
                });

                let show_overlay_command =
                    Command::perform(async {}, |_| DiscoverMessage::ShowSearchResults);

                return Command::batch([search_status_command, show_overlay_command]);
            }
            Message::SearchSuccess(results) => {
                self.load_state = LoadState::Loaded;
                self.series_search_results_images.clear();
                let show_overlay_command =
                    Command::perform(async {}, |_| DiscoverMessage::ShowSearchResults);

                let mut search_results = Vec::with_capacity(results.len());
                let mut search_results_commands = Vec::with_capacity(results.len());
                results.into_iter().enumerate().for_each(|(index, result)| {
                    let (search_result, search_result_command) = SearchResult::new(index, result);
                    search_results.push(search_result);
                    search_results_commands
                        .push(search_result_command.map(|message| {
                            DiscoverMessage::Search(Message::SearchResult(message))
                        }));
                });

                self.search_results = search_results;

                return Command::batch([
                    Command::batch(search_results_commands),
                    show_overlay_command,
                ]);
            }
            Message::SearchFail => panic!("Series Search Failed"),
            Message::SeriesResultPressed(_) => {
                unreachable!("Search page should not handle series page result")
            }
            Message::SearchResult(message) => {
                if let SearchResultMessage::SeriesResultPressed(series_id) = message {
                    return Command::perform(async {}, move |_| {
                        DiscoverMessage::Search(Message::SeriesResultPressed(series_id))
                    });
                }
                self.search_results[message.get_id().unwrap_or(0)].update(message)
            }
        }
        Command::none()
    }

    pub fn view(
        &self,
    ) -> (
        Element<'_, Message, Renderer>,
        Element<'_, Message, Renderer>,
    ) {
        let search_bar = column!(
            vertical_space(10),
            text_input("Search Series", &self.search_term)
                .width(300)
                .on_input(Message::SearchTermChanged)
                .on_submit(Message::SearchTermSearched)
        )
        .width(Length::Fill)
        .align_items(iced::Alignment::Center);

        let search_results: Element<'_, Message, Renderer> = match self.load_state {
            LoadState::Loaded => {
                let result_items: Vec<_> = self
                    .search_results
                    .iter()
                    .map(|result| result.view().map(Message::SearchResult))
                    .collect();

                if result_items.is_empty() {
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
                }
            }
            LoadState::Loading => Spinner::new().into(),
            LoadState::NotLoaded => Space::new(0, 0).into(),
        };

        let search_results = container(search_results)
            .style(styles::container_styles::first_class_container_rounded_theme());

        (search_bar.into(), scrollable(search_results).into())
    }
}

mod search_result {
    use bytes::Bytes;
    use iced::widget::{column, image, mouse_area, row, text, Space};
    use iced::{Command, Element, Renderer};

    use crate::core::{api::series_searching, caching};
    use crate::gui::styles;

    #[derive(Debug, Clone)]
    pub enum Message {
        ImageLoaded(usize, Option<Bytes>),
        SeriesResultPressed(u32),
    }

    impl Message {
        pub fn get_id(&self) -> Option<usize> {
            match self {
                Message::ImageLoaded(id, _) => Some(*id),
                Message::SeriesResultPressed(_) => None,
            }
        }
    }

    pub struct SearchResult {
        search_result: series_searching::SeriesSearchResult,
        image: Option<Bytes>,
    }

    impl SearchResult {
        pub fn new(
            id: usize,
            search_result: series_searching::SeriesSearchResult,
        ) -> (Self, Command<Message>) {
            let image_url = search_result.show.image.clone();
            (
                Self {
                    search_result,
                    image: None,
                },
                image_url
                    .map(|url| {
                        Command::perform(caching::load_image(url.medium_image_url), move |image| {
                            Message::ImageLoaded(id, image)
                        })
                    })
                    .unwrap_or(Command::none()),
            )
        }

        pub fn update(&mut self, message: Message) {
            match message {
                Message::ImageLoaded(_, image) => self.image = image,
                Message::SeriesResultPressed(_) => {
                    unreachable!("search result widget shouldn't handle being pressed")
                }
            }
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
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
                    let mut genres = String::from("Genres: ");

                    let mut series_result_iter = self.search_result.show.genres.iter().peekable();
                    while let Some(genre) = series_result_iter.next() {
                        genres.push_str(genre);
                        if series_result_iter.peek().is_some() {
                            genres.push_str(", ");
                        }
                    }
                    text(genres).size(11).into()
                } else {
                    Space::new(0, 0).into()
                };

            let mut column = column![
                text(&self.search_result.show.name)
                    .size(16)
                    .style(styles::text_styles::purple_text_theme()),
                genres
            ];

            if let Some(premier) = &self.search_result.show.premiered {
                column = column.push(text(format!("Premiered: {}", premier)).size(9));
            }

            mouse_area(row.push(column))
                .on_press(Message::SeriesResultPressed(self.search_result.show.id))
                .into()
        }
    }
}
