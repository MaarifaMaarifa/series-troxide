use crate::core::api::seasons_list::{get_seasons_list, Season as SeasonInfo};
use crate::core::api::series_information::get_series_main_info_with_id;
use crate::core::api::series_information::SeriesMainInformation;
use crate::core::api::{load_image, Image};
use crate::gui::troxide_widget::{INFO_BODY, INFO_HEADER};
use crate::gui::Message as GuiMessage;
use iced::widget::{Column, Row};
use iced::{
    alignment,
    widget::{button, column, container, horizontal_space, image, row, scrollable, text},
    Length, Renderer,
};
use iced::{Command, Element};
use season_widget::Message as SeasonMessage;

mod season_widget;

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
            if series_result_iter.peek().is_some() {
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
        if let Some(language) = &series_info.language {
            text(language)
        } else {
            text("unavailable")
        }
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
    if let Some(summary) = &series_info.summary {
        text(summary).size(15)
    } else {
        text("")
    }
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
        genres_widget(series_information),
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

fn top_bar(series_info: &SeriesMainInformation) -> Row<'_, Message, Renderer> {
    row!(
        button("<-").on_press(Message::GoToSearchPage),
        horizontal_space(Length::Fill),
        text(&series_info.name).size(30),
        horizontal_space(Length::Fill),
        button("add to track list")
    )
}

#[derive(Clone, Debug)]
pub enum Message {
    SeriesInfoObtained(Box<SeriesMainInformation>),
    SeriesImageLoaded(Option<Vec<u8>>),
    GoToSearchPage,
    SeasonsLoaded(Vec<SeasonInfo>),
    SeasonAction(usize, Box<SeasonMessage>),
}

enum LoadState {
    Loading,
    Loaded,
}

pub struct Series {
    series_id: u32,
    load_state: LoadState,
    series_information: Option<SeriesMainInformation>,
    series_image: Option<Vec<u8>>,
    season_widgets: Vec<season_widget::Season>,
}

impl Series {
    /// Counstruct the series page by providing it with id
    pub fn from_series_id(series_id: u32) -> (Self, Command<GuiMessage>) {
        let series = Self {
            series_id,
            load_state: LoadState::Loading,
            series_information: None,
            series_image: None,
            season_widgets: vec![],
        };

        (
            series,
            Command::perform(get_series_main_info_with_id(series_id), |info| {
                GuiMessage::SeriesAction(Message::SeriesInfoObtained(Box::new(
                    info.expect("Failed to load series information"),
                )))
            }),
        )
    }

    /// Counstruct the series page by providing it with SeriesMainInformation
    pub fn from_series_information(
        series_information: SeriesMainInformation,
    ) -> (Self, Command<GuiMessage>) {
        let series_id = series_information.id;
        let series_image = series_information.image.clone();
        let series = Self {
            series_id,
            load_state: LoadState::Loaded,
            series_information: Some(series_information),
            series_image: None,
            season_widgets: vec![],
        };

        (
            series,
            Command::batch(get_image_and_seasons(series_image, series_id)),
        )
    }

    pub fn update(&mut self, message: Message) -> Command<GuiMessage> {
        match message {
            Message::SeriesInfoObtained(info) => {
                self.load_state = LoadState::Loaded;
                let info_image = info.image.clone();
                self.series_information = Some(*info);

                return Command::batch(get_image_and_seasons(info_image, self.series_id));
            }
            Message::SeriesImageLoaded(image) => {
                self.series_image = image;
            }
            Message::GoToSearchPage => {
                return Command::perform(async {}, |_| {
                    GuiMessage::SeriesAction(Message::GoToSearchPage)
                })
            }
            Message::SeasonsLoaded(season_list) => {
                self.season_widgets = season_list
                    .into_iter()
                    .enumerate()
                    .map(|(index, season)| {
                        season_widget::Season::new(index, season, self.series_id)
                    })
                    .collect()
            }
            Message::SeasonAction(index, message) => {
                return self.season_widgets[index]
                    .update(*message)
                    .map(GuiMessage::SeriesAction);
            }
        }
        Command::none()
    }

    pub fn view(&self) -> Element<Message, Renderer> {
        match self.load_state {
            LoadState::Loading => text("Loading..").into(),
            LoadState::Loaded => {
                let main_body = series_page(
                    self.series_information.as_ref().unwrap(),
                    self.series_image.clone(),
                );
                let seasons_widget = column!(
                    text("Seasons").size(25),
                    Column::with_children(
                        self.season_widgets
                            .iter()
                            .enumerate()
                            .map(|(index, widget)| {
                                widget
                                    .view()
                                    .map(move |m| Message::SeasonAction(index, Box::new(m)))
                            })
                            .collect(),
                    )
                    .spacing(5)
                );

                let content = scrollable(column!(main_body, seasons_widget));
                column!(top_bar(self.series_information.as_ref().unwrap()), content).into()
            }
        }
    }
}

/// Returns two commands that requests series' image and seasons list
fn get_image_and_seasons(
    series_info_image: Option<Image>,
    series_id: u32,
) -> [Command<GuiMessage>; 2] {
    let image_command = if let Some(image_url) = series_info_image {
        Command::perform(load_image(image_url.original_image_url), |image| {
            GuiMessage::SeriesAction(Message::SeriesImageLoaded(image))
        })
    } else {
        Command::none()
    };

    let seasons_list_command = Command::perform(get_seasons_list(series_id), |seasons_list| {
        GuiMessage::SeriesAction(Message::SeasonsLoaded(
            seasons_list.expect("Failed to load seasons"),
        ))
    });

    [image_command, seasons_list_command]
}
