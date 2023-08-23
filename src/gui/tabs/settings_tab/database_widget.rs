use iced::widget::{
    button, column, container, horizontal_space, progress_bar, row, text, vertical_space, Space,
    Text,
};
use iced::{Command, Element, Length, Renderer};

use crate::core::database::get_ids_from_keys_values_vec;
use crate::core::database::{self, DB};

use crate::gui::styles;

#[derive(Debug, Clone)]
pub enum Message {
    ImportDatabasePressed,
    ExportDatabasePressed,
    ImportTimeoutComplete,
    ExportTimeoutComplete,
    ImportCachingEvent(full_caching::Event),
}

#[derive(Default)]
pub struct Database {
    import_status: Option<anyhow::Result<()>>,
    export_status: Option<anyhow::Result<()>>,
    import_progress: (usize, usize),
    importing: bool,
    keys_values_vec: Option<database::KeysValuesVec>,
    sender: Option<iced::futures::channel::mpsc::Sender<full_caching::Input>>,
}

impl Database {
    pub fn subscription(&self) -> iced::Subscription<Message> {
        full_caching::import_data_cacher().map(Message::ImportCachingEvent)
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ImportDatabasePressed => match database_transfer::import_keys_values_vec() {
                Ok(keys_values_vec) => {
                    if let Some(keys_values_vec) = keys_values_vec {
                        self.keys_values_vec = Some(keys_values_vec.clone());
                        let ids: Vec<u32> = get_ids_from_keys_values_vec(keys_values_vec)
                            .into_iter()
                            .map(|series_id| {
                                series_id
                                    .parse()
                                    .expect("series id should be parsable to u32")
                            })
                            .collect();
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
                    Command::perform(super::status_timeout(), |_| Message::ImportTimeoutComplete)
                }
            },
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
            Message::ImportCachingEvent(event) => {
                match event {
                    full_caching::Event::Ready(work_sender) => {
                        self.sender = Some(work_sender);
                    }
                    full_caching::Event::WorkFinished => {
                        self.import_progress = (0, 0);
                        self.importing = false;
                        DB.import_keys_value_vec(
                            self.keys_values_vec
                                .take()
                                .expect("there should be keys_values_vec at this point"),
                        )
                        .expect("failed to import keys_values_vec");

                        self.import_status = Some(Ok(()));
                        return Command::perform(super::status_timeout(), |_| {
                            Message::ImportTimeoutComplete
                        });
                    }
                    full_caching::Event::Progressing => {
                        self.import_progress.0 += 1;
                    }
                }
                Command::none()
            }
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
                                series_info_and_episode_list.run_full_caching().await
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

    use crate::core::database::database_transfer;
    use crate::core::database::KeysValuesVec;
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

    // TODO: Uncomment when needed
    // pub fn import() -> anyhow::Result<()> {
    //     let chosen_path = FileDialog::new()
    //         .set_directory(get_home_directory()?)
    //         .pick_file();

    //     if let Some(chosen_path) = chosen_path {
    //         database_transfer::read_database_from_path(path::Path::new(&chosen_path))?;
    //     }

    //     Ok(())
    // }

    pub fn import_keys_values_vec() -> anyhow::Result<Option<KeysValuesVec>> {
        let chosen_path = FileDialog::new()
            .set_directory(get_home_directory()?)
            .pick_file();

        if let Some(chosen_path) = chosen_path {
            let data = database_transfer::read_database_from_path_as_keys_value_vec(
                path::Path::new(&chosen_path),
            )?;
            return Ok(Some(data));
        }

        Ok(None)
    }

    pub fn get_home_directory() -> anyhow::Result<path::PathBuf> {
        let user_dirs = UserDirs::new().ok_or(anyhow::anyhow!("could not get user directory"))?;
        Ok(user_dirs.home_dir().to_path_buf())
    }
}
