use bitdemon::domain::title::Title;
use bitdemon::lobby::content_streaming::{CategoryId, StreamSlot, StreamTag};
use chrono::Utc;
use log::info;
use num_traits::ToPrimitive;
use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::types::Value;
use rusqlite::{Connection, DropBehavior, Row};
use std::cell::RefCell;
use std::fs::create_dir_all;
use std::rc::Rc;

thread_local! {
    pub static CONTENT_STREAMING_DB: RefCell<Connection> = RefCell::new(initialized_db());
}

const CONTENT_STREAMING_CHANGELOG_0: &str = "
CREATE TABLE user_stream (
    id INTEGER PRIMARY KEY,
    filename TEXT NOT NULL,
    title INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    modified_at INTEGER NOT NULL,
    owner_id INTEGER NOT NULL,
    metadata BLOB,
    category INTEGER NOT NULL,
    slot INTEGER NOT NULL,
    data BLOB
);
CREATE TABLE user_stream_tag (
    stream_id INTEGER NOT NULL REFERENCES user_stream(id) ON DELETE CASCADE,
    primary_tag INTEGER NOT NULL,
    secondary_tag INTEGER NOT NULL
);
CREATE TABLE user_info (
    user_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);
CREATE UNIQUE INDEX user_stream_title_owner_id_slot_unq ON user_stream (
	title,
	owner_id,
	slot
);
";

fn initialized_db() -> Connection {
    create_dir_all("db").expect("to be able to create dir");

    let conn = Connection::open("db/content_streaming.db")
        .expect("expected db connection to be able to open");

    conn.execute("PRAGMA foreign_keys = ON", ())
        .expect("foreign keys to be able to be set");

    rusqlite::vtab::array::load_module(&conn).expect("array extension to be loadable");

    let version: u64 = conn
        .query_row("PRAGMA user_version", (), |row| row.get(0))
        .expect("Version to be available");
    if version < 1 {
        conn.execute_batch(CONTENT_STREAMING_CHANGELOG_0)
            .expect("Initialization to succeed");

        conn.execute("PRAGMA user_version = 1", ())
            .expect("Setting pragma to succeed");

        info!("Initialized content streaming db");
    }

    conn
}

pub struct PersistedStreamInfo {
    pub id: u64,
    pub filename: String,
    pub title: Title,
    pub stream_size: u64,
    pub created: i64,
    pub modified: i64,
    pub owner_id: u64,
    pub owner_name: String,
    pub metadata: Vec<u8>,
    pub category: CategoryId,
    pub slot: StreamSlot,
    pub tags: Vec<StreamTag>,
}

const GET_BY_ID_QUERY: &str = "
SELECT
    u.id,
    u.filename,
    length(data),
    u.created_at,
    u.modified_at,
    u.owner_id,
    ui.name,
    u.metadata,
    u.category,
    u.slot
FROM user_stream u
LEFT JOIN user_info ui ON u.owner_id = ui.user_id
WHERE u.id = ?1 AND u.title = ?2
";

const TAGS_FOR_STREAM_QUERY: &str = "
SELECT primary_tag,secondary_tag
FROM user_stream_tag t WHERE t.stream_id = ?1
";

pub fn get_streams_by_ids(title: Title, file_ids: &[u64]) -> Vec<PersistedStreamInfo> {
    let title_num = title.to_u32().unwrap();

    CONTENT_STREAMING_DB.with_borrow_mut(|db| {
        let mut transaction = db.transaction().expect("transaction to be started");
        transaction.set_drop_behavior(DropBehavior::Commit);

        let mut stream_query = transaction
            .prepare(GET_BY_ID_QUERY)
            .expect("preparation to be successful");

        let mut tags_query = transaction
            .prepare(TAGS_FOR_STREAM_QUERY)
            .expect("preparation to be successful");

        file_ids
            .iter()
            .copied()
            .filter_map(|file_id| {
                let mut stream_info = stream_query
                    .query_row((file_id, title_num), |row| {
                        Ok(map_persisted_stream_info(row, title).expect("mapping to work"))
                    })
                    .ok()?;

                stream_info.tags = tags_query
                    .query((file_id,))
                    .expect("query to be successful")
                    .mapped(|row| Ok(map_tag(row).expect("mapping to work")))
                    .filter_map(|row_value| row_value.ok())
                    .collect();

                Some(stream_info)
            })
            .collect()
    })
}

