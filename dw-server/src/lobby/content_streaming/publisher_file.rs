use bitdemon::domain::result_slice::ResultSlice;
use bitdemon::domain::title::Title;
use bitdemon::lobby::content_streaming::{
    ContentStreamingServiceError, PublisherContentStreamingService, StreamInfo,
};
use bitdemon::networking::bd_session::BdSession;
use chrono::{DateTime, Utc};
use log::info;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::fs;
use std::fs::DirEntry;
use std::ops::Sub;
use std::sync::{RwLock, RwLockReadGuard};
use std::time::UNIX_EPOCH;

pub struct DwPublisherContentStreamingService {
    publisher_streams: RwLock<HashMap<Title, PublisherStreamState>>,
}

impl PublisherContentStreamingService for DwPublisherContentStreamingService {
    fn get_publisher_stream_by_id(
        &self,
        session: &BdSession,
        file_id: u64,
    ) -> Result<StreamInfo, ContentStreamingServiceError> {
        info!("Getting publisher stream {file_id}");

        let authentication = session
            .authentication()
            .expect("authentication was required for handler");

        self.stream_by_id(authentication.title, file_id)
            .ok_or(ContentStreamingServiceError::NoStreamFound)
    }

    fn list_publisher_streams(
        &self,
        session: &BdSession,
        min_date_time: i64,
        category: u16,
        item_offset: usize,
        item_count: usize,
    ) -> Result<ResultSlice<StreamInfo>, ContentStreamingServiceError> {
        info!("Listing publisher streams min={min_date_time} category={category} offset={item_offset} count={item_count}");

        let authentication = session
            .authentication()
            .expect("authentication was required for handler");

        let read = self.read_publisher_streams(authentication.title);

        let state = read
            .get(&authentication.title)
            .expect("state to be created");

        // TODO: Filter for category
        let stream_info: Vec<StreamInfo> = state
            .streams
            .iter()
            .filter(|info| info.modified >= min_date_time)
            .skip(item_offset)
            .take(item_count)
            .cloned()
            .collect();

        if !stream_info.is_empty() {
            Ok(ResultSlice::new(stream_info, item_offset))
        } else {
            Err(ContentStreamingServiceError::NoStreamFound)
        }
    }

    fn filter_publisher_streams(
        &self,
        session: &BdSession,
        min_date_time: i64,
        category: u16,
        item_offset: usize,
        item_count: usize,
        filter: String,
    ) -> Result<ResultSlice<StreamInfo>, ContentStreamingServiceError> {
        info!("Filtering publisher streams filter={filter} min={min_date_time} category={category} offset={item_offset} count={item_count}");

        let authentication = session
            .authentication()
            .expect("authentication was required for handler");

        let read = self.read_publisher_streams(authentication.title);

        let state = read
            .get(&authentication.title)
            .expect("state to be created");

        // TODO: Filter for category
        let stream_info: Vec<StreamInfo> = state
            .streams
            .iter()
            .filter(|info| info.modified >= min_date_time)
            .filter(|info| info.filename.starts_with(&filter))
            .skip(item_offset)
            .take(item_count)
            .cloned()
            .collect();

        if !stream_info.is_empty() {
            Ok(ResultSlice::new(stream_info, item_offset))
        } else {
            Err(ContentStreamingServiceError::NoStreamFound)
        }
    }
}

impl DwPublisherContentStreamingService {
    pub fn new() -> DwPublisherContentStreamingService {
        let state_map = HashMap::new();

        DwPublisherContentStreamingService {
            publisher_streams: RwLock::new(state_map),
        }
    }

    pub fn stream_by_id(&self, title: Title, file_id: u64) -> Option<StreamInfo> {
        let lock = self.read_publisher_streams(title);
        let state = lock.get(&title).expect("state to be created");

        state
            .streams
            .iter()
            .find(|info| info.id == file_id)
            .cloned()
    }

    fn read_publisher_streams(
        &self,
        title: Title,
    ) -> RwLockReadGuard<'_, HashMap<Title, PublisherStreamState>> {
        {
            let lock = self.publisher_streams.read().unwrap();
            if let Some(stream_state) = lock.get(&title) {
                if !stream_state.refresh_necessary() {
                    return lock;
                }
            }
        }

        {
            let mut lock = self.publisher_streams.write().unwrap();
            if let Some(write_state) = lock.get_mut(&title) {
                write_state.refresh_if_necessary();
            } else {
                lock.insert(title, PublisherStreamState::create_and_initialize(title));
            }
        }

        let lock = self.publisher_streams.read().unwrap();
        lock
    }
}

struct PublisherStreamState {
    last_refresh: DateTime<Utc>,
    title: Title,
    next_id: u64,
    streams: Vec<StreamInfo>,
}

const STATE_REFRESH_SECONDS: i64 = 60;

impl PublisherStreamState {
    fn create_and_initialize(title: Title) -> Self {
        let mut result = PublisherStreamState {
            last_refresh: Utc::now(),
            title,
            next_id: 1,
            streams: Vec::new(),
        };

        result.refresh();

        result
    }

    fn refresh_necessary(&self) -> bool {
        let now = Utc::now();

        now.sub(self.last_refresh).num_seconds() > STATE_REFRESH_SECONDS
    }

    fn refresh_if_necessary(&mut self) {
        if self.refresh_necessary() {
            self.refresh();
        }
    }

    fn refresh(&mut self) {
        let dir_name = format!("stream/publisher/{}", self.title.to_u32().unwrap());
        if let Ok(dir) = fs::read_dir(dir_name) {
            dir.filter_map(|entry| entry.ok())
                .for_each(|entry| self.handle_entry(entry));
        }
    }

    fn handle_entry(&mut self, entry: DirEntry) {
        let metadata = entry.metadata().expect("metadata to be retrievable");
        let filename = entry.file_name().into_string().unwrap();

        let maybe_existing_entry = self
            .streams
            .iter_mut()
            .find(|stream| stream.filename == filename);

        if let Some(existing_entry) = maybe_existing_entry {
            existing_entry.stream_size = metadata.len();
            existing_entry.modified = metadata
                .modified()
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
        } else {
            let id = self.next_id;
            let title_num = self.title.to_u32().unwrap();
            self.next_id += 1;
            self.streams.push(StreamInfo {
                id,
                filename: entry.file_name().into_string().unwrap(),
                title: self.title,
                stream_size: metadata.len(),
                created: metadata
                    .created()
                    .unwrap()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                modified: metadata
                    .modified()
                    .unwrap()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                owner_id: 0,
                owner_name: "".to_string(),
                url: format!("http://localhost:3000/content/publisher/{title_num}/{id}"),
                metadata: vec![],
                category: 0,
                slot: 0,
                tags: vec![],
                num_copies_made: 0,
                summary_file_size: 0,
                origin_id: 0,
            });
        }
    }
}
