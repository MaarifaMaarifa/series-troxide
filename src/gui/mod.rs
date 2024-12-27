use crate::core::settings_config::{self, SETTINGS};
use iced::window::Id;
use iced::Task;
use iced::{widget::column, window};
use std::sync::{mpsc, Arc};

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

impl TroxideGui<'_> {
    pub fn new() -> (Self, iced::Task<Message>) {
        let noto_font_command = iced::font::load(assets::fonts::NOTOSANS_REGULAR_STATIC);

        let icon_change_task =
            window::icon::from_file_data(crate::gui::assets::logos::IMG_LOGO, None)
                .map(|icon| window::change_icon(Id::unique(), icon))
                .unwrap_or(Task::none());

        // let bootstrap_font_command = iced::font::load(iced_aw::BOOTSTRAP_FONT_BYTES);

        let (sender, receiver) = mpsc::channel();
        let (tabs_controller, tabs_controller_command) = TabsController::new(sender.clone());

        (
            Self {
                active_tab: TabId::Discover,
                title_bar: TitleBar::new(),
                tabs_controller,
                series_page_controller: SeriesPageController::new(sender, receiver),
            },
            Task::batch([
                noto_font_command.map(Message::FontLoaded),
                // bootstrap_font_command.map(Message::FontLoaded),
                tabs_controller_command.map(Message::TabsController),
                icon_change_task,
            ]),
        )
    }

    pub fn title(&self) -> String {
        let mut program_title = String::from("Series Troxide - ");

        if let Some(series_page_name) = self.series_page_controller.get_series_page_name() {
            program_title.push_str(series_page_name)
        } else {
            program_title.push_str(&self.active_tab.to_string())
        }

        program_title
    }

    pub fn theme(&self) -> iced::Theme {
        let custom_theme = Arc::new(
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

    pub fn scale_factor(&self) -> f64 {
        let scale = SETTINGS
            .read()
            .unwrap()
            .get_current_settings()
            .appearance
            .scale
            .to_owned();
        Into::<f64>::into(scale) / 100.0
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        self.tabs_controller
            .subscription()
            .map(Message::TabsController)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabsController(message) => Task::batch([
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
                Task::none()
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
                                Task::none()
                            };

                        Task::batch([command, scrollers_offset_restore_command])
                    }
                }
            }
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
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