const COUNT_BY_OWNERS_QUERY: &str = "
SELECT COUNT(*)
FROM user_stream u
WHERE u.owner_id in rarray(?1) AND u.title = ?2
AND u.modified_at >= ?3
AND u.category = ?4
";

const GET_BY_OWNERS_QUERY: &str = "
SELECT
    u.id,
    u.filename,
    length(data),
    u.created_at,
    u.modified_at,
    u.owner_id,
    ui.name,
    u.metadata,
    u.category,
    u.slot
FROM user_stream u
LEFT JOIN user_info ui ON u.owner_id = ui.user_id
WHERE u.owner_id in rarray(?1) AND u.title = ?2
AND u.modified_at >= ?3
AND u.category = ?4
LIMIT ?6 OFFSET ?5
";

pub fn get_streams_by_owners(
    title: Title,
    owner_ids: &[u64],
    min_date_time: i64,
    category: u16,
    item_offset: usize,
    item_count: usize,
) -> (Vec<PersistedStreamInfo>, usize) {
    let title_num = title.to_u32().unwrap();
    let owner_id_values = Rc::new(
        owner_ids
            .iter()
            .copied()
            .map(|v| Value::from(v as i64))
            .collect::<Vec<Value>>(),
    );

    CONTENT_STREAMING_DB.with_borrow_mut(|db| {
        let mut transaction = db.transaction().expect("transaction to be started");
        transaction.set_drop_behavior(DropBehavior::Commit);

        let count: usize = transaction
            .query_row(
                COUNT_BY_OWNERS_QUERY,
                (owner_id_values.clone(), title_num, min_date_time, category),
                |row| row.get(0),
            )
            .expect("query to be successful");

        if count == 0 {
            return (Vec::new(), 0);
        }

        let mut tags_query = transaction
            .prepare(TAGS_FOR_STREAM_QUERY)
            .expect("preparation to be successful");

        let values = transaction
            .prepare(GET_BY_OWNERS_QUERY)
            .expect("preparing get query to be successful")
            .query((
                owner_id_values.clone(),
                title_num,
                min_date_time,
                category,
                item_offset,
                item_count,
            ))
            .expect("query to be successful")
            .mapped(|row| {
                let mut stream_info =
                    map_persisted_stream_info(row, title).expect("mapping to work");

                stream_info.tags = tags_query
                    .query((stream_info.id,))
                    .expect("query to be successful")
                    .mapped(|row| Ok(map_tag(row).expect("mapping to work")))
                    .filter_map(|row_value| row_value.ok())
                    .collect();

                Ok(stream_info)
            })
            .filter_map(|row_value| row_value.ok())
            .collect();

        (values, count)
    })
}

pub struct SlotCountForUpload {
    pub used_slots: usize,
    pub given_slot_is_taken: bool,
}

const COUNT_BY_USER_QUERY: &str = "
SELECT COUNT(*) FROM user_stream u
WHERE u.owner_id = ?1 AND u.title = ?2
";

const EXISTS_BY_SLOT_QUERY: &str = "
SELECT EXISTS(
    SELECT * FROM user_stream u
    WHERE u.owner_id = ?1 AND u.title = ?2 AND u.slot = ?3
)
";

