use crate::{api::series_information, api::series_searching, Message};
use iced::{
    widget::{button, column, container, horizontal_space, image, row, text},
    Alignment, Renderer,
};

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

/// Generates the Series Page
pub fn series_page(
    series_information: &series_information::SeriesMainInformation,
    image_bytes: Option<Vec<u8>>,
) -> container::Container<'_, Message, Renderer> {
    let mut content = column!();

    let header = row!(
        button("<-").on_press(Message::GoToSearchPage),
        text(&series_information.name).size(30),
        button("Track Series").on_press(Message::TrackSeries)
    );

    content = content.push(header);

    let mut main_info = row!();

    // Putting the image to the main info
    if let Some(image_bytes) = image_bytes {
        let image_handle = image::Handle::from_memory(image_bytes);
        let image = image(image_handle).height(250);
        main_info = main_info.push(image);
    }

    // Getting genres
    // Putting series information to the main info
    let series_data = column!(
        text(format!("Status: {}", series_information.status)),
        text(genres_parse(&series_information.genres)).size(18),
        text(&series_information.summary).size(15),
    )
    .spacing(3);

    main_info = main_info.push(series_data);

    content = content.push(main_info);

    container(content)
}

fn genres_parse(genres: &Vec<String>) -> String {
    if !genres.is_empty() {
        let mut parsed_genres = String::from("Genres: ");

        let mut series_result_iter = genres.iter().peekable();
        while let Some(genre) = series_result_iter.next() {
            parsed_genres.push_str(genre);
            if let Some(_) = series_result_iter.peek() {
                parsed_genres.push_str(", ");
            }
        }
        parsed_genres
    } else {
        String::new()
    }
}
