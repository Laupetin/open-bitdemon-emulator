use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyInit};
use aes::Aes256;
use cbc::cipher::block_padding::ZeroPadding;
use log::info;
use rand::RngCore;
use snafu::Snafu;
use std::error::Error;
use std::sync::RwLock;

pub type AesKey = [u8; 32];

pub struct BackendPrivateKey {
    aes_key: AesKey,
}

#[derive(Debug, Snafu)]
#[snafu(display("The buffer size must be multiple of AES block size"))]
struct BufferSizeError {}

impl BackendPrivateKey {
    pub fn encrypt_data(&self, buf: &mut [u8]) -> Result<(), Box<dyn Error>> {
        let cipher = Aes256::new_from_slice(&self.aes_key).unwrap();
        cipher
            .encrypt_padded_mut::<ZeroPadding>(buf, buf.len())
            .map(|_| ())
            .map_err(|e| {
                info!("{e}");
                BufferSizeSnafu {}.build().into()
            })
    }

    pub fn decrypt_data(&self, buf: &mut [u8]) -> Result<(), Box<dyn Error>> {
        let cipher = Aes256::new_from_slice(&self.aes_key).unwrap();
        cipher
            .decrypt_padded_mut::<ZeroPadding>(buf)
            .map(|_| ())
            .map_err(|_| BufferSizeSnafu {}.build().into())
    }
}

pub trait BackendPrivateKeyStorage {
    fn get_current_key(&self) -> BackendPrivateKey;
    fn get_valid_keys(&self) -> Vec<BackendPrivateKey>;
}

pub type ThreadSafeBackendPrivateKeyStorage = dyn BackendPrivateKeyStorage + Sync + Send;

/// How long each key lives
const IN_MEMORY_KEY_LIFESPAN: i64 = 15 * 60; // 15 min

/// How much in advance a key should no longer be used
const IN_MEMORY_KEY_TIMEOUT: i64 = 14 * 60; // 6 min
const MAX_CONCURRENTLY_VALID_KEYS: usize =
    (IN_MEMORY_KEY_LIFESPAN / (IN_MEMORY_KEY_LIFESPAN - IN_MEMORY_KEY_TIMEOUT)) as usize;
const IN_MEMORY_KEY_STORAGE_COUNT: usize = MAX_CONCURRENTLY_VALID_KEYS + 1;

pub struct InMemoryKeyStore {
    state: RwLock<InMemoryKeyState>,
}

impl Default for InMemoryKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryKeyStore {
    pub fn new() -> InMemoryKeyStore {
        InMemoryKeyStore {
            state: RwLock::new(InMemoryKeyState {
                keys: [InMemoryKey::empty(); IN_MEMORY_KEY_STORAGE_COUNT],
                key_index: 0,
            }),
        }
    }
}

struct InMemoryKeyState {
    keys: [InMemoryKey; IN_MEMORY_KEY_STORAGE_COUNT],
    key_index: usize,
}

impl BackendPrivateKeyStorage for InMemoryKeyStore {
    fn get_current_key(&self) -> BackendPrivateKey {
        let now = chrono::Utc::now().timestamp();
        let min_lifespan = now + IN_MEMORY_KEY_TIMEOUT;

        let mut state = self.state.write().unwrap();

        let current_key = &state.keys[state.key_index];

        if current_key.valid_until >= min_lifespan {
            return current_key.export();
        }

        info!("Current key reached end of lifetime, creating a new one");

        state.key_index = (state.key_index + 1) % IN_MEMORY_KEY_STORAGE_COUNT;

        let mut aes_key = [0u8; 32];
        rand::rng().fill_bytes(&mut aes_key);
        let next_key = InMemoryKey {
            aes_key,
            valid_until: now + IN_MEMORY_KEY_LIFESPAN,
        };

        let key_index = state.key_index;
        state.keys[key_index] = next_key;

        next_key.export()
    }

    fn get_valid_keys(&self) -> Vec<BackendPrivateKey> {
        let now = chrono::Utc::now().timestamp();
        let state = self.state.read().unwrap();

        state
            .keys
            .iter()
            .filter(|key| key.valid_until >= now)
            .map(|key| key.export())
            .collect()
    }
}

#[derive(Copy, Clone)]
struct InMemoryKey {
    aes_key: AesKey,
    valid_until: i64,
}

impl InMemoryKey {
    fn empty() -> InMemoryKey {
        InMemoryKey {
            aes_key: [0; 32],
            valid_until: 0,
        }
    }

    fn export(&self) -> BackendPrivateKey {
        BackendPrivateKey {
            aes_key: self.aes_key,
        }
    }
}
