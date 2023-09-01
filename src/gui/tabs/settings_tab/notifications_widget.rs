use iced::widget::{column, container, horizontal_space, row, text};
use iced::{Element, Length, Renderer};
use iced_aw::NumberInput;

use crate::core::settings_config::SETTINGS;
use crate::gui::styles;

#[derive(Debug, Clone)]
pub enum Message {
    TimeChanged(u32),
}

#[derive(Default)]
pub struct Notifications;

impl Notifications {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::TimeChanged(new_time) => {
                SETTINGS
                    .write()
                    .unwrap()
                    .change_settings()
                    .notifications
                    .time_to_notify = new_time;
            }
        }
    }
    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let current_time_to_notify = SETTINGS
            .read()
            .unwrap()
            .get_current_settings()
            .notifications
            .time_to_notify;

        let notifications_info = column![
            text("When to notify"),
            text(format!(
                "System notification will be sent {} minutes before an episode release",
                current_time_to_notify
            ))
            .size(11)
        ];

        let time_to_notify =
            NumberInput::new(current_time_to_notify, u32::MAX, Message::TimeChanged)
                .width(Length::FillPortion(2));

        let content = row![
            notifications_info,
            horizontal_space(Length::Fill),
            time_to_notify,
        ];

        let content = column![
            text("Notifications")
                .style(styles::text_styles::purple_text_theme())
                .size(21),
            content,
        ]
        .spacing(5);

        container(content)
            .style(styles::container_styles::first_class_container_rounded_theme())
            .padding(5)
            .width(1000)
            .into()
    }
}
