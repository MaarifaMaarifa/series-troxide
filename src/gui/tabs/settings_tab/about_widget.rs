use crate::gui::styles;

use iced::widget::{column, container, horizontal_rule, mouse_area, row, text, vertical_space};
use iced::{Element, Renderer};
use iced_aw::Grid;
use tracing::error;

#[derive(Debug, Clone)]
pub enum Message {
    Repository,
    TvMaze,
    BootstrapIcons,
    Iced,
}

#[derive(Default)]
pub struct About;

impl About {
    pub fn update(&mut self, message: Message) {
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
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let content = column![
            text("About")
                .style(styles::text_styles::accent_color_theme())
                .size(21),
            info_widget(),
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
}

fn info_widget() -> Element<'static, Message, Renderer> {
    let mut grid = Grid::with_columns(2);

    grid.insert(text("Program"));
    grid.insert(text(built_info::PKG_NAME));

    grid.insert(text("Author"));
    grid.insert(text(built_info::PKG_AUTHORS));

    grid.insert(text("Version"));
    grid.insert(text(built_info::PKG_VERSION));

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
