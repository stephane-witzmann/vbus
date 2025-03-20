use crate::Error;
use std::env::temp_dir;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicUsize;

pub struct TempFile {
    path_buf: PathBuf,
}

impl TempFile {
    pub fn new() -> Result<Self, Error> {
        TempFile::new_named(&next_name())
    }

    fn new_named(basename: &str) -> Result<Self, Error> {
        let mut path_buf = temp_dir();
        path_buf.push(basename);

        let path = path_buf.as_path();

        remove_if_exists(path)?;

        Ok(Self { path_buf })
    }

    pub fn path(&self) -> &Path {
        self.path_buf.as_path()
    }
}

impl Deref for TempFile {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        self.path()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        remove_if_exists(self.path()).unwrap(); // panic if we're not able to remove our own file
    }
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn next_name() -> String {
    /*
     * TODO: add some process-specific hash. If tests are run concurrently,
     * they will end up with the same file names.
     */
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    format!("vbus-tmp-{}", n)
}

fn remove_if_exists(path: &Path) -> Result<(), std::io::Error> {
    if path.exists() {
        std::fs::remove_file(path)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{TempFile, next_name};
    use std::fs::File;

    #[test]
    fn test_removal() {
        let name = next_name();
        let temp_file = TempFile::new_named(&name).unwrap();

        // Create file, panic if already exists
        File::create_new(temp_file.path()).unwrap();
        assert!(temp_file.path().exists());

        // Check if removed after we're dropped
        let path_buf = temp_file.path().to_path_buf();
        drop(temp_file);
        assert!(!path_buf.exists());

        // Recreate file
        File::create_new(&path_buf).unwrap();
        assert!(&path_buf.exists());

        // Call constructor and check if the file has been removed
        let _temp_file = TempFile::new_named(&name).unwrap();
        assert!(!&path_buf.exists());
    }

    fn path_from_new() -> String {
        TempFile::new().unwrap().path().to_str().unwrap().to_owned()
    }

    #[test]
    fn test_auto_name() {
        assert_ne!(path_from_new(), path_from_new());
    }
}
