use crate::core::settings_config::{self, SETTINGS};
use iced::widget::column;
use iced::{Application, Command};
use std::sync::mpsc;

use series_page::{Message as SeriesPageControllerMessage, SeriesPageController};
use tabs::{Message as TabsControllerMessage, TabId, TabsController};
use troxide_widget::title_bar::{Message as TitleBarMessage, TitleBar};

pub mod assets;
pub mod helpers;
pub mod message;
pub mod series_page;
mod styles;
mod tabs;
mod troxide_widget;

#[derive(Debug, Clone)]
pub enum Message {
    TitleBar(TitleBarMessage),
    SeriesPageController(SeriesPageControllerMessage),
    TabsController(TabsControllerMessage),
    FontLoaded(Result<(), iced::font::Error>),
}

pub struct TroxideGui<'a> {
    active_tab: TabId,
    title_bar: TitleBar,
    tabs_controller: TabsController<'a>,
    series_page_controller: SeriesPageController<'a>,
}

impl<'a> Application for TroxideGui<'a> {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let font_command = iced::font::load(assets::fonts::NOTOSANS_REGULAR_STATIC);
        let (sender, receiver) = mpsc::channel();
        let (tabs_controller, tabs_controller_command) = TabsController::new(sender.clone());

        (
            Self {
                active_tab: TabId::Discover,
                title_bar: TitleBar::new(),
                tabs_controller,
                series_page_controller: SeriesPageController::new(sender, receiver),
            },
            Command::batch([
                font_command.map(Message::FontLoaded),
                tabs_controller_command.map(Message::TabsController),
            ]),
        )
    }

    fn title(&self) -> String {
        "Series Troxide".to_string()
    }

    fn theme(&self) -> iced::Theme {
        let custom_theme = Box::new(
            match SETTINGS
                .read()
                .unwrap()
                .get_current_settings()
                .appearance
                .theme
            {
                settings_config::Theme::Light => styles::theme::TroxideTheme::Light,
                settings_config::Theme::Dark => styles::theme::TroxideTheme::Dark,
            }
            .get_custom_theme(),
        );
        iced::Theme::Custom(custom_theme)
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        self.tabs_controller
            .subscription()
            .map(Message::TabsController)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::TabsController(message) => Command::batch([
                self.tabs_controller
                    .update(message)
                    .map(Message::TabsController),
                self.series_page_controller
                    .try_series_page_switch()
                    .map(Message::SeriesPageController),
            ]),
            Message::SeriesPageController(message) => self
                .series_page_controller
                .update(message)
                .map(Message::SeriesPageController),
            Message::FontLoaded(res) => {
                if res.is_err() {
                    tracing::error!("failed to load font");
                }
                Command::none()
            }
            Message::TitleBar(message) => {
                self.title_bar.update(message.clone());
                match message {
                    TitleBarMessage::TabSelected(tab_id) => {
                        self.series_page_controller.clear_all_pages();
                        let tab_id: TabId = tab_id.into();
                        self.active_tab = tab_id;
                        self.tabs_controller
                            .switch_to_tab(tab_id)
                            .map(Message::TabsController)
                    }
                    TitleBarMessage::BackButtonPressed => {
                        let command = self
                            .series_page_controller
                            .go_previous()
                            .map(Message::SeriesPageController);
                        let scrollers_offset_restore_command =
                            if !self.series_page_controller.has_a_series_page() {
                                self.tabs_controller
                                    .update_scrollables_offsets()
                                    .map(Message::TabsController)
                            } else {
                                Command::none()
                            };

                        Command::batch([command, scrollers_offset_restore_command])
                    }
                }
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message, iced::Renderer<Self::Theme>> {
        let view = if let Some(series_page_view) = self.series_page_controller.view() {
            series_page_view.map(Message::SeriesPageController)
        } else {
            self.tabs_controller.view().map(Message::TabsController)
        };

        column![
            self.title_bar
                .view(
                    &self.tabs_controller.get_labels(),
                    self.series_page_controller.has_a_series_page()
                )
                .map(Message::TitleBar),
            view
        ]
        .into()
    }
}
