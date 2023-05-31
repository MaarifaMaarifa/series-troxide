use crate::core::api::series_information::SeriesMainInformation;
use crate::gui::troxide_widget::{INFO_BODY, INFO_HEADER};
use crate::gui::Message;
use iced::{
    alignment,
    widget::{button, column, container, horizontal_space, image, row, scrollable, text},
    Length, Renderer,
};

enum SeriesStatus {
    Running,
    Ended,
    ToBeDetermined,
    InDevelopment,
    Other,
}

impl SeriesStatus {
    fn new(series_info: &SeriesMainInformation) -> Self {
        match series_info.status.as_ref() {
            "Running" => Self::Running,
            "Ended" => Self::Ended,
            "To Be Determined" => Self::ToBeDetermined,
            "In Development" => Self::InDevelopment,
            _ => Self::Other,
        }
    }
}

const RED_COLOR: iced::Color = iced::Color::from_rgb(2.55, 0.0, 0.0);
const GREEN_COLOR: iced::Color = iced::Color::from_rgb(0.0, 1.28, 0.0);

const RED_THEME: iced::theme::Text = iced::theme::Text::Color(RED_COLOR);
const GREEN_THEME: iced::theme::Text = iced::theme::Text::Color(GREEN_COLOR);

fn status_widget(series_info: &SeriesMainInformation) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(text("Status: ").size(INFO_HEADER));

    let status_text = match SeriesStatus::new(series_info) {
        SeriesStatus::Running => text("Running").style(GREEN_THEME),
        SeriesStatus::Ended => text("Ended").style(RED_THEME),
        SeriesStatus::ToBeDetermined => text("To Be Determined"),
        SeriesStatus::InDevelopment => text("In Development"),
        SeriesStatus::Other => text(&series_info.status),
    }
    .vertical_alignment(alignment::Vertical::Bottom)
    .size(INFO_BODY)
    .height(INFO_HEADER);

    row.push(status_text)
}

fn average_runtime_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(text("Average runtime: ").size(INFO_HEADER));
    let body_widget = if let Some(average_runtime) = series_info.average_runtime {
        text(format!("{} mins", average_runtime))
    } else {
        text("unavailable")
    };
    row.push(
        body_widget
            .size(INFO_BODY)
            .height(INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

fn genres_widget(series_info: &SeriesMainInformation) -> iced::widget::Row<'_, Message, Renderer> {
    if !series_info.genres.is_empty() {
        let row = row!(text("Genres: ").size(INFO_HEADER));
        let mut genres = String::new();

        let mut series_result_iter = series_info.genres.iter().peekable();
        while let Some(genre) = series_result_iter.next() {
            genres.push_str(genre);
            if let Some(_) = series_result_iter.peek() {
                genres.push_str(", ");
            }
        }
        row.push(text(genres).size(INFO_BODY))
    } else {
        row!()
    }
}

fn language_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(
        text("Language: ").size(INFO_HEADER),
        text(&series_info.language)
            .size(INFO_BODY)
            .height(INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom)
    );
    row
}

fn premiered_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(text("Premiered: ").size(INFO_HEADER));
    let body_text = if let Some(premier) = &series_info.premiered {
        text(premier)
    } else {
        text("unavailable")
    };

    row.push(
        body_text
            .size(INFO_BODY)
            .height(INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

fn ended_widget(series_info: &SeriesMainInformation) -> iced::widget::Row<'_, Message, Renderer> {
    // Creating the widget only when the series has ended
    match SeriesStatus::new(series_info) {
        SeriesStatus::Ended => {}
        _ => return row!(),
    }

    let row = row!(text("Ended: ").size(INFO_HEADER));
    let body_text = if let Some(ended) = &series_info.ended {
        text(ended)
    } else {
        text("unavailable")
    };

    row.push(
        body_text
            .size(INFO_BODY)
            .height(INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

fn summary_widget(series_info: &SeriesMainInformation) -> iced::widget::Text<'_, Renderer> {
    text(&series_info.summary).size(15)
}

fn rating_widget(series_info: &SeriesMainInformation) -> iced::widget::Row<'_, Message, Renderer> {
    let row = row!(text("Average rating: ").size(INFO_HEADER));
    let body_wiget = if let Some(average_rating) = series_info.rating.average {
        text(average_rating.to_string())
    } else {
        text("unavailable")
    };

    row.push(
        body_wiget
            .size(INFO_BODY)
            .height(INFO_HEADER)
            .vertical_alignment(alignment::Vertical::Bottom),
    )
}

fn network_widget(series_info: &SeriesMainInformation) -> iced::widget::Row<'_, Message, Renderer> {
    if let Some(network) = &series_info.network {
        // TODO: Add a clickable link
        row!(
            text("Network:  ").size(INFO_HEADER),
            text(format!("{} ({})", &network.name, &network.country.name))
                .size(INFO_BODY)
                .height(INFO_HEADER)
                .vertical_alignment(alignment::Vertical::Bottom),
        )
    } else {
        row!()
    }
}

fn webchannel_widget(
    series_info: &SeriesMainInformation,
) -> iced::widget::Row<'_, Message, Renderer> {
    if let Some(webchannel) = &series_info.web_channel {
        // TODO: Add a clickable link
        row!(
            text("Webchannel: ").size(INFO_HEADER),
            text(&webchannel.name)
                .size(INFO_BODY)
                .height(INFO_HEADER)
                .vertical_alignment(alignment::Vertical::Bottom),
        )
    } else {
        row!()
    }
}

/// Generates the Series Page
pub fn series_page(
    series_information: &SeriesMainInformation,
    image_bytes: Option<Vec<u8>>,
) -> container::Container<'_, Message, Renderer> {
    let mut content = column!();

    let header = row!(
        button("<-"),
        horizontal_space(Length::Fill),
        text(&series_information.name).size(30),
        horizontal_space(Length::Fill),
        button("add to track list")
    );

    content = content.push(header);

    let mut main_info = row!().padding(5);

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
        genres_widget(&series_information),
        language_widget(series_information),
        average_runtime_widget(series_information),
        rating_widget(series_information),
        network_widget(series_information),
        webchannel_widget(series_information),
        premiered_widget(series_information),
        ended_widget(series_information),
        summary_widget(series_information),
    )
    .spacing(3)
    .padding(5);

    main_info = main_info.push(series_data);

    content = content.push(main_info);

    container(scrollable(content))
}
