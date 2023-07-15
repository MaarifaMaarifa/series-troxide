use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, text, vertical_space,
};
use iced::{Alignment, Element, Length, Renderer};

use crate::core::settings_config::{save_config, Config, Theme, ALL_THEMES};
use crate::gui::assets::icons::GEAR_WIDE_CONNECTED;
use crate::gui::{styles, troxide_widget, Message as GuiMessage, Tab};

#[derive(Debug, Clone)]
pub enum Message {
    ThemeSelected(Theme),
    SaveSettings,
}

#[derive(Default)]
pub struct SettingsTab {
    settings_config: Config,
    unsaved_config: Option<Config>,
}

impl SettingsTab {
    pub fn new(settings_config: Config) -> Self {
        Self {
            settings_config,
            unsaved_config: None,
        }
    }

    pub fn get_config_settings(&self) -> &Config {
        if let Some(config) = &self.unsaved_config {
            config
        } else {
            &self.settings_config
        }
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ThemeSelected(theme) => {
                if let Some(config) = &mut self.unsaved_config {
                    config.theme = theme
                } else {
                    let mut unsaved_config = self.settings_config.clone();
                    unsaved_config.theme = theme;
                    self.unsaved_config = Some(unsaved_config);
                }
            }
            Message::SaveSettings => {
                if let Some(config) = self.unsaved_config.take() {
                    self.settings_config = config;
                    save_config(&self.settings_config);
                }
            }
        }
    }
    pub fn view(&self) -> Element<Message, Renderer> {
        let settings_body = column![
            self.appearance_settings_view(),
            self.database_settings_view(),
        ]
        .spacing(5)
        .padding(5);

        let mut save_settings_button = button("Save Settings");

        if let Some(unsaved_settings) = &self.unsaved_config {
            if *unsaved_settings != self.settings_config {
                save_settings_button = save_settings_button.on_press(Message::SaveSettings);
            }
        };

        let save_button_bar = row!(horizontal_space(Length::Fill), save_settings_button).padding(5);

        column![settings_body, vertical_space(Length::Fill), save_button_bar]
            .align_items(Alignment::Center)
            .spacing(5)
            .padding(10)
            .into()
    }

    fn appearance_settings_view(&self) -> Element<Message, Renderer> {
        let content = column![text("Appearance").size(25)].padding(5).spacing(5);

        let theme_text = text("Theme");
        let theme_picklist = pick_list(
            &ALL_THEMES[..],
            Some(if let Some(config) = &self.unsaved_config {
                config.theme.clone()
            } else {
                self.settings_config.theme.clone()
            }),
            Message::ThemeSelected,
        );

        let content = content.push(
            row!(theme_text, horizontal_space(20), theme_picklist)
                .padding(5)
                .spacing(5),
        );

        container(content)
            .style(styles::container_styles::first_class_container_theme())
            .width(1000)
            .into()
    }

    fn database_settings_view(&self) -> Element<Message, Renderer> {
        let import_widget = column![
            text("Import Data").size(22),
            row![
                "Import your series tracking data into Series Troxide",
                horizontal_space(Length::Fill),
                button("Import")
            ]
        ];

        let export_widget = column![
            text("Export Data").size(22),
            row![
                "Export your series tracking data from Series Troxide",
                horizontal_space(Length::Fill),
                button("Export")
            ]
        ];

        let content = column![
            text("Series Troxide Data").size(25),
            import_widget,
            export_widget,
        ]
        .padding(5)
        .spacing(5);

        container(content)
            .style(styles::container_styles::first_class_container_theme())
            .width(1000)
            .into()
    }
}

impl Tab for SettingsTab {
    type Message = GuiMessage;

    fn title(&self) -> String {
        "Settings".to_owned()
    }

    fn tab_label(&self) -> troxide_widget::tabs::TabLabel {
        troxide_widget::tabs::TabLabel::new(self.title(), GEAR_WIDE_CONNECTED)
    }

    fn content(&self) -> Element<'_, Self::Message> {
        self.view().map(GuiMessage::Settings)
    }
}
