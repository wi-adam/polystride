pub mod agent;
pub mod model;
pub mod runner;
pub mod store;
pub mod transport;

use std::io::{self, Read};
use std::path::Path;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[must_use]
pub fn error(message: &str) -> Box<dyn std::error::Error + Send + Sync> {
    io::Error::other(message).into()
}

/// Read a file while enforcing an upper bound on its size.
///
/// # Errors
///
/// Returns an error if the file cannot be read or exceeds `limit` bytes.
pub fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(error("file exceeds size limit"));
    }
    Ok(bytes)
}

/// Read a JSON file within the protocol's size limit.
///
/// # Errors
///
/// Returns an error for unreadable, oversized, or invalid JSON files.
pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(&read_bounded(
        path,
        model::MAX_FRAME,
    )?)?)
}

/// Publish a complete immutable file, without replacing an existing result/job.
///
/// # Errors
///
/// Returns an error if serialization, writing, syncing, or publication fails,
/// including when the destination already exists or has no parent directory.
pub fn publish_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    use std::io::Write;
    let mut temporary =
        tempfile::NamedTempFile::new_in(path.parent().ok_or_else(|| error("missing parent"))?)?;
    serde_json::to_writer(&mut temporary, value)?;
    temporary.write_all(b"\n")?;
    temporary.as_file().sync_all()?;
    temporary.persist_noclobber(path)?;
    Ok(())
}
