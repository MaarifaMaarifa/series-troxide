use iced::widget::{column, combo_box, container, horizontal_space, row, text};
use iced::{Element, Length, Renderer};
use locale_settings::{get_country_code_from_settings, get_country_name_from_country_code};
use rust_iso3166::ALL;

use crate::core::settings_config::{locale_settings, SETTINGS};
use crate::gui::styles;

#[derive(Clone, Debug)]
pub enum Message {
    CountrySelected(String),
}

pub struct Locale {
    country_combo_box_state: combo_box::State<String>,
}

impl Locale {
    pub fn new() -> Self {
        let country_list = ALL
            .iter()
            .map(|country_code| country_code.name.to_owned())
            .collect::<Vec<String>>();

        Self {
            country_combo_box_state: combo_box::State::new(country_list),
        }
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::CountrySelected(country_name) => {
                let country_code =
                    locale_settings::get_country_code_from_country_name(&country_name).unwrap();

                SETTINGS
                    .write()
                    .unwrap()
                    .change_settings()
                    .locale
                    .country_code = country_code.to_owned();
            }
        };
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let content = column![text("Locale")
            .size(21)
            .style(styles::text_styles::purple_text_theme())]
        .padding(5)
        .spacing(5);

        let selected_country =
            get_country_name_from_country_code(&get_country_code_from_settings())
                .unwrap()
                .to_owned();

        let country_setting_info = column![
        text("Country").size(18),
        text("The chosen country will be used by the discover page to provide locally aired series.").size(11)];

        let country_combo_box = combo_box(
            &self.country_combo_box_state,
            "select a country",
            Some(&selected_country),
            Message::CountrySelected,
        );

        let content = content.push(
            row!(
                country_setting_info,
                horizontal_space(Length::Fill),
                country_combo_box
            )
            .padding(5)
            .spacing(5),
        );

        container(content)
            .style(styles::container_styles::first_class_container_rounded_theme())
            .width(1000)
            .into()
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new()
    }
}
