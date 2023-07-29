use iced::widget::{column, container, horizontal_space, pick_list, row, text};
use iced::{Element, Length, Renderer};
use rust_iso3166::ALL;

use crate::core::settings_config::{locale_settings, SETTINGS};
use crate::gui::styles;

#[derive(Clone, Debug)]
pub enum Message {
    CountrySelected(String),
}

#[derive(Default)]
pub struct Locale {}

impl Locale {
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
        use locale_settings::{get_country_code_from_settings, get_country_name_from_country_code};

        let content = column![text("Locale")
            .size(21)
            .style(styles::text_styles::purple_text_theme())]
        .padding(5)
        .spacing(5);

        let country_list = ALL
            .iter()
            .map(|country_code| country_code.name.to_owned())
            .collect::<Vec<String>>();

        let selected_country =
            get_country_name_from_country_code(&get_country_code_from_settings())
                .unwrap()
                .to_owned();

        let country_setting_info = column![
        text("Country").size(18),
        text("The chosen country will be used by the discover page to provide locally aired series.").size(11)];

        let theme_picklist = pick_list(
            country_list,
            Some(selected_country),
            Message::CountrySelected,
        );

        let content = content.push(
            row!(
                country_setting_info,
                horizontal_space(Length::Fill),
                theme_picklist
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
