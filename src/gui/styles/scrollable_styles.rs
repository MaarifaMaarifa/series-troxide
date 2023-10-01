use iced::widget::scrollable::{Direction, Properties};

pub fn vertical_direction() -> Direction {
    Direction::Vertical(Properties::new().width(5).scroller_width(5))
}