pub fn get_slot_count_for_upload(
    title: Title,
    owner_id: u64,
    slot: StreamSlot,
) -> SlotCountForUpload {
    let title_num = title.to_u32().unwrap();

    CONTENT_STREAMING_DB.with_borrow_mut(|db| {
        let mut transaction = db.transaction().expect("transaction to be started");
        transaction.set_drop_behavior(DropBehavior::Commit);

        let used_slots: usize = transaction
            .query_row(COUNT_BY_USER_QUERY, (owner_id, title_num), |row| row.get(0))
            .expect("query to be successful");

        if used_slots == 0 {
            return SlotCountForUpload {
                used_slots,
                given_slot_is_taken: false,
            };
        }

        transaction
            .query_row(EXISTS_BY_SLOT_QUERY, (owner_id, title_num, slot), |row| {
                row.get(0)
            })
            .map(|given_slot_is_taken| SlotCountForUpload {
                used_slots,
                given_slot_is_taken,
            })
            .unwrap_or_else(|_| SlotCountForUpload {
                used_slots,
                given_slot_is_taken: false,
            })
    })
}

const CREATE_EMPTY_STREAM_SQL: &str = "
INSERT INTO user_stream (
    filename,
    title,
    created_at,
    modified_at,
    owner_id,
    metadata,
    category,
    slot,
    data
) VALUES (
    ?1, ?2, ?3, ?4, ?5, null, ?6, ?7, null
) ON CONFLICT (title, owner_id, slot) DO UPDATE SET
    filename=?1,
    modified_at=?4,
    metadata=null,
    category=?6,
    data=null
RETURNING id
";

pub fn create_empty_stream(
    title: Title,
    owner_id: u64,
    filename: &str,
    slot: StreamSlot,
    category: CategoryId,
) -> u64 {
    let title_num = title.to_u32().unwrap();
    let now = Utc::now().timestamp();

    CONTENT_STREAMING_DB.with_borrow_mut(|db| {
        let mut transaction = db.transaction().expect("transaction to be started");
        transaction.set_drop_behavior(DropBehavior::Commit);

        transaction
            .query_row(
                CREATE_EMPTY_STREAM_SQL,
                (filename, title_num, now, now, owner_id, category, slot),
                |row| row.get(0),
            )
            .expect("Insertion to be successful")
    })
}

const GET_DATA_BY_ID_QUERY: &str = "
SELECT
    u.data
    FROM user_stream u
WHERE u.title = ?1 AND u.id = ?2
";

pub fn get_stream_data(title: Title, stream_id: u64) -> Option<Vec<u8>> {
    let title_num = title.to_u32().unwrap();

    CONTENT_STREAMING_DB.with_borrow(|db| {
        db.query_row(GET_DATA_BY_ID_QUERY, (title_num, stream_id), |row| {
            row.get(0)
        })
        .ok()
    })
}

const IS_DATA_NULL_QUERY: &str = "
SELECT EXISTS(
    SELECT * FROM user_stream u
    WHERE u.title = ?1 AND u.id = ?2 AND u.data IS NULL
)
";

const SET_DATA_BY_ID_SQL: &str = "
UPDATE user_stream
SET data = ?3
WHERE title = ?1 AND id = ?2
";

pub fn set_stream_data(title: Title, stream_id: u64, data: Vec<u8>) -> bool {
    let title_num = title.to_u32().unwrap();

    CONTENT_STREAMING_DB.with_borrow_mut(|db| {
        let mut transaction = db.transaction().expect("transaction to be started");
        transaction.set_drop_behavior(DropBehavior::Commit);

        let can_set_data: bool = transaction
            .query_row(IS_DATA_NULL_QUERY, (title_num, stream_id), |row| row.get(0))
            .expect("query to be successful");

        if !can_set_data {
            return false;
        }

        transaction
            .execute(SET_DATA_BY_ID_SQL, (title_num, stream_id, data))
            .expect("setting data to be successful");

        true
    })
}

const GET_ID_FOR_SLOT_AND_NULL_METADATA_QUERY: &str = "
SELECT u.id FROM user_stream u
WHERE u.title = ?1 AND u.slot = ?2 AND u.owner_id = ?3 AND u.metadata IS NULL
";

