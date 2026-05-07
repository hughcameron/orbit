//! File-level locking for substrate-written entities.
//!
//! Per ac-03 + constraints.concurrency:
//! - Default timeout: 30 seconds (matches typical LLM tool-call latency)
//! - Lock files at `.orbit/locks/<key>.lock` contain `pid|nanos`
//! - Stale lock recovery: locks older than 3× the timeout are reclaimable
//! - Reads do not require lock acquisition; readers see slightly-stale views,
//!   never partial ones (the atomic-write contract guarantees this)
//!
//! The lock acquisition path uses `OpenOptions::new().create_new(true)` which
//! is the POSIX `O_EXCL`-equivalent — atomic across processes on a single
//! filesystem mount. NFS and FUSE are out of scope for v0.1; single-machine
//! single-mount is the constraint per `goal`.

use crate::error::{Category, Error, Result};
use crate::layout::OrbitLayout;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Default lock acquisition timeout, per spec constraints.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Multiplier applied to the timeout to determine when a lock is stale and
/// reclaimable. Per spec constraints.
pub const STALE_MULTIPLIER: u32 = 3;

/// RAII guard for a held file lock. Drop releases the lock by unlinking the
/// lock file. If unlinking fails (the file was removed externally), Drop
/// silently ignores it — the lock is effectively released either way.
#[derive(Debug)]
pub struct LockGuard {
    path: PathBuf,
    /// Set to `true` if the guard has been explicitly released; Drop is a no-op.
    released: bool,
}

impl LockGuard {
    /// Explicitly release the lock. Equivalent to letting the guard drop, but
    /// surfaces I/O errors instead of swallowing them.
    pub fn release(mut self) -> Result<()> {
        self.released = true;
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
            Err(e) => Err(Error::unavailable(
                "lock.release",
                format!("remove lock file failed: {e}"),
            )
            .with_source(e)),
        }
    }

    /// Path of the lock file (for diagnostic / test use).
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        if !self.released {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// Acquire an exclusive lock for `key` rooted at `layout.locks_dir()`.
///
/// Behaviour:
/// - Polls every 50ms (cheap, well under any realistic timeout).
/// - If a lock file exists and is younger than `STALE_MULTIPLIER * timeout`,
///   waits for it to release.
/// - If a lock file exists and is older than the staleness threshold,
///   reclaims it (unlinks the stale file, then re-attempts acquire).
/// - On total elapsed > `timeout`, returns [`Category::Locked`] with a
///   diagnostic message naming the holder pid and lock age.
pub fn acquire(layout: &OrbitLayout, key: &str, timeout: Duration) -> Result<LockGuard> {
    let path = lock_path(layout, key);
    // Make sure the locks directory exists; cheap idempotent op.
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::unavailable("lock.acquire", format!("create locks dir: {e}"))
                    .with_source(e)
            })?;
        }
    }

    let start = Instant::now();
    let stale_threshold = timeout * STALE_MULTIPLIER;
    let poll_interval = Duration::from_millis(50);

    loop {
        match try_create_lock(&path) {
            Ok(guard) => return Ok(guard),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                // Decide: wait, or reclaim stale?
                match read_lock_age(&path) {
                    Ok(Some(age)) if age >= stale_threshold => {
                        // Stale — reclaim. Best-effort unlink; on race with
                        // another reclaimer, the next loop iteration will
                        // either succeed in creating or find the new owner.
                        let _ = std::fs::remove_file(&path);
                        continue;
                    }
                    Ok(_) => {
                        // Held by a live (non-stale) lock — wait or time out.
                        if start.elapsed() >= timeout {
                            return Err(locked_error(&path, "lock held; timeout exceeded"));
                        }
                        std::thread::sleep(poll_interval);
                        continue;
                    }
                    Err(read_err) => {
                        // Couldn't read the lock — surface it.
                        return Err(read_err);
                    }
                }
            }
            Err(other) => {
                return Err(Error::unavailable(
                    "lock.acquire",
                    format!("create lock failed: {other}"),
                )
                .with_source(other));
            }
        }
    }
}

/// Acquire with the default timeout.
pub fn acquire_default(layout: &OrbitLayout, key: &str) -> Result<LockGuard> {
    acquire(layout, key, DEFAULT_TIMEOUT)
}

fn lock_path(layout: &OrbitLayout, key: &str) -> PathBuf {
    layout.locks_dir().join(format!("{key}.lock"))
}

fn try_create_lock(path: &Path) -> std::io::Result<LockGuard> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    write!(file, "{pid}|{nanos}")?;
    file.sync_all()?;
    Ok(LockGuard {
        path: path.to_path_buf(),
        released: false,
    })
}

