use crate::api::series_information::SeriesMainInformation;
use crate::gui::Message;
use iced::{
    alignment,
    widget::{button, column, container, horizontal_space, image, row, text, text::Appearance},
    Alignment, Length, Renderer,
};

enum SeriesStatus {
    Running,
    Ended,
    ToBeDetermined,
    Other,
}

impl SeriesStatus {
    fn new(series_info: &SeriesMainInformation) -> Self {
        match series_info.status.as_ref() {
            "Running" => Self::Running,
            "Ended" => Self::Ended,
            "To Be Determined" => Self::ToBeDetermined,
            _ => Self::Other,
        }
    }
}

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
    let row = row!(text("Average runtime: ").size(super::INFO_HEADER));
    let body_widget = if let Some(average_runtime) = series_info.average_runtime {
        text(format!("{} mins", average_runtime))
    } else {
        text("unavailable")
    };
    row.push(
        body_widget
            .size(super::INFO_BODY)
            .height(super::INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

fn language_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(
        text("Language: ").size(super::INFO_HEADER),
        text(&series_info.language)
            .size(super::INFO_BODY)
            .height(super::INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom)
    );
    row
}

fn premiered_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(text("Premiered: ").size(super::INFO_HEADER));
    let body_text = if let Some(premier) = &series_info.premiered {
        text(premier)
    } else {
        text("unavailable")
    };

    row.push(
        body_text
            .size(super::INFO_BODY)
            .height(super::INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

fn ended_widget(series_info: &SeriesMainInformation) -> iced::widget::Row<'_, Message, Renderer> {
    if let SeriesStatus::Running = SeriesStatus::new(series_info) {
        return row!();
    }

    let row = row!(text("Ended: ").size(super::INFO_HEADER));
    let body_text = if let Some(ended) = &series_info.ended {
        text(ended)
    } else {
        text("unavailable")
    };

    row.push(
        body_text
            .size(super::INFO_BODY)
            .height(super::INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

fn summary_widget(series_info: &SeriesMainInformation) -> iced::widget::Text<'_, Renderer> {
    text(&series_info.summary).size(15)
}

fn rating_widget(series_info: &SeriesMainInformation) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(text("Average rating: ").size(super::INFO_HEADER));
    let body_wiget = if let Some(average_rating) = series_info.rating.average {
        text(average_rating.to_string())
    } else {
        text("unavailable")
    };

    row.push(
        body_wiget
            .size(super::INFO_BODY)
            .height(super::INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

fn network_widget(series_info: &SeriesMainInformation) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(text("Network:  ").size(super::INFO_HEADER));
    let body_wiget = if let Some(network) = &series_info.network {
        // TODO: Add a clickable link
        text(format!("{} ({})", &network.name, &network.country.name))
    } else {
        text("unavailable")
    };

    row.push(
        body_wiget
            .size(super::INFO_BODY)
            .height(super::INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

fn webchannel_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(text("Webchannel: ").size(super::INFO_HEADER));
    let body_wiget = if let Some(webchannel) = &series_info.web_channel {
        // TODO: Add a clickable link
        text(&webchannel.name)
    } else {
        text("unavailable")
    };

    row.push(
        body_wiget
            .size(super::INFO_BODY)
            .height(super::INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
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
        language_widget(series_information),
        average_runtime_widget(series_information),
        rating_widget(series_information),
        network_widget(series_information),
        webchannel_widget(series_information),
        premiered_widget(series_information),
        ended_widget(series_information),
        summary_widget(series_information),
    )
    .spacing(3);

    main_info = main_info.push(series_data);

    content = content.push(main_info);

    container(content)
}
