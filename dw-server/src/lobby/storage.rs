use bitdemon::domain::result_slice::ResultSlice;
use bitdemon::domain::title::Title;
use bitdemon::lobby::handler::storage::{
    FileVisibility, PublisherStorageService, StorageFileInfo, StorageHandler, StorageService,
    StorageServiceError,
};
use bitdemon::lobby::ThreadSafeLobbyHandler;
use bitdemon::networking::bd_session::BdSession;
use chrono::Utc;
use log::{info, warn};
use num_traits::{FromPrimitive, ToPrimitive};
use rusqlite::Connection;
use std::cell::RefCell;
use std::fs;
use std::fs::DirEntry;
use std::path::{Component, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

pub fn create_storage_handler() -> Arc<ThreadSafeLobbyHandler> {
    Arc::new(StorageHandler::new(
        Arc::new(DwStorageService::new()),
        Arc::new(DwPublisherStorageService::new()),
    ))
}

struct DwStorageService {}

fn initialized_db() -> Connection {
    let conn =
        Connection::open("db/storage.db").expect("expected db connection to be able to open");

    let version: u64 = conn
        .query_row("PRAGMA user_version", (), |row| row.get(0))
        .expect("Version to be available");
    if version < 1 {
        conn.execute(
            "CREATE TABLE user_file (
                    id INTEGER PRIMARY KEY,
                    filename TEXT NOT NULL,
                    title INTEGER NOT NULL,
                    created_at INTEGER NOT NULL,
                    modified_at INTEGER NOT NULL,
                    visibility INTEGER NOT NULL,
                    owner_id INTEGER NOT NULL,
                    data BLOB NOT NULL
                 )",
            (),
        )
        .expect("Initialization to succeed");

        conn.execute("PRAGMA user_version = 1", ())
            .expect("Setting pragma to succeed");

        info!("Initialized storage db");
    }

    conn
}

thread_local! {
    pub static STORAGE_DB: RefCell<Connection> = RefCell::new(initialized_db());
}

const MAX_FILENAME_LENGTH: usize = 260;
const MAX_USER_FILE_SIZE: usize = 50_000; // 50KB

fn from_file_visibility(value: FileVisibility) -> u8 {
    match value {
        FileVisibility::VisiblePrivate => 0u8,
        FileVisibility::VisiblePublic => 1u8,
    }
}

fn to_file_visibility(value: u8) -> FileVisibility {
    match value {
        0 => FileVisibility::VisiblePrivate,
        value => {
            debug_assert_eq!(value, 1u8);
            FileVisibility::VisiblePublic
        }
    }
}

fn from_title(value: Title) -> u32 {
    value.to_u32().unwrap()
}

fn to_title(value: u32) -> Title {
    Title::from_u32(value).expect("to be a valid title")
}

impl StorageService for DwStorageService {
    fn get_storage_file_data_by_id(
        &self,
        session: &BdSession,
        owner_id: u64,
        file_id: u64,
    ) -> Result<Vec<u8>, StorageServiceError> {
        info!(
            "[Session {}] Requesting file file_id={file_id} owner_id={owner_id}",
            session.id,
        );

        if session.authentication().unwrap().user_id != owner_id {
            return Err(StorageServiceError::PermissionDeniedError);
        }

        let res = STORAGE_DB.with_borrow(|db| {
            db.query_row(
                "SELECT data FROM user_file u
                     WHERE u.id = ?1 AND u.owner_id = ?2",
                (file_id, owner_id),
                |row| row.get(0),
            )
        });

        res.map_err(|_| StorageServiceError::StorageFileNotFoundError)
    }