/// Read the age of a lock file (now − file's recorded timestamp).
///
/// Returns `Ok(None)` if the lock file disappears between the existence check
/// and the read (race with release / reclaim).
fn read_lock_age(path: &Path) -> Result<Option<Duration>> {
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(Error::unavailable(
                "lock.acquire",
                format!("open lock for staleness check failed: {e}"),
            )
            .with_source(e));
        }
    };
    let mut buf = String::new();
    file.read_to_string(&mut buf).map_err(|e| {
        Error::unavailable("lock.acquire", format!("read lock failed: {e}")).with_source(e)
    })?;

    // Parse "pid|nanos". On malformed contents, treat as immediately stale —
    // the previous holder evidently crashed or something else corrupted it.
    let recorded_nanos = buf
        .split_once('|')
        .and_then(|(_, n)| n.trim().parse::<u128>().ok())
        .ok_or(())
        .or_else(|_| Ok::<u128, ()>(0))
        .unwrap_or(0);

    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let age_nanos = now_nanos.saturating_sub(recorded_nanos);
    let age = Duration::from_nanos(u64::try_from(age_nanos).unwrap_or(u64::MAX));
    Ok(Some(age))
}

fn locked_error(path: &Path, msg: &str) -> Error {
    let holder = std::fs::read_to_string(path).unwrap_or_default();
    Error::new(
        "lock.acquire",
        Category::Locked,
        format!("{msg} (lock at {}: {holder})", path.display()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn fresh_layout() -> (tempfile::TempDir, OrbitLayout) {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        (dir, layout)
    }

    #[test]
    fn acquire_release_basic() {
        let (_dir, layout) = fresh_layout();
        let guard = acquire_default(&layout, "spec-0001").unwrap();
        assert!(guard.path().exists());
        guard.release().unwrap();
        // After release the file is gone.
        let path = layout.locks_dir().join("spec-0001.lock");
        assert!(!path.exists());
    }

    #[test]
    fn drop_releases_lock() {
        let (_dir, layout) = fresh_layout();
        let path = layout.locks_dir().join("spec-0001.lock");
        {
            let _guard = acquire_default(&layout, "spec-0001").unwrap();
            assert!(path.exists());
        } // guard drops here
        assert!(!path.exists(), "drop must release the lock");
    }

    #[test]
    fn second_acquire_times_out_when_first_is_held() {
        let (_dir, layout) = fresh_layout();
        let _guard = acquire_default(&layout, "spec-0001").unwrap();

        // Use a tiny timeout so the test runs fast.
        let start = Instant::now();
        let err = acquire(&layout, "spec-0001", Duration::from_millis(200)).unwrap_err();
        let elapsed = start.elapsed();

        assert_eq!(err.category, Category::Locked);
        assert!(
            elapsed >= Duration::from_millis(180),
            "should have waited at least the timeout: {elapsed:?}"
        );
        assert!(
            elapsed < Duration::from_millis(800),
            "should not have waited far past the timeout: {elapsed:?}"
        );
    }

    #[test]
    fn stale_lock_is_reclaimed() {
        let (_dir, layout) = fresh_layout();
        let path = layout.locks_dir().join("spec-stale.lock");

        // Manually plant a stale lock file with a timestamp from years ago.
        std::fs::write(&path, "9999|1000000000000000000").unwrap();

        // Acquire with a short timeout. Because the planted lock is older than
        // 3× the timeout, it must be reclaimed and acquisition must succeed.
        let guard = acquire(&layout, "spec-stale", Duration::from_millis(100)).unwrap();
        assert!(guard.path().exists());
        // The lock now contains our pid (overwritten via reclaim path).
        let contents = std::fs::read_to_string(guard.path()).unwrap();
        let our_pid = std::process::id().to_string();
        assert!(
            contents.starts_with(&our_pid),
            "reclaimed lock should be owned by us: got {contents}"
        );
    }

    #[test]
    fn reads_do_not_require_lock_acquisition() {
        // ac-03 read consistency: reads do not require lock acquisition. We
        // verify this property by holding a lock and reading the underlying
        // canonical file directly — the read must succeed with no lock work.
        let (dir, layout) = fresh_layout();
        let target = layout.spec_file("0001");
        std::fs::write(&target, "id: '0001'\nstatus: open\n").unwrap();

        let _writer_guard = acquire_default(&layout, "spec-0001").unwrap();

        // Reader path: just read the file. No lock involved.
        let read_text = std::fs::read_to_string(&target).unwrap();
        assert!(read_text.contains("0001"));

        // Sanity: the lock dir is in the right place under the temp.
        assert!(layout.locks_dir().starts_with(dir.path()));
    }

    #[test]
    fn two_keys_do_not_block_each_other() {
        let (_dir, layout) = fresh_layout();
        let g1 = acquire_default(&layout, "spec-0001").unwrap();
        let g2 = acquire_default(&layout, "spec-0002").unwrap();
        assert!(g1.path().exists());
        assert!(g2.path().exists());
    }

    #[test]
    fn loud_failure_message_names_holder() {
        let (_dir, layout) = fresh_layout();
        let _guard = acquire_default(&layout, "spec-0001").unwrap();
        let err = acquire(&layout, "spec-0001", Duration::from_millis(50)).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("locked"));
        assert!(
            msg.contains(".lock"),
            "error must name the lock path: {msg}"
        );
    }
}
