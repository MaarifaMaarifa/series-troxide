use iced::widget::{
    button, column, container, horizontal_space, progress_bar, row, text, vertical_space, Space,
    Text,
};
use iced::{Command, Element, Length, Renderer};

use crate::core::database::database_transfer::TransferData;
use crate::core::database::DB;

use crate::gui::styles;

mod trakt_integration;

#[derive(Debug, Clone)]
pub enum Message {
    ImportDatabasePressed,
    ExportDatabasePressed,
    ImportTimeoutComplete,
    ExportTimeoutComplete,
    ImportCachingEvent(full_caching::Event),
    TraktIntegration(trakt_integration::Message),
}

pub struct Database {
    import_status: Option<anyhow::Result<()>>,
    export_status: Option<anyhow::Result<()>>,
    import_progress: (usize, usize),
    importing: bool,
    transfer_data: Option<TransferData>,
    sender: Option<iced::futures::channel::mpsc::Sender<full_caching::Input>>,
    trakt_widget: trakt_integration::TraktIntegration,
}

impl Database {
    pub fn new() -> Self {
        Self {
            import_status: None,
            export_status: None,
            import_progress: (0, 0),
            importing: false,
            transfer_data: None,
            sender: None,
            trakt_widget: trakt_integration::TraktIntegration::new(),
        }
    }
    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch([
            full_caching::import_data_cacher().map(Message::ImportCachingEvent),
            self.trakt_widget
                .subscription()
                .map(Message::TraktIntegration),
        ])
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ImportDatabasePressed => match database_transfer::import_transfer_data() {
                Ok(transfer_data) => {
                    if let Some(transfer_data) = transfer_data {
                        let ids: Vec<u32> = transfer_data
                            .get_series()
                            .iter()
                            .map(|series| series.id())
                            .collect();

                        self.transfer_data = Some(transfer_data);

                        self.import_progress.1 = ids.len();
                        self.importing = true;
                        self.sender
                            .as_mut()
                            .expect("there should be a work sender at this point")
                            .try_send(full_caching::Input::CacheSeries(ids))
                            .expect("full caching receiver disconnected");
                    }
                    Command::none()
                }
                Err(err) => {
                    self.import_status = Some(Err(anyhow::anyhow!(err)));
                    Command::perform(status_timeout(), |_| Message::ImportTimeoutComplete)
                }
            },
            Message::ExportDatabasePressed => {
                self.export_status = Some(database_transfer::export());
                Command::perform(status_timeout(), |_| Message::ExportTimeoutComplete)
            }
            Message::ImportTimeoutComplete => {
                self.import_status = None;
                Command::none()
            }
            Message::ExportTimeoutComplete => {
                self.export_status = None;
                Command::none()
            }
            Message::ImportCachingEvent(event) => {
                match event {
                    full_caching::Event::Ready(work_sender) => {
                        self.sender = Some(work_sender);
                    }
                    full_caching::Event::WorkFinished => {
                        self.import_progress = (0, 0);
                        self.importing = false;

                        let data = self
                            .transfer_data
                            .as_ref()
                            .expect("there should be transfer data at this point");

                        DB.import(data);

                        self.import_status = Some(Ok(()));
                        return Command::perform(status_timeout(), |_| {
                            Message::ImportTimeoutComplete
                        });
                    }
                    full_caching::Event::Progressing => {
                        self.import_progress.0 += 1;
                    }
                }
                Command::none()
            }
            Message::TraktIntegration(message) => self
                .trakt_widget
                .update(message)
                .map(Message::TraktIntegration),
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        let import_progress: Element<'_, Message, Renderer> = if self.importing {
            row![
                text("caching the import").size(11),
                progress_bar(
                    0.0..=self.import_progress.1 as f32,
                    self.import_progress.0 as f32,
                )
                .height(13),
                text(format!(
                    "{} / {}",
                    self.import_progress.0, self.import_progress.1
                ))
                .size(11)
            ]
            .spacing(10)
            .into()
        } else {
            Space::new(0, 0).into()
        };

        let import_widget = column![
            text("Import Data").size(18),
            row![
                text("Import your series tracking data into Series Troxide").size(11),
                horizontal_space(Length::Fill),
                get_status_text(self.import_status.as_ref()),
                {
                    let mut button = button("Import");
                    if !self.importing {
                        button = button.on_press(Message::ImportDatabasePressed);
                    }
                    button
                },
            ]
            .spacing(5),
            vertical_space(5),
            import_progress,
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

        let series_troxide_data = column![
            text("Series Troxide Data").size(18),
            import_widget,
            export_widget,
        ]
        .spacing(5);

        let trakt_data = column![
            text("Trakt Data").size(18),
            self.trakt_widget.view().map(Message::TraktIntegration)
        ]
        .spacing(5);

        let content = column![
            text("Series Tracking Data")
                .size(21)
                .style(styles::text_styles::accent_color_theme()),
            series_troxide_data,
            trakt_data
        ]
        .padding(5);

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

/// A function that sleeps for 3 seconds designed to provide timeout
/// for status texts in widgets like the database and caching widget.
async fn status_timeout() {
    tokio::time::sleep(std::time::Duration::from_secs(3)).await
}

mod full_caching {
    use crate::core::caching::series_info_and_episode_list::SeriesInfoAndEpisodeList;

    use iced::futures::channel::mpsc;
    use iced::futures::sink::SinkExt;
    use iced::subscription::{self, Subscription};

    #[derive(Debug, Clone)]
    pub enum Event {
        Ready(mpsc::Sender<Input>),
        WorkFinished,
        Progressing,
    }

    #[derive(Debug, Clone)]
    pub enum Input {
        CacheSeries(Vec<u32>),
    }

    enum State {
        Starting,
        Ready(mpsc::Receiver<Input>),
    }

    pub fn import_data_cacher() -> Subscription<Event> {
        subscription::channel("settings-data-importer", 100, |mut output| async move {
            let mut state = State::Starting;

            loop {
                match &mut state {
                    State::Starting => {
                        let (sender, receiver) = mpsc::channel(100);

                        output
                            .send(Event::Ready(sender))
                            .await
                            .expect("failed to send input sender");

                        state = State::Ready(receiver);
                    }
                    State::Ready(receiver) => {
                        use iced::futures::StreamExt;

                        let input = receiver.select_next_some().await;

                        #[allow(irrefutable_let_patterns)]
                        if let Input::CacheSeries(ids) = input {
                            let (series_info_and_episode_list, mut progress_receiver) =
                                SeriesInfoAndEpisodeList::new(ids);

                            let handle = tokio::spawn(async move {
                                let series_info_and_episode_list = series_info_and_episode_list;
                                series_info_and_episode_list.run_full_caching(true).await
                            });

                            while let Some(result) = progress_receiver.recv().await {
                                if let Err(err) = result {
                                    tracing::error!("progress error: {}", err);
                                }

                                output
                                    .send(Event::Progressing)
                                    .await
                                    .expect("failed to send the progress");
                            }
                            handle
                                .await
                                .expect("failed to await progress handle")
                                .unwrap();
                            output
                                .send(Event::WorkFinished)
                                .await
                                .expect("failed to send work completion");
                            state = State::Starting;
                        }
                    }
                }
            }
        })
    }
}

mod database_transfer {
    use directories::UserDirs;
    use std::path;

    use crate::core::database::database_transfer::TransferData;
    use rfd::FileDialog;

    pub fn export() -> anyhow::Result<()> {
        let chosen_path = FileDialog::new()
            .set_directory(get_home_directory()?)
            .save_file();

        if let Some(chosen_path) = chosen_path {
            TransferData::export_from_db(chosen_path)?;
        }

        Ok(())
    }

    pub fn import_transfer_data() -> anyhow::Result<Option<TransferData>> {
        let chosen_path = FileDialog::new()
            .set_directory(get_home_directory()?)
            .pick_file();

        if let Some(chosen_path) = chosen_path {
            let data = TransferData::import(chosen_path)?;
            return Ok(Some(data));
        }

        Ok(None)
    }

    pub fn get_home_directory() -> anyhow::Result<path::PathBuf> {
        let user_dirs = UserDirs::new().ok_or(anyhow::anyhow!("could not get user directory"))?;
        Ok(user_dirs.home_dir().to_path_buf())
    }
}
