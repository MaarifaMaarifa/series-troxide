use crate::gui::Message;
use crate::{api::series_information, api::series_searching};
use iced::alignment;
use iced::{
    widget::{button, column, container, horizontal_space, image, row, text},
    Alignment, Renderer,
};

pub mod series_page;

// The text size of the beginning part of a info
const INFO_HEADER: u16 = 18;
// The text size of the main part of a info
const INFO_BODY: u16 = 15;

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

fn genres_widget(genres: &Vec<String>) -> iced::widget::Row<'_, Message, Renderer> {
    if !genres.is_empty() {
        let parsed_genres_row = row!(text("Genres: ").size(INFO_HEADER));

        let mut parsed_genres = String::new();
        let mut series_result_iter = genres.iter().peekable();
        while let Some(genre) = series_result_iter.next() {
            parsed_genres.push_str(genre);
            if let Some(_) = series_result_iter.peek() {
                parsed_genres.push_str(", ");
            }
        }
        parsed_genres_row.push(
            text(parsed_genres)
                .size(INFO_BODY)
                .height(INFO_HEADER)
                .vertical_alignment(alignment::Vertical::Bottom),
        )
    } else {
        row!()
    }
}
