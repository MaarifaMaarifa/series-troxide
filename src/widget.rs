use crate::{api::series_searching, Message};
use iced::{
    widget::{column, horizontal_space, image, row, text},
    Renderer,
};

/// Generates the SeriesSearchResult widget
pub fn series_result(
    series_result: &series_searching::SeriesSearchResultLoaded,
) -> iced::widget::Row<'_, Message, Renderer> {
    let mut row = row!();

    if let Some(image_bytes) = &series_result.image_bytes {
        let image_handle = image::Handle::from_memory(image_bytes.clone());

        let image = image(image_handle).height(60);
        row = row
            .push(horizontal_space(5))
            .push(image)
            .push(horizontal_space(5));
    }

    // Getting the series genres
    let genres = if !series_result.genres.is_empty() {
        let mut genres = String::from("Genres: ");

        let mut series_result_iter = series_result.genres.iter().peekable();
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

    let mut column = column!(text(&series_result.name).size(20), text(genres).size(15),);

    if let Some(premier) = &series_result.premiered {
        column = column.push(text(format!("Premiered: {}", premier)).size(13));
    }

    row.push(column)
}
