use crate::lobby::storage::db::{from_file_visibility, from_title, to_file_visibility, STORAGE_DB};
use bitdemon::domain::result_slice::ResultSlice;
use bitdemon::lobby::storage::{
    FileVisibility, StorageFileInfo, StorageServiceError, UserStorageService,
};
use bitdemon::networking::bd_session::BdSession;
use chrono::Utc;
use log::{info, warn};

pub struct DwUserStorageService {}

const MAX_FILENAME_LENGTH: usize = 260;
const MAX_USER_FILE_SIZE: usize = 50_000; // 50KB

impl UserStorageService for DwUserStorageService {
    fn get_storage_file_data_by_id(
        &self,
        session: &BdSession,
        owner_id: u64,
        file_id: u64,
    ) -> Result<Vec<u8>, StorageServiceError> {
        info!("Requesting file file_id={file_id} owner_id={owner_id}");

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
        info!("Requesting file filename={filename} owner_id={owner_id}",);

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
        _session: &BdSession,
        _owner_id: u64,
        _min_date_time: i64,
        _page_offset: usize,
        _page_size: usize,
    ) -> Result<ResultSlice<StorageFileInfo>, StorageServiceError> {
        todo!()
    }

    fn filter_storage_files(
        &self,
        _session: &BdSession,
        _owner_id: u64,
        _min_date_time: i64,
        _item_offset: usize,
        _item_count: usize,
        _filter: String,
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
        info!("Uploading file filename={filename} owner_id={owner_id} visibility={visibility:?} len={file_size}");

        let user_id = session.authentication().unwrap().user_id;
        if user_id != owner_id {
            warn!("Tried to upload file for other user");
            return Err(StorageServiceError::PermissionDeniedError);
        }

        if filename.len() > MAX_FILENAME_LENGTH {
            warn!("Tried to upload file with too long name");
            return Err(StorageServiceError::FilenameTooLongError);
        }

        if file_size > MAX_USER_FILE_SIZE {
            warn!("Tried to upload file that is too large");
            return Err(StorageServiceError::StorageFileTooLargeError);
        }

        let title = session.authentication().unwrap().title;
        let title_num = from_title(title);
        let now = Utc::now().timestamp();
        let visibility_num = from_file_visibility(visibility);

        let file_id: u64 = STORAGE_DB.with_borrow_mut(|db| {
            let transaction = db.transaction().expect("transaction to be started");

            let existing_file: rusqlite::Result<u64> = transaction.query_row(
                "SELECT u.id FROM user_file u WHERE u.filename = ? AND title = ? AND owner_id = ?",
                (filename.as_str(), title_num, owner_id),
                |row| row.get(0),
            );

            let file_id;
            if let Ok(existing_file_id) = existing_file {
                file_id = existing_file_id;
                transaction
                    .execute(
                        "UPDATE user_file SET data = ?2, modified_at = ?3 WHERE id = ?1",
                        (file_id, file_data, now),
                    )
                    .expect("file update to succeed");
            } else {
                transaction
                    .execute(
                        "INSERT INTO user_file
                             (filename, title, created_at, modified_at, visibility, owner_id, data)
                             VALUES
                             (?, ?, ?, ?, ?, ?, ?)",
                        (
                            filename.as_str(),
                            title_num,
                            now,
                            now,
                            visibility_num,
                            owner_id,
                            file_data,
                        ),
                    )
                    .expect("insertion to be successful");
                file_id = transaction.last_insert_rowid() as u64;
            }

            transaction.commit().expect("commit to be successful");

            file_id
        });

        Ok(StorageFileInfo {
            id: file_id,
            filename,
            title,
            file_size: file_size as u64,
            created: now,
            modified: now,
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
        info!("Uploading file file_id={file_id} owner_id={owner_id} len={file_size}");

        if session.authentication().unwrap().user_id != owner_id {
            warn!("Tried to update file for other user");
            return Err(StorageServiceError::PermissionDeniedError);
        }

        if file_size > MAX_USER_FILE_SIZE {
            warn!("Tried to update file with data that is too large");
            return Err(StorageServiceError::StorageFileTooLargeError);
        }

        let now = Utc::now().timestamp();
        let title = session.authentication().unwrap().title;
        let title_num = from_title(title);

        STORAGE_DB.with_borrow_mut(|db| {
            let transaction = db.transaction().expect("transaction to be open");

            let res: u64 = transaction
                .query_row(
                    "SELECT u.owner_id FROM user_file u WHERE u.id = ? AND title = ?",
                    (file_id, title_num),
                    |row| row.get(0),
                )
                .map_err(|_| StorageServiceError::StorageFileNotFoundError)?;

            if res != owner_id {
                return Err(StorageServiceError::PermissionDeniedError);
            }

            transaction
                .execute(
                    "UPDATE user_file SET data = ?2, modified_at = ?3 WHERE id = ?1",
                    (file_id, file_data, now),
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
        info!("Removing file filename={filename} owner_id={owner_id}");

        if session.authentication().unwrap().user_id != owner_id {
            warn!("Tried to delete file for other user");
            return Err(StorageServiceError::PermissionDeniedError);
        }

        if filename.len() > MAX_FILENAME_LENGTH {
            warn!("Tried to delete file with too long name");
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

impl DwUserStorageService {
    pub fn new() -> DwUserStorageService {
        DwUserStorageService {}
    }
}
