use iced::widget::{
    column, horizontal_space, image, row, scrollable, text, text_input, vertical_space,
};
use iced::{Command, Element, Length, Renderer};

use crate::core::api::series_searching;
use crate::gui::Message as GuiMessage;

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
    SearchSuccess(Vec<(series_searching::SeriesSearchResult, Option<Vec<u8>>)>),
    SearchFail,
}

#[derive(Default)]
pub struct Search {
    search_term: String,
    series_search_result: Vec<(series_searching::SeriesSearchResult, Option<Vec<u8>>)>,
    load_state: LoadState,
}

impl Search {
    pub fn update(&mut self, message: Message) -> Command<GuiMessage> {
        match message {
            Message::SearchTermChanged(term) => {
                self.search_term = term;
                Command::none()
            }
            Message::SearchTermSearched => {
                self.load_state = LoadState::Loading;

                let series_result = series_searching::search_series(self.search_term.clone());

                Command::perform(series_result, |res| match res {
                    Ok(res) => GuiMessage::SearchAction(Message::SearchSuccess(res)),
                    Err(err) => {
                        println!("{:?}", err);
                        GuiMessage::SearchAction(Message::SearchFail)
                    }
                })
            }
            Message::SearchSuccess(res) => {
                self.load_state = LoadState::Loaded;
                self.series_search_result = res;
                Command::none()
            }
            Message::SearchFail => panic!("Series Search Failed"),
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let search_bar = column!(
            vertical_space(10),
            text_input("Search Series", &self.search_term)
                .width(300)
                .on_input(|term| Message::SearchTermChanged(term))
                .on_submit(Message::SearchTermSearched)
        )
        .width(Length::Fill)
        .align_items(iced::Alignment::Center);

        let search_body = column!();

        let search_body = match self.load_state {
            LoadState::Loaded => search_body.push(load(&self.series_search_result)),
            LoadState::Loading => search_body.push(
                column!("Loading Search Results")
                    .width(Length::Fill)
                    .align_items(iced::Alignment::Center),
            ),
            LoadState::NotLoaded => search_body.push(
                column!("Nothing to show, waiting to search.")
                    .width(Length::Fill)
                    .align_items(iced::Alignment::Center),
            ),
        };

        column!(search_bar, scrollable(search_body).width(Length::Fill)).into()
    }
}

fn load(
    series_result: &Vec<(series_searching::SeriesSearchResult, Option<Vec<u8>>)>,
) -> Element<'_, Message, Renderer> {
    let mut results = column!();

    for (series, image_bytes) in series_result {
        results = results.push(series_result_widget(series, image_bytes.clone()));
    }
    results.spacing(5).into()
}

pub fn series_result_widget(
    series_result: &series_searching::SeriesSearchResult,
    image_bytes: Option<Vec<u8>>,
) -> iced::widget::Row<'_, Message, Renderer> {
    let mut row = row!();

    if let Some(image_bytes) = image_bytes {
        let image_handle = image::Handle::from_memory(image_bytes);

        let image = image(image_handle).height(60);
        row = row
            .push(horizontal_space(5))
            .push(image)
            .push(horizontal_space(5));
    }

    // Getting the series genres
    let genres = if !series_result.show.genres.is_empty() {
        let mut genres = String::from("Genres: ");

        let mut series_result_iter = series_result.show.genres.iter().peekable();
        while let Some(genre) = series_result_iter.next() {
            genres.push_str(genre);
            if let Some(_) = series_result_iter.peek() {
                genres.push_str(", ");
            }
        }
        genres
    } else {
        String::new()
    };

    let mut column = column!(
        text(&series_result.show.name).size(20),
        text(genres).size(15),
    );

    if let Some(premier) = &series_result.show.premiered {
        column = column.push(text(format!("Premiered: {}", premier)).size(13));
    }

    row.push(column)
}
