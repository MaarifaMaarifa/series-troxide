use iced::widget::{button, column, container, horizontal_space, row, text, Text};
use iced::{Command, Element, Length, Renderer};

use crate::gui::styles;

#[derive(Debug, Clone)]
pub enum Message {
    ImportDatabasePressed,
    ExportDatabasePressed,
    ImportTimeoutComplete,
    ExportTimeoutComplete,
}

#[derive(Default)]
pub struct Database {
    import_status: Option<anyhow::Result<()>>,
    export_status: Option<anyhow::Result<()>>,
}

impl Database {
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ImportDatabasePressed => {
                self.import_status = Some(database_transfer::import());

                Command::perform(super::status_timeout(), |_| Message::ImportTimeoutComplete)
            }
            Message::ExportDatabasePressed => {
                self.export_status = Some(database_transfer::export());
                Command::perform(super::status_timeout(), |_| Message::ExportTimeoutComplete)
            }
            Message::ImportTimeoutComplete => {
                self.import_status = None;
                Command::none()
            }
            Message::ExportTimeoutComplete => {
                self.export_status = None;
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let import_widget = column![
            text("Import Data").size(18),
            row![
                text("Import your series tracking data into Series Troxide").size(11),
                horizontal_space(Length::Fill),
                get_status_text(self.import_status.as_ref()),
                button("Import").on_press(Message::ImportDatabasePressed)
            ]
            .spacing(5)
        ];

        let export_widget = column![
            text("Export Data").size(18),
            row![
                text("Export your series tracking data from Series Troxide").size(11),
                horizontal_space(Length::Fill),
                get_status_text(self.export_status.as_ref()),
                button("Export").on_press(Message::ExportDatabasePressed)
            ]
            .spacing(5)
        ];

        let content = column![
            text("Series Troxide Data")
                .size(21)
                .style(styles::text_styles::purple_text_theme()),
            import_widget,
            export_widget,
        ]
        .padding(5)
        .spacing(5);

        container(content)
            .style(styles::container_styles::first_class_container_rounded_theme())
            .width(1000)
            .into()
    }
}

fn get_status_text(status: Option<&anyhow::Result<()>>) -> Text {
    if let Some(res) = status {
        if let Err(err) = res {
            text(err.to_string()).style(styles::text_styles::red_text_theme())
        } else {
            text("Done!").style(styles::text_styles::green_text_theme())
        }
    } else {
        text("")
    }
}

mod database_transfer {
    use directories::UserDirs;
    use std::path;

    use crate::core::database::database_transfer;
    use rfd::FileDialog;

    pub fn export() -> anyhow::Result<()> {
        let chosen_path = FileDialog::new()
            .set_directory(get_home_directory()?)
            .save_file();

        if let Some(mut chosen_path) = chosen_path {
            let file_name = chosen_path.file_name().map(std::ffi::OsString::from);
            chosen_path.pop();
            database_transfer::write_database_to_path(&chosen_path, file_name.as_deref())?;
        }

        Ok(())
    }

    pub fn import() -> anyhow::Result<()> {
        let chosen_path = FileDialog::new()
            .set_directory(get_home_directory()?)
            .pick_file();

        if let Some(chosen_path) = chosen_path {
            database_transfer::read_database_from_path(path::Path::new(&chosen_path))?;
        }

        Ok(())
    }

    pub fn get_home_directory() -> anyhow::Result<path::PathBuf> {
        let user_dirs = UserDirs::new().ok_or(anyhow::anyhow!("could not get user directory"))?;
        Ok(user_dirs.home_dir().to_path_buf())
    }
}
