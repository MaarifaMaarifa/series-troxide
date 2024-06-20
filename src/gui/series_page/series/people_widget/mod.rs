use iced::widget::{column, container, text, toggler};

use iced::{Command, Element, Length, Renderer};

use cast_widget::{CastWidget, Message as CastWidgetMessage};
use crew_widget::{CrewWidget, Message as CrewWidgetMessage};

use crate::gui::styles;

mod cast_widget;
mod crew_widget;

#[derive(Debug, Clone)]
pub enum Message {
    PeopleToggled(bool),
    CastWidget(CastWidgetMessage),
    CrewWidget(CrewWidgetMessage),
}

enum People {
    Crew,
    Cast,
}

impl std::fmt::Display for People {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            People::Crew => "Crew",
            People::Cast => "Cast",
        };

        write!(f, "{str}")
    }
}

pub struct PeopleWidget {
    series_id: u32,
    current_people: People,
    cast_widget: CastWidget,
    crew_widget: Option<CrewWidget>,
    toggled: bool,
}

impl PeopleWidget {
    pub fn new(series_id: u32) -> (Self, Command<Message>) {
        let (cast_widget, cast_widget_command) = CastWidget::new(series_id);
        (
            Self {
                series_id,
                current_people: People::Cast,
                cast_widget,
                crew_widget: None,
                toggled: false,
            },
            cast_widget_command.map(Message::CastWidget),
        )
    }

    fn fetch_crew(&mut self) -> Command<Message> {
        let (crew_widget, crew_widget_command) = CrewWidget::new(self.series_id);
        self.crew_widget = Some(crew_widget);

        crew_widget_command.map(Message::CrewWidget)
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::CastWidget(message) => {
                self.cast_widget.update(message).map(Message::CastWidget)
            }
            Message::CrewWidget(message) => {
                if let Some(ref mut crew_widget) = self.crew_widget {
                    crew_widget.update(message).map(Message::CrewWidget)
                } else {
                    Command::none()
                }
            }
            Message::PeopleToggled(toggled) => {
                self.toggled = toggled;
                match self.current_people {
                    People::Crew => {
                        self.current_people = People::Cast;
                        Command::none()
                    }
                    People::Cast => {
                        self.current_people = People::Crew;
                        if self.crew_widget.is_none() {
                            self.fetch_crew()
                        } else {
                            Command::none()
                        }
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let current_people = match self.current_people {
            People::Cast => self
                .cast_widget
                .view()
                .map(|view| view.map(Message::CastWidget)),
            People::Crew => self
                .crew_widget
                .as_ref()
                .expect("crew should be set already")
                .view()
                .map(|view| view.map(Message::CrewWidget)),
        };

        let people_toggler = toggler(
            Some(self.current_people.to_string()),
            self.toggled,
            Message::PeopleToggled,
        )
        .spacing(5)
        .text_size(21)
        .style(styles::toggler_styles::always_colored_toggler_theme())
        .width(Length::Shrink);

        column![
            people_toggler,
            current_people.unwrap_or(self.people_not_found()),
        ]
        .padding(5)
        .into()
    }

    fn people_not_found(&self) -> Element<'_, Message, Renderer> {
        container(text(format!("No {} Found!", self.current_people)))
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(100)
            .into()
    }
}
