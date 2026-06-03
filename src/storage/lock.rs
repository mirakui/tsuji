use std::fs::File;
use std::io;

use fs2::FileExt;

/// RAII guard wrapping an exclusive `flock(2)` advisory lock on a file.
///
/// The lock is released when the guard is dropped — including on panic.
pub struct ExclusiveLockGuard<'a> {
    file: &'a File,
}

impl<'a> ExclusiveLockGuard<'a> {
    /// Blocks until the exclusive lock is held, then returns the guard.
    pub fn acquire(file: &'a File) -> io::Result<Self> {
        file.lock_exclusive()?;
        Ok(Self { file })
    }
}

impl Drop for ExclusiveLockGuard<'_> {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn lock_can_be_acquired_and_released() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        let f = File::create(&path).unwrap();
        {
            let _g = ExclusiveLockGuard::acquire(&f).unwrap();
            let mut f2 = File::options().append(true).open(&path).unwrap();
            f2.write_all(b"x").unwrap();
        }
        // After guard drops, lock should be released; a new acquire works.
        let _g = ExclusiveLockGuard::acquire(&f).unwrap();
    }
}
