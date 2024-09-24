use iced::widget::{column, combo_box, container, text};
use iced::{Element, Task};
use locale_settings::{get_country_code_from_settings, get_country_name_from_country_code};
use rust_iso3166::ALL;

use crate::core::settings_config::{locale_settings, SETTINGS};
use crate::gui::styles;
use hidden_series::{HiddenSeries, Message as HiddenSeriesMessage};

#[derive(Clone, Debug)]
pub enum Message {
    CountrySelected(String),
    HiddenSeries(HiddenSeriesMessage),
}

pub struct Discover {
    country_combo_box_state: combo_box::State<String>,
    hidden_series: HiddenSeries,
}

impl Discover {
    pub fn new() -> Self {
        let country_list = ALL
            .iter()
            .map(|country_code| country_code.name.to_owned())
            .collect::<Vec<String>>();

        Self {
            country_combo_box_state: combo_box::State::new(country_list),
            hidden_series: HiddenSeries,
        }
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CountrySelected(country_name) => {
                let country_code =
                    locale_settings::get_country_code_from_country_name(&country_name).unwrap();

                country_code.clone_into(
                    &mut SETTINGS
                        .write()
                        .unwrap()
                        .change_settings()
                        .locale
                        .country_code,
                );
                Task::none()
            }
            Message::HiddenSeries(message) => self
                .hidden_series
                .update(message)
                .map(Message::HiddenSeries),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = column![
            text("Discover")
                .size(21)
                .style(styles::text_styles::accent_color_theme),
            self.country_widget(),
            self.hidden_series.view().map(Message::HiddenSeries),
        ]
        .padding(5)
        .spacing(10);

        container(content)
            .style(styles::container_styles::first_class_container_rounded_theme)
            .width(1000)
            .into()
    }

    pub fn country_widget(&self) -> Element<'_, Message> {
        let selected_country =
            get_country_name_from_country_code(&get_country_code_from_settings())
                .unwrap()
                .to_owned();

        let country_setting_info = column![
        text("Country").size(18),
        text("The \"local aired series section\" of the discover page will display locally aired series from the selected country if available.").size(11)];

        let country_combo_box = combo_box(
            &self.country_combo_box_state,
            "select a country",
            Some(&selected_country),
            Message::CountrySelected,
        )
        .width(500);

        column![country_setting_info, country_combo_box]
            .spacing(5)
            .into()
    }
}

impl Default for Discover {
    fn default() -> Self {
        Self::new()
    }
}

mod hidden_series {
    use iced::widget::{button, column, container, row, scrollable, text, Column, Space};
    use iced::{Element, Task};

    use crate::{core::posters_hiding::HIDDEN_SERIES, gui::styles};

    #[derive(Clone, Debug)]
    pub enum Message {
        UnhideSeries(u32),
        SeriesUnhidden,
    }

    pub struct HiddenSeries;

    impl HiddenSeries {
        pub fn update(&mut self, message: Message) -> Task<Message> {
            match message {
                Message::UnhideSeries(series_id) => Task::perform(
                    async move {
                        let mut hidden_series = HIDDEN_SERIES.write().await;
                        hidden_series.unhide_series(series_id).await
                    },
                    |_| Message::SeriesUnhidden,
                ),
                Message::SeriesUnhidden => Task::none(),
            }
        }

        pub fn view(&self) -> Element<'_, Message> {
            let hidden_series = HIDDEN_SERIES.blocking_read();

            let content = if let Some(hidden_series) = hidden_series.get_hidden_series() {
                let content: Element<'_, Message> = if hidden_series.is_empty() {
                    Self::empty_posters_widget()
                } else {
                    let content = Column::with_children(hidden_series.iter().map(
                        |(series_id, (series_name, premier_date))| {
                            Self::series_entry(
                                *series_id,
                                series_name.clone(),
                                premier_date.clone(),
                            )
                        },
                    ))
                    .spacing(5);

                    scrollable(container(content).padding(10))
                        .direction(styles::scrollable_styles::vertical_direction())
                        .into()
                };
                content
            } else {
                Self::empty_posters_widget()
            };

            let content = container(content)
                .padding(5)
                .max_height(500)
                .style(styles::container_styles::second_class_container_rounded_theme);

            column![text("Hidden Discover Posters").size(18), content]
                .spacing(5)
                .into()
        }

        fn empty_posters_widget() -> Element<'static, Message> {
            container(text("No hidden posters")).center_x(200).into()
        }

        fn series_entry(
            series_id: u32,
            series_name: String,
            premier_date: Option<String>,
        ) -> Element<'static, Message> {
            let unhide_button = button(text("unhide").size(11))
                .style(styles::button_styles::transparent_button_with_rounded_border_theme)
                .on_press(Message::UnhideSeries(series_id));
            let premier_date: Element<'_, Message> = if let Some(premier_date) = premier_date {
                text(format!("({})", premier_date))
                    .style(styles::text_styles::accent_color_theme)
                    .into()
            } else {
                Space::new(0, 0).into()
            };

            row![unhide_button, text(series_name), premier_date]
                .spacing(5)
                .into()
        }
    }
}
