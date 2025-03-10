use serde::{Deserialize, Serialize};

const DEFAULT_CONTENT_PORT: u16 = 3076;
const DEFAULT_HOSTNAME: &str = "localhost";

#[derive(Serialize, Deserialize, Default)]
pub struct DwServerConfig {
    content_port: Option<u16>,
    /// The hostname under which the server can be reached
    hostname: Option<String>,
}

impl DwServerConfig {
    pub fn content_port(&self) -> u16 {
        self.content_port.unwrap_or(DEFAULT_CONTENT_PORT)
    }

    pub fn hostname(&self) -> &str {
        self.hostname.as_deref().unwrap_or(DEFAULT_HOSTNAME)
    }
}
