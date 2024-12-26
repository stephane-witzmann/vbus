use crate::error::Error;
use std::env::temp_dir;
use std::ops::Deref;
use std::path::{Path, PathBuf};

pub(crate) struct TempFile {
    path_buf: PathBuf,
}

impl TempFile {
    pub fn new(basename: &str) -> Result<Self, Error> {
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

fn remove_if_exists(path: &Path) -> Result<(), std::io::Error> {
    if path.exists() {
        std::fs::remove_file(path)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::temp_file::TempFile;
    use std::fs::File;

    #[test]
    fn test_removal() {
        const BASENAME: &str = "temp-removal-test";

        let temp_file = TempFile::new(BASENAME).unwrap();

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
        let _temp_file = TempFile::new(BASENAME).unwrap();
        assert!(!&path_buf.exists());
    }
}
