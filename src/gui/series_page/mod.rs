use std::sync::mpsc;

use iced::{Element, Task};
use indexmap::IndexMap;

use series::{Message as SeriesMessage, Series};

use crate::core::api::tv_maze::series_information::SeriesMainInformation;

use super::troxide_widget::series_poster::IndexedMessage;

mod series;

#[derive(Debug, Clone)]
pub enum Message {
    Series(IndexedMessage<u32, SeriesMessage>),
    SeriesCacheFileWritten,
}

pub struct SeriesPageController<'a> {
    series_pages: IndexMap<u32, Series<'a>>,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
    series_page_receiver: mpsc::Receiver<SeriesMainInformation>,
}

impl<'a> SeriesPageController<'a> {
    pub fn new(
        series_page_sender: mpsc::Sender<SeriesMainInformation>,
        series_page_receiver: mpsc::Receiver<SeriesMainInformation>,
    ) -> Self {
        Self {
            series_pages: IndexMap::new(),
            series_page_sender,
            series_page_receiver,
        }
    }

    /// Clears all the series pages
    pub fn clear_all_pages(&mut self) {
        self.series_pages.clear();
    }

    /// whether there is a series page
    pub fn has_a_series_page(&self) -> bool {
        !self.series_pages.is_empty()
    }

    /// Goes to the previous opened series page discarding the current one
    pub fn go_previous(&mut self) -> Task<Message> {
        self.series_pages.pop();
        self.series_pages
            .last()
            .map(|(id, series_page)| {
                let id = *id;
                series_page
                    .restore_scroller_relative_offset()
                    .map(move |message| Message::Series(IndexedMessage::new(id, message)))
            })
            .unwrap_or(Task::none())
    }

    /// Retrieves the `Series Name` for the current active series page if available
    pub fn get_series_page_name(&self) -> Option<&str> {
        self.series_pages
            .last()
            .as_ref()
            .map(|(_, series)| series.get_series_main_information().name.as_str())
    }

    /// Tries to switch to series page if any has been received
    pub fn try_series_page_switch(&mut self) -> Task<Message> {
        use crate::core::caching::{CacheFilePath, CACHER};
        use tokio::fs;
        use tracing::error;

        match self.series_page_receiver.try_recv() {
            Ok(series_info) => {
                let series_page_id = series_info.id;

                let series_page_command = if let Some((series_page_id, series_page)) =
                    self.series_pages.shift_remove_entry(&series_page_id)
                {
                    let restore_scroller_command = series_page.set_relative_offset_to_start();

                    // Shifting the series page to the front if it already exists in the map
                    self.series_pages.insert(series_page_id, series_page);

                    restore_scroller_command.map(move |message| {
                        Message::Series(IndexedMessage::new(series_page_id, message))
                    })
                } else {
                    let (series_page, series_page_command) =
                        Series::new(series_info.clone(), self.series_page_sender.clone());
                    self.series_pages.insert(series_page_id, series_page);

                    series_page_command.map(move |message| {
                        Message::Series(IndexedMessage::new(series_page_id, message))
                    })
                };

                // Caching SeriesMainInformation if it is not cached already
                //
                // Since discover poster's SeriesInformation are mostly taken online directly and hence don't
                // use the caching version of api to be obtained. This makes their cache folder lack their
                // SeriesMainInformation cache after being clicked. This cause their folders to be skipped
                // during cache cleaning making the show have same old episode and cast cache forever! unless
                // when it's tracked. So we fix this by caching it if it does not exists when switching to a series page.
                let series_main_info_cache_path = CACHER
                    .get_cache_file_path(CacheFilePath::SeriesMainInformation(series_page_id));

                let cache_file_creation_future = async move {
                    if !fs::try_exists(&series_main_info_cache_path)
                        .await
                        .unwrap_or(false)
                    {
                        let mut folder_path = series_main_info_cache_path.to_owned();
                        folder_path.pop();

                        fs::create_dir_all(folder_path).await.unwrap_or_else(|err| {
                            error!(
                                "failed to create series cache folder for series id {}: {}",
                                series_page_id, err
                            )
                        });

                        fs::write(
                            series_main_info_cache_path,
                            serde_json::to_string_pretty(&series_info)
                                .expect("fail to serialize series info to json"),
                        )
                        .await
                        .unwrap_or_else(|err| {
                            error!(
                                "failed to save series main information cache for series id {}: {}",
                                series_page_id, err
                            )
                        });
                    }
                };

                Task::batch([
                    series_page_command,
                    Task::perform(cache_file_creation_future, |_| {
                        Message::SeriesCacheFileWritten
                    }),
                ])
            }
            Err(err) => match err {
                mpsc::TryRecvError::Empty => Task::none(),
                mpsc::TryRecvError::Disconnected => panic!("series page senders disconnected"),
            },
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Series(identifiable_message) => {
                let series_page_id = identifiable_message.index();

                let command = if let Some(series_page) = self.series_pages.get_mut(&series_page_id)
                {
                    series_page
                        .update(identifiable_message.message())
                        .map(move |message| {
                            Message::Series(IndexedMessage::new(series_page_id, message))
                        })
                } else {
                    Task::none()
                };

                Task::batch([command, self.try_series_page_switch()])
            }
            Message::SeriesCacheFileWritten => Task::none(),
        }
    }

    pub fn view(&self) -> Option<Element<'_, Message>> {
        self.series_pages.last().map(|(id, series_page)| {
            series_page
                .view()
                .map(|message| Message::Series(IndexedMessage::new(*id, message)))
        })
    }
}
