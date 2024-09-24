use iced::widget::scrollable::{Direction, Scrollbar};

pub fn vertical_direction() -> Direction {
    let scroll_bar = Scrollbar::new().width(5).scroller_width(5);
    Direction::Vertical(scroll_bar)
}
