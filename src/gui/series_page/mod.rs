use std::sync::mpsc;

use iced::{Command, Element, Renderer};
use indexmap::IndexMap;

use series::{IdentifiableMessage, Series};

use crate::core::api::series_information::SeriesMainInformation;

mod series;

#[derive(Debug, Clone)]
pub enum Message {
    Series(IdentifiableMessage),
}

pub struct SeriesPageController {
    series_pages: IndexMap<u32, Series>,
    series_page_sender: mpsc::Sender<SeriesMainInformation>,
    series_page_receiver: mpsc::Receiver<SeriesMainInformation>,
}

impl SeriesPageController {
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

    pub fn clear_all_pages(&mut self) {
        self.series_pages.clear();
    }

    pub fn try_series_page_switch(&mut self) -> Command<Message> {
        use crate::core::caching::{CacheFilePath, CACHER};

        match self.series_page_receiver.try_recv() {
            Ok(series_info) => {
                let series_page_id = series_info.id;

                // let series_id = series_page.get_series_id();
                // let series_info = series_page.get_series_main_information();

                // Caching SeriesMainInformation if it is not cached already
                //
                // Since discover poster's SeriesInformation are mostly taken online directly and hence don't
                // use the caching version of api to be obtained. This makes their cache folder lack their
                // SeriesMainInformation cache after being clicked. This cause their folders to be skipped
                // during cache cleaning making the show have same old episode and cast cache forever! unless
                // when it's tracked. So we fix this by caching it if it does not exists when switching to a series page.
                let series_main_info_cache_path = CACHER
                    .get_cache_file_path(CacheFilePath::SeriesMainInformation(series_page_id));
                if !series_main_info_cache_path.exists() {
                    // TODO: Asynchronously write the cache.
                    let mut folder_path = series_main_info_cache_path.to_owned();
                    folder_path.pop();

                    std::fs::create_dir_all(folder_path)
                        .expect("failed to create series cache folder");

                    std::fs::write(
                        series_main_info_cache_path,
                        serde_json::to_string_pretty(&series_info).expect("fail to serialize json"),
                    )
                    .expect("failed to save series main information cache");
                }

                let (series_page, series_page_command) =
                    Series::new(series_info, self.series_page_sender.clone());
                self.series_pages.insert(series_page_id, series_page);

                series_page_command.map(move |message| {
                    Message::Series(IdentifiableMessage::new(series_page_id, message))
                })
            }
            Err(err) => match err {
                mpsc::TryRecvError::Empty => Command::none(),
                mpsc::TryRecvError::Disconnected => panic!("series page senders disconnected"),
            },
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Series(identifiable_message) => {
                let series_page_id = identifiable_message.get_id();

                let command = if let Some(series_page) = self.series_pages.get_mut(&series_page_id)
                {
                    series_page
                        .update(identifiable_message.get_message())
                        .map(move |message| {
                            Message::Series(IdentifiableMessage::new(series_page_id, message))
                        })
                } else {
                    Command::none()
                };

                Command::batch([command, self.try_series_page_switch()])
            }
        }
    }

    pub fn view(&self) -> Option<Element<'_, Message, Renderer>> {
        self.series_pages.last().map(|(id, series_page)| {
            series_page
                .view()
                .map(|message| Message::Series(IdentifiableMessage::new(*id, message)))
        })
    }
}
