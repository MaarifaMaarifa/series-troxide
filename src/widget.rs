use crate::{api::series_searching, Message};
use iced::{
    widget::{column, image, row, text},
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
        row = row.push(image);
    }
    let mut column = column!(
        text(&series_result.name),
        text(format!("{:?}", &series_result.genres)),
    );

    if let Some(premier) = &series_result.premiered {
        column = column.push(text(premier));
    }

    row.push(column)
}
