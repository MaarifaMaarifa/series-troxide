use crate::gui::Message;
use crate::{api::series_information, api::series_searching};
use iced::{
    widget::{button, column, container, horizontal_space, image, row, text},
    Alignment, Renderer,
};

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
        text(super::genres_parse(&series_information.genres)).size(18),
        text(format!("Language: {}", series_information.language)),
        text(format!(
            "Average runtime(mins): {}",
            series_information
                .average_runtime
                .map_or("Unavailable".to_owned(), |t| t.to_string())
        )),
        text(format!(
            "Premiered: {}",
            series_information
                .premiered
                .as_ref()
                .map_or("unavailable".to_owned(), |p| p.clone())
        )),
        text(format!(
            "Ended: {}",
            series_information
                .ended
                .as_ref()
                .map_or("unavailable".to_owned(), |p| p.clone())
        )),
        text(&series_information.summary).size(15),
    )
    .spacing(3);

    main_info = main_info.push(series_data);

    content = content.push(main_info);

    container(content)
}
