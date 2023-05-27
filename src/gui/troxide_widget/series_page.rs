use crate::api::series_information::SeriesMainInformation;
use crate::gui::Message;
use iced::{
    alignment,
    widget::{button, column, container, horizontal_space, image, row, text, text::Appearance},
    Alignment, Length, Renderer,
};

const RED_COLOR: iced::Color = iced::Color::from_rgb(2.55, 0.0, 0.0);
const GREEN_COLOR: iced::Color = iced::Color::from_rgb(0.0, 1.28, 0.0);

const RED_THEME: iced::theme::Text = iced::theme::Text::Color(RED_COLOR);
const GREEN_THEME: iced::theme::Text = iced::theme::Text::Color(GREEN_COLOR);

fn status_widget(series_info: &SeriesMainInformation) -> iced::widget::Row<'_, Message, Renderer> {
    let status_str = &series_info.status;

    let row = row!(text("Status: ").size(super::INFO_HEADER));
    let status_text = match status_str.as_ref() {
        "Running" => text("Running").style(GREEN_THEME),
        "Ended" => text("Ended").style(RED_THEME),
        rest => text(rest),
    }
    .vertical_alignment(alignment::Vertical::Bottom)
    .size(super::INFO_BODY)
    .height(super::INFO_HEADER);

    row.push(status_text)
}

fn average_runtime_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    let mut row = row!(text("Average runtime: ").size(super::INFO_HEADER));
    if let Some(average_runtime) = series_info.average_runtime {
        row = row.push(
            text(format!("{} mins", average_runtime))
                .size(super::INFO_BODY)
                .vertical_alignment(alignment::Vertical::Bottom)
                .height(super::INFO_HEADER),
        )
    } else {
        row = row.push(text("unavailable").size(super::INFO_BODY))
    }
    row
}

/// Generates the Series Page
pub fn series_page(
    series_information: &SeriesMainInformation,
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
        // text(format!("Status: {}", series_information.status)),
        status_widget(series_information),
        super::genres_widget(&series_information.genres),
        text(format!("Language: {}", series_information.language)),
        average_runtime_widget(series_information),
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