const SET_METADATA_BY_ID_SQL: &str = "
UPDATE user_stream
SET metadata = ?4
WHERE title = ?1 AND id = ?2 AND owner_id = ?3
";

const ADD_TAG_SQL: &str = "
INSERT INTO user_stream_tag
(stream_id, primary_tag, secondary_tag)
VALUES (?1, ?2, ?3);
";

pub fn set_stream_metadata(
    title: Title,
    owner_id: u64,
    slot: StreamSlot,
    metadata: Vec<u8>,
    tags: Vec<StreamTag>,
) -> Result<u64, ()> {
    let title_num = title.to_u32().unwrap();

    CONTENT_STREAMING_DB.with_borrow_mut(|db| {
        let mut transaction = db.transaction().expect("transaction to be started");
        transaction.set_drop_behavior(DropBehavior::Commit);

        let stream_id: u64 = transaction
            .query_row(
                GET_ID_FOR_SLOT_AND_NULL_METADATA_QUERY,
                (title_num, slot, owner_id),
                |row| row.get(0),
            )
            .map_err(|_| ())?;

        transaction
            .execute(
                SET_METADATA_BY_ID_SQL,
                (title_num, stream_id, owner_id, metadata),
            )
            .expect("setting data to be successful");

        let mut tags_insert = transaction
            .prepare(ADD_TAG_SQL)
            .expect("preparation to be successful");

        tags.iter().for_each(|tag| {
            tags_insert
                .execute((stream_id, tag.primary, tag.secondary))
                .expect("setting metadata to be successful");
        });

        Ok(stream_id)
    })
}

const GET_ID_FOR_SLOT_QUERY: &str = "
SELECT u.id FROM user_stream u
WHERE u.title = ?1 AND u.slot = ?2 AND u.owner_id = ?3
";

pub fn get_stream_id_for_slot(title: Title, owner_id: u64, slot: StreamSlot) -> Result<u64, ()> {
    let title_num = title.to_u32().unwrap();

    CONTENT_STREAMING_DB.with_borrow(|db| {
        db.query_row(GET_ID_FOR_SLOT_QUERY, (title_num, slot, owner_id), |row| {
            row.get(0)
        })
        .map_err(|_| ())
    })
}

const DELETE_STREAM_BY_ID_SQL: &str = "
DELETE FROM user_stream u
WHERE u.title = ?1 AND u.id = ?2
";

pub fn delete_db_stream(title: Title, stream_id: u64) -> Result<(), ()> {
    let title_num = title.to_u32().unwrap();

    CONTENT_STREAMING_DB.with_borrow(|db| {
        db.execute(DELETE_STREAM_BY_ID_SQL, (title_num, stream_id))
            .map(|_| ())
            .map_err(|_| ())
    })
}

const RECORD_USER_NAME_SQL: &str = "
INSERT INTO user_info
(user_id, name)
VALUES (?1, ?2)
ON CONFLICT (user_id) DO UPDATE SET
name = ?2
";

pub fn record_user_name(user_id: u64, name: &str) {
    CONTENT_STREAMING_DB.with_borrow(|db| {
        db.execute(RECORD_USER_NAME_SQL, (user_id, name))
            .expect("recording user name to work");
    })
}

fn map_persisted_stream_info(row: &Row, title: Title) -> rusqlite::Result<PersistedStreamInfo> {
    Ok(PersistedStreamInfo {
        id: row.get(0)?,
        filename: row.get(1)?,
        title,
        stream_size: row.get(2)?,
        created: row.get(3)?,
        modified: row.get(4)?,
        owner_id: row.get(5)?,
        owner_name: row.get(6).unwrap_or_else(|_| "".to_string()),
        metadata: row.get(7)?,
        category: row.get(8)?,
        slot: row.get(9)?,
        tags: Vec::new(),
    })
}

fn map_tag(row: &Row) -> rusqlite::Result<StreamTag> {
    Ok(StreamTag {
        primary: row.get(0)?,
        secondary: row.get(1)?,
    })
}
