use crate::core::api::series_searching;
use crate::gui::Message;
use iced::{
    widget::{column, horizontal_space, image, row, text},
    Renderer,
};
use iced::{Command, Element};

// The text size of the beginning part of a info
pub const INFO_HEADER: u16 = 18;
// The text size of the main part of a info
pub const INFO_BODY: u16 = 15;

// const INFO_BODY_HEIGHT: u16 = INFO_HEADER - (INFO_HEADER - INFO_BODY);

/// Generates the SeriesSearchResult widget
pub fn series_result(
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

#[derive(Clone, Debug)]
pub enum SMessage {}

pub struct SearchResult {
    series_search_result: series_searching::SeriesSearchResult,
    image_bytes: Option<Vec<u8>>,
}

impl SearchResult {
    pub fn new(
        series_search_result: series_searching::SeriesSearchResult,
        image_bytes: Option<Vec<u8>>,
    ) -> Self {
        Self {
            series_search_result,
            image_bytes,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        todo!()
    }
    pub fn view(&self) -> Element<SMessage, Renderer> {
        let mut row = row!();

        if let Some(image_bytes) = &self.image_bytes {
            let image_handle = image::Handle::from_memory(image_bytes.clone());

            let image = image(image_handle).height(60);
            row = row
                .push(horizontal_space(5))
                .push(image)
                .push(horizontal_space(5));
        }

        // Getting the series genres
        let genres = if !self.series_search_result.show.genres.is_empty() {
            let mut genres = String::from("Genres: ");

            let mut series_result_iter = self.series_search_result.show.genres.iter().peekable();
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
            text(&self.series_search_result.show.name).size(20),
            text(genres).size(15),
        );

        if let Some(premier) = &self.series_search_result.show.premiered {
            column = column.push(text(format!("Premiered: {}", premier)).size(13));
        }

        row.push(column).into()
    }
}
