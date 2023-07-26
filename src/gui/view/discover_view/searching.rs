use bytes::Bytes;
use iced::widget::{
    column, container, horizontal_space, image, mouse_area, row, scrollable, text, text_input,
    vertical_space, Column, Space,
};
use iced::{Command, Element, Length, Renderer};
use iced_aw::Spinner;
use tokio::task::JoinHandle;

use super::Message as DiscoverMessage;
use crate::core::api::series_searching;
use crate::core::caching;
use crate::gui::styles;

#[derive(Default)]
pub enum LoadState {
    Loaded,
    Loading,
    #[default]
    NotLoaded,
}

#[derive(Clone, Debug)]
pub enum Message {
    SearchTermChanged(String),
    SearchTermSearched,
    SearchSuccess(Vec<series_searching::SeriesSearchResult>),
    SearchFail,
    ImagesLoaded(Vec<Option<Bytes>>),
    SeriesResultPressed(/*series id*/ u32),
}

#[derive(Default)]
pub struct Search {
    search_term: String,
    series_search_result: Vec<series_searching::SeriesSearchResult>,
    series_search_results_images: Vec<Option<Bytes>>,
    pub load_state: LoadState,
}

impl Search {
    pub fn update(&mut self, message: Message) -> Command<DiscoverMessage> {
        match message {
            Message::SearchTermChanged(term) => {
                self.search_term = term;
                return if self.search_term.is_empty() {
                    self.load_state = LoadState::NotLoaded;
                    Command::perform(async {}, |_| DiscoverMessage::HideOverlay)
                } else {
                    Command::none()
                };
            }
            Message::SearchTermSearched => {
                self.load_state = LoadState::Loading;

                let series_result = series_searching::search_series(self.search_term.clone());

                let search_status_command = Command::perform(series_result, |res| match res {
                    Ok(res) => DiscoverMessage::SearchAction(Message::SearchSuccess(res)),
                    Err(err) => {
                        println!("{:?}", err);
                        DiscoverMessage::SearchAction(Message::SearchFail)
                    }
                });

                let show_overlay_command =
                    Command::perform(async {}, |_| DiscoverMessage::ShowOverlay);

                return Command::batch([search_status_command, show_overlay_command]);
            }
            Message::SearchSuccess(res) => {
                self.load_state = LoadState::Loaded;
                self.series_search_results_images.clear();
                self.series_search_result = res.clone();
                let image_command = Command::perform(load_series_result_images(res), |images| {
                    DiscoverMessage::SearchAction(Message::ImagesLoaded(images))
                });
                let show_overlay_command =
                    Command::perform(async {}, |_| DiscoverMessage::ShowOverlay);

                return Command::batch([image_command, show_overlay_command]);
            }
            Message::SearchFail => panic!("Series Search Failed"),
            Message::ImagesLoaded(images) => self.series_search_results_images = images,
            Message::SeriesResultPressed(_) => {
                unreachable!("Search page should not handle series page result")
            }
        }
        Command::none()
    }

    pub fn view(
        &self,
    ) -> (
        Element<'_, Message, Renderer>,
        Element<'_, Message, Renderer>,
    ) {
        let search_bar = column!(
            vertical_space(10),
            text_input("Search Series", &self.search_term)
                .width(300)
                .on_input(Message::SearchTermChanged)
                .on_submit(Message::SearchTermSearched)
        )
        .width(Length::Fill)
        .align_items(iced::Alignment::Center);

        let menu_widgets: Element<'_, Message, Renderer> = match self.load_state {
            LoadState::Loaded => {
                let items = load(
                    &self.series_search_result,
                    &self.series_search_results_images,
                );

                if items.is_empty() {
                    container(text("No results"))
                        .padding(10)
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .center_x()
                        .center_y()
                        .into()
                } else {
                    Column::with_children(items).padding(20).spacing(5).into()
                }
            }
            LoadState::Loading => container(Spinner::new())
                .height(Length::Fill)
                .width(Length::Fill)
                .center_x()
                .center_y()
                .into(),
            LoadState::NotLoaded => container("").into(),
        };

        let menu_widgets = container(menu_widgets)
            .style(styles::container_styles::first_class_container_rounded_theme());

        (search_bar.into(), scrollable(menu_widgets).into())
    }
}

fn load<'a>(
    series_result: &'a [series_searching::SeriesSearchResult],
    series_images: &[Option<Bytes>],
) -> Vec<Element<'a, Message, Renderer>> {
    let mut results = Vec::new();

    for (index, series_result) in series_result.iter().enumerate() {
        results.push(series_result_widget(
            series_result,
            if series_images.is_empty() {
                None
            } else {
                series_images[index].clone().take()
            },
        ));
    }
    results
}

pub fn series_result_widget(
    series_result: &series_searching::SeriesSearchResult,
    image_bytes: Option<Bytes>,
) -> iced::Element<'_, Message, Renderer> {
    let mut row = row!();

    let image: Element<'_, Message, Renderer> = if let Some(image_bytes) = image_bytes {
        let image_handle = image::Handle::from_memory(image_bytes);
        image(image_handle).height(60).into()
    } else {
        Space::new(43, 60).into()
    };

    row = row
        .push(horizontal_space(5))
        .push(image)
        .push(horizontal_space(5));

    // Getting the series genres
    let genres = if !series_result.show.genres.is_empty() {
        let mut genres = String::from("Genres: ");

        let mut series_result_iter = series_result.show.genres.iter().peekable();
        while let Some(genre) = series_result_iter.next() {
            genres.push_str(genre);
            if series_result_iter.peek().is_some() {
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

    mouse_area(row.push(column))
        .on_press(Message::SeriesResultPressed(series_result.show.id))
        .into()
}

async fn load_series_result_images(
    series_results: Vec<series_searching::SeriesSearchResult>,
) -> Vec<Option<Bytes>> {
    let mut loaded_results = Vec::with_capacity(series_results.len());
    let handles: Vec<JoinHandle<Option<Bytes>>> = series_results
        .into_iter()
        .map(|result| {
            tokio::task::spawn(async {
                if let Some(url) = result.show.image {
                    caching::load_image(url.medium_image_url).await
                } else {
                    None
                }
            })
        })
        .collect();

    for handle in handles {
        let loaded_result = handle
            .await
            .expect("Failed to await all the search images handles");
        loaded_results.push(loaded_result)
    }
    loaded_results
}
