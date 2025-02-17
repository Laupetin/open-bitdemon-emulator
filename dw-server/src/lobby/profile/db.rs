use log::info;
use rusqlite::Connection;
use std::cell::RefCell;
use std::fs::create_dir_all;

thread_local! {
    pub static PROFILE_DB: RefCell<Connection> = RefCell::new(initialized_db());
}

fn initialized_db() -> Connection {
    create_dir_all("db").expect("to be able to create dir");

    let conn =
        Connection::open("db/profile.db").expect("expected db connection to be able to open");

    let version: u64 = conn
        .query_row("PRAGMA user_version", (), |row| row.get(0))
        .expect("Version to be available");
    if version < 1 {
        conn.execute(
            "CREATE TABLE user_profile (
                    id INTEGER PRIMARY KEY,
                    title INTEGER NOT NULL,
                    owner_id INTEGER NOT NULL,
                    profile_type INTEGER NOT NULL,
                    created_at INTEGER NOT NULL,
                    modified_at INTEGER NOT NULL,
                    data BLOB NOT NULL
                 )",
            (),
        )
        .expect("Initialization to succeed");

        conn.execute("PRAGMA user_version = 1", ())
            .expect("Setting pragma to succeed");

        info!("Initialized profile db");
    }

    conn
}

pub enum ProfileType {
    Public,
    Private,
}

impl From<u8> for ProfileType {
    fn from(value: u8) -> Self {
        match value {
            0 => ProfileType::Private,
            _ => {
                debug_assert_eq!(value, 1);
                ProfileType::Public
            }
        }
    }
}

impl From<ProfileType> for u8 {
    fn from(value: ProfileType) -> Self {
        match value {
            ProfileType::Private => 0,
            ProfileType::Public => 1,
        }
    }
}
