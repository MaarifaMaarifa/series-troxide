use crate::core::database;
use iced::{
    widget::{column, text, Column},
    Element, Renderer,
};

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Default)]
pub struct MyShows;

impl MyShows {
    pub fn view(&self) -> Element<Message, Renderer> {
        let title = text("Tracked Shows").size(30);
        let texts: Vec<_> = database::DB
            .get_series_collection()
            .into_iter()
            .map(|series| text(series.get_name()).into())
            .collect();

        column!(title, Column::with_children(texts))
            .padding(5)
            .into()
    }
}
