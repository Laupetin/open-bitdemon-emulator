use bitdemon::domain::title::Title;
use bitdemon::lobby::storage::FileVisibility;
use log::info;
use num_traits::{FromPrimitive, ToPrimitive};
use rusqlite::Connection;
use std::cell::RefCell;
use std::fs::create_dir_all;

thread_local! {
    pub static STORAGE_DB: RefCell<Connection> = RefCell::new(initialized_db());
}

fn initialized_db() -> Connection {
    create_dir_all("db").expect("to be able to create dir");

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

pub fn from_title(value: Title) -> u32 {
    value.to_u32().unwrap()
}

pub fn to_title(value: u32) -> Title {
    Title::from_u32(value).expect("to be a valid title")
}

pub fn from_file_visibility(value: FileVisibility) -> u8 {
    match value {
        FileVisibility::VisiblePrivate => 0u8,
        FileVisibility::VisiblePublic => 1u8,
    }
}

pub fn to_file_visibility(value: u8) -> FileVisibility {
    match value {
        0 => FileVisibility::VisiblePrivate,
        value => {
            debug_assert_eq!(value, 1u8);
            FileVisibility::VisiblePublic
        }
    }
}
