use crate::core::settings_config::{self, SETTINGS};
use iced::widget::column;
use iced::{Application, Command};
use std::sync::mpsc;

use series_page::{
    IdentifiableMessage as IdentifiableSeriesMessage, Message as SeriesMessage, Series,
};
use tabs::{Message as TabsControllerMessage, Tab as TabId, TabsController};
use troxide_widget::title_bar::{Message as TitleBarMessage, TitleBar};

pub mod assets;
pub mod helpers;
pub mod series_page;
mod styles;
mod tabs;
mod troxide_widget;

#[derive(Debug, Clone)]
pub enum Message {
    Tabs(TitleBarMessage),
    Series(IdentifiableSeriesMessage),
    TabsController(TabsControllerMessage),
    FontLoaded(Result<(), iced::font::Error>),
}

pub struct TroxideGui {
    active_tab: TabId,
    title_bar: TitleBar,
    series_view_active: bool,
    tabs_controller: TabsController,
    series_view: Option<Series>,
    // TODO: to use iced::subscription
    series_page_receiver: mpsc::Receiver<(Series, Command<SeriesMessage>)>,
}

impl Application for TroxideGui {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let font_command = iced::font::load(assets::get_static_cow_from_asset(
            assets::fonts::NOTOSANS_REGULAR_STATIC,
        ));
        let (sender, receiver) = mpsc::channel();
        let (tabs_controller, tabs_controller_command) = TabsController::new(sender.clone());

        (
            Self {
                active_tab: TabId::Discover,
                title_bar: TitleBar::new(),
                series_view_active: false,
                tabs_controller,
                series_view: None,
                series_page_receiver: receiver,
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
                self.try_series_page_switch(),
            ]),
            Message::Series(message) => {
                let series_page = self
                    .series_view
                    .as_mut()
                    .expect("for series view to send a message it must exist");

                let series_id = series_page.get_series_id();

                let command = if message.matches(series_id) {
                    series_page.update(message.message).map(move |message| {
                        Message::Series(IdentifiableSeriesMessage::new(series_id, message))
                    })
                } else {
                    Command::none()
                };

                Command::batch([command, self.try_series_page_switch()])
            }

            Message::FontLoaded(res) => {
                if res.is_err() {
                    tracing::error!("failed to load font");
                }
                Command::none()
            }
            Message::Tabs(message) => {
                self.title_bar.update(message.clone());
                match message {
                    TitleBarMessage::TabSelected(tab_id) => {
                        self.series_view_active = false;
                        let tab_id: TabId = tab_id.into();
                        self.active_tab = tab_id.clone();
                        self.tabs_controller
                            .switch_to_tab(tab_id)
                            .map(Message::TabsController)
                    }
                }
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message, iced::Renderer<Self::Theme>> {
        let mut tab_view = self.tabs_controller.view().map(Message::TabsController);

        // Hijacking the current tab view when series view is active
        if self.series_view_active {
            let series_view = self.series_view.as_ref().unwrap();
            let series_id = series_view.get_series_id();
            tab_view = series_view.view().map(move |message| {
                Message::Series(IdentifiableSeriesMessage::new(series_id, message))
            });
        }

        column![
            self.title_bar
                .view(&self.tabs_controller.get_labels())
                .map(Message::Tabs),
            tab_view
        ]
        .into()
    }
}

impl TroxideGui {
    fn try_series_page_switch(&mut self) -> Command<Message> {
        use crate::core::caching::{CacheFilePath, CACHER};

        match self.series_page_receiver.try_recv() {
            Ok((series_page, series_page_command)) => {
                let series_id = series_page.get_series_id();
                let series_info = series_page.get_series_main_information();

                // Caching SeriesMainInformation if it is not cached already
                //
                // Since discover poster's SeriesInformation are mostly taken online directly and hence don't
                // use the caching version of api to be obtained. This makes their cache folder lack their
                // SeriesMainInformation cache after being clicked. This cause their folders to be skipped
                // during cache cleaning making the show have same old episode and cast cache forever! unless
                // when it's tracked. So we fix this by caching it if it does not exists when switching to a series page.
                let series_main_info_cache_path =
                    CACHER.get_cache_file_path(CacheFilePath::SeriesMainInformation(series_id));
                if !series_main_info_cache_path.exists() {
                    // TODO: Asynchronously write the cache.
                    let mut folder_path = series_main_info_cache_path.to_owned();
                    folder_path.pop();

                    std::fs::create_dir_all(folder_path)
                        .expect("failed to create series cache folder");

                    std::fs::write(
                        series_main_info_cache_path,
                        serde_json::to_string_pretty(series_info).expect("fail to serialize json"),
                    )
                    .expect("failed to save series main information cache");
                }

                self.series_view = Some(series_page);
                self.series_view_active = true;

                series_page_command.map(move |message| {
                    Message::Series(IdentifiableSeriesMessage::new(series_id, message))
                })
            }
            Err(err) => match err {
                mpsc::TryRecvError::Empty => Command::none(),
                mpsc::TryRecvError::Disconnected => panic!("series page senders disconnected"),
            },
        }
    }
}
