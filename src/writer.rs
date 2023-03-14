use crate::common::{Header, EOF_BLOCK};
use std::{
    error::Error,
    fs::File,
    io::{copy, Write},
    path::{Path, PathBuf},
};

pub struct Writer {
    file: std::fs::File,
    paths: Vec<PathBuf>,
}

impl Writer {
    /// Creates a new writer with the destination being the path supplied.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Writer, std::io::Error> {
        Ok(Writer {
            file: File::create(path)?,
            paths: vec![],
        })
    }

    /// Lazily adds paths to the `Writer`. It merely tells the `Writer` to note the supplied path
    /// and does not write to the underlying file. To write to the underlying file, use the
    /// `write` method after `add`ing all the files.
    pub fn add<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref();
        // If the given path is a directory,
        // recursively add all the files and
        // subdirectories inside it.
        if path.is_dir() {
            for entry in path.read_dir()? {
                self.add(entry?.path())?;
            }
        } else if path.is_file() {
            self.paths.push(path.to_path_buf());
        }
        // Do not add symbolic links.
        Ok(())
    }

    /// Writes header structures and associated data to the underlying file handle. Since the
    /// object is consumed, the file is closed on drop, making sure we cannot incorrectly write
    /// multiple times to the same file.
    pub fn write(mut self) -> Result<(), Box<dyn Error>> {
        for path in self.paths.iter() {
            let header = Header::from_file(path)?;
            let mut handle = File::open(path)?;
            self.file.write(&header.bytes)?;
            copy(&mut handle, &mut self.file)?;
        }
        // This marks the end of the file.
        self.file.write(EOF_BLOCK)?;
        Ok(())
    }

    pub fn files_count(&self) -> usize {
        self.paths.len()
    }
}
