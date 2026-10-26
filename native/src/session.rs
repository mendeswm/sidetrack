use std::path::PathBuf;
use std::sync::{Arc, OnceLock, Mutex as StdMutex};

use librespot_core::SessionConfig;
use librespot_core::cache::Cache;
use librespot_core::authentication::Credentials;
use librespot_core::Session;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::error::{SidetrackError, Result};

/// Global tokio runtime shared across the native library.
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Global session state.
static SESSION: OnceLock<Arc<Mutex<Option<Session>>>> = OnceLock::new();

/// Custom tmp dir set from Android (app cache dir).
static TMP_DIR: OnceLock<StdMutex<Option<PathBuf>>> = OnceLock::new();

/// Audio cache directory (optional, from SD storage).
static AUDIO_CACHE_DIR: OnceLock<StdMutex<Option<PathBuf>>> = OnceLock::new();

/// Audio cache size limit in bytes (optional).
static AUDIO_CACHE_LIMIT: OnceLock<StdMutex<Option<u64>>> = OnceLock::new();

/// Set the temporary directory for librespot file downloads.
pub fn set_tmp_dir(path: &str) {
    let slot = TMP_DIR.get_or_init(|| StdMutex::new(None));
    *slot.lock().unwrap() = Some(PathBuf::from(path));
    log::info!("tmp_dir set to: {path}");
}

fn get_tmp_dir() -> Option<PathBuf> {
    TMP_DIR.get().and_then(|m| m.lock().unwrap().clone())
}

fn get_audio_cache_dir() -> Option<PathBuf> {
    AUDIO_CACHE_DIR.get().and_then(|m| m.lock().unwrap().clone())
}

fn get_audio_cache_limit() -> Option<u64> {
    AUDIO_CACHE_LIMIT.get().and_then(|m| m.lock().unwrap().clone())
}

/// Set audio cache directory and size limit.
/// @param dir absolute path; empty means disable.
/// @param limit_mb megabytes.
    pub fn set_audio_cache(dir: &str, limit_mb: u64) -> Result<()> {
    // Use manual error mapping to avoid adding From<std::io::Error>
    if dir.is_empty() {
        let slot = AUDIO_CACHE_DIR.get_or_init(|| StdMutex::new(None));
        *slot.lock().unwrap() = None;
        let limit_slot = AUDIO_CACHE_LIMIT.get_or_init(|| StdMutex::new(None));
        *limit_slot.lock().unwrap() = None;
        log::info!("audio cache disabled");
        return Ok(());
    }
    let path = PathBuf::from(dir);
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Err(SidetrackError::Session(format!("cache parent dir failed: {e}")));
        }
    }
    if let Err(e) = std::fs::create_dir_all(&path) {
        return Err(SidetrackError::Session(format!("cache dir failed: {e}")));
    }
    let slot = AUDIO_CACHE_DIR.get_or_init(|| StdMutex::new(None));
    *slot.lock().unwrap() = Some(path);
    let limit_slot = AUDIO_CACHE_LIMIT.get_or_init(|| StdMutex::new(None));
    *limit_slot.lock().unwrap() = Some(limit_mb * 1024 * 1024);
        log::info!("audio cache set to: {}, limit: {} MB", dir, limit_mb);
    Ok(())
}

pub fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("sidetrack-rt")
            .build()
            .expect("failed to create tokio runtime")
    })
}

fn session_slot() -> &'static Arc<Mutex<Option<Session>>> {
    SESSION.get_or_init(|| Arc::new(Mutex::new(None)))
}

/// Connect to Spotify with the given access token.
/// Returns Ok(()) on success, stores the session globally.
pub async fn connect_with_token(access_token: &str) -> Result<()> {
    connect_with_credentials(Credentials::with_access_token(access_token)).await
}

/// Connect to Spotify with already-built credentials (OAuth access token or
/// Zeroconf-derived stored credentials). Returns Ok(()) on success, stores the
/// session globally.
pub async fn connect_with_credentials(credentials: Credentials) -> Result<()> {
    let mut config = SessionConfig::default();
    if let Some(tmp) = get_tmp_dir() {
        config.tmp_dir = tmp;
    }

    // Apply autoplay setting from stored config
    {
        let app_cfg = crate::player::config_slot().lock().unwrap();
        if let Some(autoplay) = app_cfg.autoplay {
            config.autoplay = Some(autoplay);
        }
    }

    // Build audio cache if configured
    let cache = get_audio_cache_dir().and_then(|dir| {
        let limit = get_audio_cache_limit()?;
        let cache_dir = dir.clone();
        match Cache::new(
            None::<&std::path::Path>,
            None::<&std::path::Path>,
            Some(&cache_dir),
            Some(limit),
        ) {
            Ok(c) => {
                log::info!("audio cache enabled at: {}", dir.display());
                Some(c)
            }
            Err(e) => {
                log::warn!("audio cache init failed: {e}");
                None
            }
        }
    });

    let session = Session::new(config, cache);

    session
        .connect(credentials, true)
        .await
        .map_err(|e| SidetrackError::Session(format!("connect failed: {e}")))?;

    log::info!("Spotify session connected successfully");

    let mut slot = session_slot().lock().await;
    *slot = Some(session);

    Ok(())
}

/// Connect using a Zeroconf-derived stored credential blob (username +
/// base64 auth_data + protobuf AuthenticationType value), as persisted from
/// a previous [`crate::discovery`] pairing.
pub async fn connect_with_stored_credentials(
    username: String,
    auth_data_b64: String,
    auth_type_value: i32,
) -> Result<()> {
    use base64::Engine as _;
    use protobuf::Enum;

    let auth_data = base64::engine::general_purpose::STANDARD
        .decode(auth_data_b64)
        .map_err(|e| SidetrackError::Session(format!("invalid stored credentials: {e}")))?;
    let auth_type = librespot_protocol::authentication::AuthenticationType::from_i32(
        auth_type_value,
    )
    .ok_or_else(|| SidetrackError::Session("invalid stored credentials: bad auth type".into()))?;

    connect_with_credentials(Credentials {
        username: Some(username),
        auth_type,
        auth_data,
    })
    .await
}

/// Disconnect the current session.
pub async fn disconnect() {
    let mut slot = session_slot().lock().await;
    if let Some(session) = slot.take() {
        session.shutdown();
        log::info!("Spotify session disconnected");
    }
}

/// Get a clone of the current session, if connected.
pub async fn get_session() -> Result<Session> {
    let slot = session_slot().lock().await;
    slot.clone().ok_or(SidetrackError::NoSession)
}

/// Check if a session is currently active.
pub async fn is_connected() -> bool {
    let slot = session_slot().lock().await;
    slot.is_some()
}
