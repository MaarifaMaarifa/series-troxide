use crate::core::api::crates::{get_program_info, CrateInformation};
use crate::gui::assets::icons::{ARROW_REPEAT, CUP_HOT_FILL, SERIES_TROXIDE_ICON};
use crate::gui::styles;

use iced::font::Weight;
use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, mouse_area, row, svg, text,
    vertical_space, Space,
};
use iced::{Command, Element, Font, Length, Renderer};
use iced_aw::Grid;
use tracing::error;

#[derive(Debug, Clone)]
pub enum Message {
    Repository,
    TvMaze,
    BootstrapIcons,
    Iced,
    CrateInfoLoaded(Result<CrateInformation, String>),
    RecheckUpdate,
    Coffee,
}

pub struct About {
    crate_information: Option<Result<CrateInformation, String>>,
}

impl About {
    pub fn new() -> (Self, Command<Message>) {
        (
            Self {
                crate_information: None,
            },
            Self::check_update(),
        )
    }
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Repository => {
                webbrowser::open(built_info::PKG_REPOSITORY)
                    .unwrap_or_else(|err| error!("failed to open repository site: {}", err));
            }
            Message::TvMaze => {
                webbrowser::open("https://www.tvmaze.com/")
                    .unwrap_or_else(|err| error!("failed to open TVmaze site: {}", err));
            }
            Message::BootstrapIcons => {
                webbrowser::open("https://icons.getbootstrap.com/")
                    .unwrap_or_else(|err| error!("failed to open bootstrap icons site: {}", err));
            }
            Message::Iced => {
                webbrowser::open("https://iced.rs/")
                    .unwrap_or_else(|err| error!("failed to open Iced site: {}", err));
            }
            Message::CrateInfoLoaded(info_result) => self.crate_information = Some(info_result),
            Message::RecheckUpdate => {
                self.crate_information = None;
                return Self::check_update();
            }
            Message::Coffee => {
                webbrowser::open("https://www.patreon.com/MaarifaMaarifa")
                    .unwrap_or_else(|err| error!("failed to open patreon site: {}", err));
            }
        };

        Command::none()
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let content = column![
            text("About")
                .style(styles::text_styles::accent_color_theme())
                .size(21),
            update_widget(self),
            info_widget(),
            social_buttons(),
            horizontal_rule(1),
            vertical_space(5),
            text("Credits").size(18),
            credit_widget(),
        ]
        .spacing(10);

        container(content)
            .style(styles::container_styles::first_class_container_rounded_theme())
            .width(1000)
            .padding(5)
            .into()
    }

    fn check_update() -> Command<Message> {
        Command::perform(get_program_info(), |result| {
            Message::CrateInfoLoaded(result.map_err(|err| err.to_string()))
        })
    }
}

