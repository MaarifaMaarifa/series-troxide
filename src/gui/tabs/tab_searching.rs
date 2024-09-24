//! Module responsible with fuzzy searching of Series in a particular tab

use iced::widget::{column, container, text, text_input, Container};
use iced::{Element, Length};

use crate::core::api::tv_maze::series_information::SeriesMainInformation;

/// A trait enabling searching of particular matching `Series`' against a search term in a tab
pub trait Searchable {
    /// Retrieves all the `SeriesMainInformation`s available on a particular tab
    ///
    /// After retrieving this collection, a search will be performed in the collection
    /// against a particular search term.
    fn get_series_information_collection(&self) -> Vec<&SeriesMainInformation>;

    /// Provides a mutable reference to the tab's matched series ids collection so that
    /// we can update them when the search term changes
    ///
    /// The mutable reference to the tab's matched ids is `Optional`. This is to provide a clear
    /// indication whether no term has be provided to search (`None`), or a term has been provided
    /// and the reference has `Some` matched ids based upon that search term
    fn matches_id_collection(&mut self) -> &mut Option<Vec<u32>>;

    /// Checks if a particular `SeriesMainInformation` id is among the matched ids after a search
    fn is_matched_id(&self, matched_ids: &[u32], id: u32) -> bool {
        matched_ids.iter().any(|matched_id| *matched_id == id)
    }

    /// Updates the matched series ids of the tab based on the search term
    fn update_matches(&mut self, search_term: &str) {
        let new_matches_ids = Self::search_for_matches(
            search_term,
            self.get_series_information_collection().as_slice(),
        );

        let matches_id_collection = self.matches_id_collection();

        // If the search term is empty, we set the matched_id_collection to None
        // so as to clearly indicate that situation.
        if search_term.is_empty() {
            *matches_id_collection = None;
            return;
        }

        *matches_id_collection = Some(new_matches_ids);
    }

    /// Search for `SeriesMainInformation` matches based on a particular search term returning
    /// the Series Ids of the matched ones
    fn search_for_matches(
        search_term: &str,
        search_collection: &[&SeriesMainInformation],
    ) -> Vec<u32> {
        use fuzzy_matcher::skim::SkimMatcherV2;
        use fuzzy_matcher::FuzzyMatcher;

        let matcher = SkimMatcherV2::default();
        search_collection
            .iter()
            .filter(|series_info| {
                matcher
                    .fuzzy_match(&series_info.name, search_term)
                    .is_some()
            })
            .map(|series_info| series_info.id)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SearchTermChanged(String),
}

pub struct Searcher {
    placeholder: String,
    current_search_term: String,
}

impl Searcher {
    pub fn new(placeholder: String) -> Self {
        Self {
            placeholder,
            current_search_term: String::new(),
        }
    }

    pub fn current_search_term(&self) -> &str {
        &self.current_search_term
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SearchTermChanged(new_search_term) => {
                self.current_search_term = new_search_term;
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let search_bar = column!(text_input(&self.placeholder, &self.current_search_term)
            .width(300)
            .on_input(Message::SearchTermChanged))
        .width(Length::Fill)
        .align_x(iced::Alignment::Center);

        search_bar.into()
    }
}

/// A helper function that produces an `Element` that indicates absence of series posters based
/// on the supplied absence reason
pub fn unavailable_posters<Message: 'static>(absence_reason: &str) -> Container<Message> {
    container(text(absence_reason).align_x(iced::alignment::Horizontal::Center))
        .center_x(Length::Shrink)
        .center_y(Length::Shrink)
}
