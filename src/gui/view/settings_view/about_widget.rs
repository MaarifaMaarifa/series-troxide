use crate::gui::styles;

use super::Message;
use iced::{
    widget::{column, container, text},
    Element, Renderer,
};
use iced_aw::Grid;

pub fn about_widget() -> Element<'static, Message, Renderer> {
    let mut grid = Grid::with_columns(2);

    grid.insert(text("Program"));
    grid.insert(text(built_info::PKG_NAME));

    grid.insert(text("Author"));
    grid.insert(text(built_info::PKG_AUTHORS));

    grid.insert(text("Version"));
    grid.insert(text(built_info::PKG_VERSION));

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

    let content = column![
        text("About")
            .style(styles::text_styles::purple_text_theme())
            .size(25),
        grid
    ]
    .spacing(10);

    container(content)
        .style(styles::container_styles::first_class_container_theme())
        .width(1000)
        .padding(5)
        .into()
}

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