    fn get_storage_file_data_by_name(
        &self,
        session: &BdSession,
        owner_id: u64,
        filename: String,
    ) -> Result<Vec<u8>, StorageServiceError> {
        info!(
            "[Session {}] Requesting file filename={filename} owner_id={owner_id}",
            session.id,
        );

        let is_owner = session.authentication().unwrap().user_id == owner_id;

        if filename.len() > MAX_FILENAME_LENGTH {
            return Err(StorageServiceError::StorageFileNotFoundError);
        }

        let res: rusqlite::Result<(u8, Vec<u8>)> = STORAGE_DB.with_borrow(|db| {
            db.query_row(
                "SELECT u.visibility, u.data FROM user_file u
                     WHERE u.filename = ?1 AND u.owner_id = ?2",
                (filename.as_str(), owner_id),
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
        });

        res.map_err(|_| StorageServiceError::StorageFileNotFoundError)
            .and_then(|file| {
                let visibility = to_file_visibility(file.0);
                if visibility == FileVisibility::VisiblePrivate && !is_owner {
                    return Err(StorageServiceError::PermissionDeniedError);
                }

                Ok(file.1)
            })
    }

    fn list_storage_files(
        &self,
        session: &BdSession,
        owner_id: u64,
        min_date_time: i64,
        page_offset: usize,
        page_size: usize,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError> {
        todo!()
    }

    fn filter_storage_files(
        &self,
        session: &BdSession,
        owner_id: u64,
        min_date_time: i64,
        item_offset: usize,
        item_count: usize,
        filter: String,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError> {
        todo!()
    }

    fn create_storage_file(
        &self,
        session: &BdSession,
        owner_id: u64,
        filename: String,
        visibility: FileVisibility,
        file_data: Vec<u8>,
    ) -> Result<StorageFileInfo, StorageServiceError> {
        let file_size = file_data.len();
        info!(
            "[Session {}] Uploading file filename={filename} owner_id={owner_id} visibility={visibility:?} len={file_size}",
            session.id
        );

        let user_id = session.authentication().unwrap().user_id;
        if user_id != owner_id {
            warn!(
                "[Session {}] Tried to upload file for other user",
                session.id
            );
            return Err(StorageServiceError::PermissionDeniedError);
        }

        if filename.len() > MAX_FILENAME_LENGTH {
            warn!(
                "[Session {}] Tried to upload file with too long name",
                session.id
            );
            return Err(StorageServiceError::FilenameTooLongError);
        }

        if file_size > MAX_USER_FILE_SIZE {
            warn!(
                "[Session {}] Tried to upload file that is too large",
                session.id
            );
            return Err(StorageServiceError::StorageFileTooLargeError);
        }

        let title = session.authentication().unwrap().title;
        let title_num = from_title(title);
        let created_at = Utc::now().timestamp();
        let visibility_num = from_file_visibility(visibility);

        let file_id: u64 = STORAGE_DB.with_borrow_mut(|db| {
            let transaction = db.transaction().expect("transaction to be started");

            transaction
                .execute(
                    "INSERT INTO user_file u
                     (filename, title, created, modified, visibility, owner_id, data)
                     VALUES
                     (?, ?, ?, ?, ?, ?, ?)",
                    (
                        filename.as_str(),
                        title_num,
                        created_at,
                        created_at,
                        visibility_num,
                        owner_id,
                        file_data,
                    ),
                )
                .expect("insertion to be successful");

            let file_id = transaction.last_insert_rowid() as u64;

            transaction.commit().expect("commit to be successful");

            file_id
        });

        Ok(StorageFileInfo {
            id: file_id,
            filename,
            title,
            file_size: file_size as u64,
            created: created_at,
            modified: created_at,
            visibility,
            owner_id,
        })
    }

    fn update_storage_file_data(
        &self,
        session: &BdSession,
        owner_id: u64,
        file_id: u64,
        file_data: Vec<u8>,
    ) -> Result<(), StorageServiceError> {
        let file_size = file_data.len();
        info!(
            "[Session {}] Uploading file file_id={file_id} owner_id={owner_id} len={file_size}",
            session.id
        );

        if session.authentication().unwrap().user_id != owner_id {
            warn!(
                "[Session {}] Tried to update file for other user",
                session.id
            );
            return Err(StorageServiceError::PermissionDeniedError);
        }

        if file_size > MAX_USER_FILE_SIZE {
            warn!(
                "[Session {}] Tried to update file with data that is too large",
                session.id
            );
            return Err(StorageServiceError::StorageFileTooLargeError);
        }

        STORAGE_DB.with_borrow_mut(|db| {
            let transaction = db.transaction().expect("transaction to be open");

            let res: u64 = transaction
                .query_row(
                    "SELECT u.owner_id FROM user_file u WHERE u.id = ?",
                    (file_id,),
                    |row| row.get(0),
                )
                .map_err(|_| StorageServiceError::StorageFileNotFoundError)?;

            if res != owner_id {
                return Err(StorageServiceError::PermissionDeniedError);
            }

            transaction
                .execute(
                    "UPDATE user_file SET data = ?2 WHERE id = ?1",
                    (file_id, file_data),
                )
                .expect("file update to succeed");

            transaction.commit().expect("commit to work");

            Ok(())
        })
    }

    fn remove_storage_file(
        &self,
        session: &BdSession,
        owner_id: u64,
        filename: String,
    ) -> Result<(), StorageServiceError> {
        info!(
            "[Session {}] Removing file filename={filename} owner_id={owner_id}",
            session.id
        );

        if session.authentication().unwrap().user_id != owner_id {
            warn!(
                "[Session {}] Tried to delete file for other user",
                session.id
            );
            return Err(StorageServiceError::PermissionDeniedError);
        }

        if filename.len() > MAX_FILENAME_LENGTH {
            warn!(
                "[Session {}] Tried to delete file with too long name",
                session.id
            );
            return Err(StorageServiceError::FilenameTooLongError);
        }

        STORAGE_DB.with_borrow(move |db| {
            let res = db
                .execute("DELETE FROM user_file u WHERE u.filename = ?", (filename,))
                .map_err(|_| StorageServiceError::StorageFileNotFoundError)?;

            if res > 0 {
                Ok(())
            } else {
                Err(StorageServiceError::StorageFileNotFoundError)
            }
        })
    }
}

impl DwStorageService {
    pub fn new() -> DwStorageService {
        DwStorageService {}
    }
}

struct DwPublisherStorageService {}

impl PublisherStorageService for DwPublisherStorageService {
    fn get_publisher_file_data(
        &self,
        session: &BdSession,
        filename: String,
    ) -> Result<Vec<u8>, StorageServiceError> {
        info!(
            "[Session {}] Requesting publisher file {}",
            session.id,
            filename.as_str()
        );

        let path_buf = PathBuf::from_str(&filename)
            .or_else(|_| Err(StorageServiceError::StorageFileNotFoundError))?;

        let directory_traversal = path_buf
            .components()
            .into_iter()
            .any(|component| component == Component::ParentDir);

        if directory_traversal {
            warn!(
                "[Session {}] User attempted directory traversal!",
                session.id
            );
            return Err(StorageServiceError::StorageFileNotFoundError);
        }

        let full_file_path = format!(
            "storage/publisher/{}/{filename}",
            session.authentication().unwrap().title.to_u32().unwrap()
        );

        let buf = fs::read(full_file_path).or_else(|_| {
            warn!(
                "[Session {}] Requested publisher file could not be found",
                session.id
            );
            Err(StorageServiceError::StorageFileNotFoundError)
        })?;

        Ok(buf)
    }

    fn list_publisher_files(
        &self,
        session: &BdSession,
        min_date_time: i64,
        item_offset: usize,
        item_count: usize,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError> {
        info!(
            "[Session {}] Listing publisher files min_date_time={min_date_time} item_offset={item_offset} item_count={item_count}",
            session.id
        );

        let title = session.authentication().unwrap().title;
        let full_dir_path = format!("storage/publisher/{}", title.to_u32().unwrap());

        let dir = fs::read_dir(full_dir_path);
        if dir.is_err() {
            return Ok(ResultSlice::new(Vec::new(), item_offset));
        }

        let file_info = dir
            .unwrap()
            .filter(|entry| entry.is_ok())
            .skip(item_offset)
            .map(|entry| entry.unwrap())
            .map(|entry| Self::map_info_info(title, entry))
            .filter(|info| info.created >= min_date_time)
            .take(item_count)
            .collect();

        Ok(ResultSlice::new(file_info, item_offset))
    }

    fn filter_publisher_files(
        &self,
        session: &BdSession,
        min_date_time: i64,
        item_offset: usize,
        item_count: usize,
        filter: String,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError> {
        info!(
            "[Session {}] Filtering publisher files min_date_time={min_date_time} item_offset={item_offset} item_count={item_count} filter={filter}",
            session.id
        );

        let title = session.authentication().unwrap().title;
        let full_dir_path = format!("storage/publisher/{}", title.to_u32().unwrap());

        let dir = fs::read_dir(full_dir_path);
        if dir.is_err() {
            return Ok(ResultSlice::new(Vec::new(), item_offset));
        }

        let file_info = dir
            .unwrap()
            .filter(|entry| entry.is_ok())
            .filter(|entry| {
                entry
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .to_str()
                    .unwrap()
                    .starts_with(&filter)
            })
            .skip(item_offset)
            .map(|entry| entry.unwrap())
            .map(|entry| Self::map_info_info(title, entry))
            .filter(|info| info.created >= min_date_time)
            .take(item_count)
            .collect();

        Ok(ResultSlice::new(file_info, item_offset))
    }
}

impl DwPublisherStorageService {
    fn new() -> DwPublisherStorageService {
        DwPublisherStorageService {}
    }

    fn map_info_info(title: Title, entry: DirEntry) -> StorageFileInfo {
        let metadata = entry.metadata().unwrap();
        StorageFileInfo {
            id: 0,
            filename: entry.file_name().into_string().unwrap(),
            title,
            file_size: metadata.len(),
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
            visibility: FileVisibility::VisiblePublic,
            owner_id: 0,
        }
    }
}
