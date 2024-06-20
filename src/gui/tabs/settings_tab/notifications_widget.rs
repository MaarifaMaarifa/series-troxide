use iced::widget::{column, container, text, toggler};
use iced::{Element, Length, Renderer};
use iced_aw::NumberInput;

use crate::core::settings_config::SETTINGS;
use crate::gui::styles;

#[derive(Debug, Clone)]
pub enum Message {
    TimeChanged(u32),
    NotifyWhenOutdated(bool),
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
            Message::NotifyWhenOutdated(notify_when_outdate) => {
                SETTINGS
                    .write()
                    .unwrap()
                    .change_settings()
                    .notifications
                    .notify_when_outdated = notify_when_outdate;
            }
        }
    }
    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let notification_settings = SETTINGS
            .read()
            .unwrap()
            .get_current_settings()
            .notifications
            .clone();

        let current_time_to_notify = notification_settings.time_to_notify;
        let notify_when_outdated = notification_settings.notify_when_outdated;

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
                .width(Length::Fixed(200.0));

        let when_to_notify = column![notifications_info, time_to_notify,].spacing(5);

        let notify_when_outdated = toggler(
            Some("Notify when outdated".to_string()),
            notify_when_outdated,
            Message::NotifyWhenOutdated,
        )
        .spacing(10)
        .width(Length::Shrink);

        let content = column![
            text("Notifications")
                .style(styles::text_styles::accent_color_theme())
                .size(21),
            when_to_notify,
            notify_when_outdated,
        ]
        .spacing(10);

        container(content)
            .style(styles::container_styles::first_class_container_rounded_theme())
            .padding(5)
            .width(1000)
            .into()
    }
}
