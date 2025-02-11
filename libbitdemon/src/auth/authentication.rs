use crate::domain::title::Title;

pub struct SessionAuthentication {
    pub user_id: u64,
    pub username: String,
    pub session_key: [u8; 24],
    pub title: Title,
}
