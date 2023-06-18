pub mod discover_view;
pub mod menu_view;
pub mod my_shows_view;
pub mod search_view;
pub mod series_view;
pub mod settings_view;
pub mod statistics_view;
pub mod watchlist_view;

#[derive(Default)]
pub enum View {
    #[default]
    Search,
    Discover,
    MyShows,
    Statistics,
    Watchlist,
    Series,
    Settings,
}