fn update_widget(about: &About) -> Element<'static, Message, Renderer> {
    let series_troxide_icon_handle = svg::Handle::from_memory(SERIES_TROXIDE_ICON);
    let icon = svg(series_troxide_icon_handle).width(40);
    let program_info = column![
        text("Series Troxide")
            .font(Font {
                weight: Weight::Bold,
                ..Default::default()
            })
            .size(18)
            .style(styles::text_styles::accent_color_theme()),
        text(built_info::PKG_DESCRIPTION),
    ];

    let program_main_info = row![icon, program_info].spacing(5);

    let latest_version_and_container_style: (&str, iced::theme::Container, Option<bool>) =
        match &about.crate_information {
            Some(crate_info_result) => match crate_info_result {
                Ok(crate_info) => {
                    let container_style = if crate_info.package.is_up_to_date() {
                        styles::container_styles::success_container_theme()
                    } else {
                        styles::container_styles::failure_container_theme()
                    };
                    (
                        crate_info.package.newest_version(),
                        container_style,
                        Some(crate_info.package.is_up_to_date()),
                    )
                }
                Err(_) => (
                    "unavailable",
                    styles::container_styles::failure_container_theme(),
                    None,
                ),
            },
            None => (
                "loading...",
                styles::container_styles::loading_container_theme(),
                None,
            ),
        };

    let update_status: Element<'_, Message, Renderer> =
        if let Some(is_up_to_date) = latest_version_and_container_style.2 {
            if is_up_to_date {
                text("Up to date")
            } else {
                text("Out of date")
            }
            .font(Font {
                weight: Weight::Bold,
                ..Default::default()
            })
            .into()
        } else {
            Space::new(0, 0).into()
        };

    let refresh_icon_handle = svg::Handle::from_memory(ARROW_REPEAT);
    let refresh_icon = svg(refresh_icon_handle).style(styles::svg_styles::colored_svg_theme());

    let refresh_button = button(refresh_icon)
        .style(styles::button_styles::transparent_button_theme())
        .on_press(Message::RecheckUpdate);

    let version_information = container(row![
        column![
            update_status,
            row![
                text("Latest version: ").style(styles::text_styles::accent_color_theme()),
                text(latest_version_and_container_style.0)
            ],
            row![
                text("Program version: ").style(styles::text_styles::accent_color_theme()),
                text(env!("CARGO_PKG_VERSION"))
            ],
        ],
        horizontal_space(Length::Fill),
        refresh_button
    ])
    .style(latest_version_and_container_style.1)
    .width(300)
    .padding(10);

    column![program_main_info, version_information]
        .spacing(5)
        .into()
}

fn info_widget() -> Element<'static, Message, Renderer> {
    let mut grid = Grid::with_columns(2);

    grid.insert(text("Author"));
    grid.insert(text(built_info::PKG_AUTHORS));

    grid.insert(text("License"));
    grid.insert(text(built_info::PKG_LICENSE));

    let repository = mouse_area(
        text(built_info::PKG_REPOSITORY).style(styles::text_styles::accent_color_theme()),
    )
    .on_press(Message::Repository);
    grid.insert(text("Repository"));
    grid.insert(repository);

    if !built_info::GIT_DIRTY.unwrap_or(false) {
        if let Some(commit_hash) = built_info::GIT_COMMIT_HASH {
            grid.insert(text("Commit Hash"));
            grid.insert(text(commit_hash));
        }
    }

    grid.insert(text("Build Time"));
    grid.insert(text(built_info::BUILT_TIME_UTC));

    grid.insert(text("Rust Version    ")); // adding some space in grid since it is the longest text
    grid.insert(text(built_info::RUSTC_VERSION));

    grid.into()
}

fn social_buttons() -> Element<'static, Message, Renderer> {
    let coffee_icon_handle = svg::Handle::from_memory(CUP_HOT_FILL);
    let coffee_icon = svg(coffee_icon_handle)
        .style(styles::svg_styles::colored_svg_theme())
        .height(30)
        .width(30);
    let coffee_button = button(coffee_icon)
        .style(styles::button_styles::transparent_button_theme())
        .on_press(Message::Coffee);

    container(coffee_button)
        .width(Length::Fill)
        .center_x()
        .center_y()
        .into()
}

fn credit_widget() -> Element<'static, Message, Renderer> {
    let go_to_site_text = text("here")
        .size(11)
        .style(styles::text_styles::accent_color_theme());

    let tv_maze = row![
        text("- The API used has been provided by TVmaze, you can check out the site ").size(11),
        mouse_area(go_to_site_text.clone()).on_press(Message::TvMaze)
    ];
    let bootstrap_icons = row![
        text("- The Icons used have been provided by boostrap icons, you can check out the site ")
            .size(11),
        mouse_area(go_to_site_text.clone()).on_press(Message::BootstrapIcons)
    ];
    let iced =
        row![
        text("- The Graphical User Interface has been made using Iced, you can check out the site ")
            .size(11),
        mouse_area(go_to_site_text).on_press(Message::Iced)
    ];
    column![tv_maze, bootstrap_icons, iced].into()
}

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
