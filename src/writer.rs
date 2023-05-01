use crate::{
    common::{ArchiveError, Header, EOF_BLOCK},
    FileParseError,
};
use std::{
    fs::File,
    io::{copy, Write},
    path::{Path, PathBuf},
};

/// Structure to write multiple files and corresponding metadata into a wpress archive.
pub struct Writer {
    file: std::fs::File,
    paths: Vec<PathBuf>,
}
impl Writer {
    /// Creates a new `Writer` with the destination being the path supplied.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Writer, ArchiveError> {
        Ok(Writer {
            file: File::create(path).map_err(ArchiveError::FileCreation)?,
            paths: vec![],
        })
    }

    /// Lazily adds paths to the `Writer`. It merely tells the `Writer` to note the supplied path
    /// and does not write to the underlying file. To write to the underlying file, use the
    /// `write` method after `add`ing all the files.
    pub fn add<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ArchiveError> {
        let path = path.as_ref();
        // If the given path is a directory,
        // recursively add all the files and
        // subdirectories inside it.
        if path.is_dir() {
            for entry in path.read_dir().map_err(ArchiveError::EntryAddition)? {
                self.add(entry.map_err(ArchiveError::EntryAddition)?.path())?;
            }
        } else if path.is_file() {
            self.paths.push(path.to_path_buf());
        }
        // Do not add symbolic links or devices.
        Ok(())
    }

    /// Writes header structures and associated data to the underlying file handle. Since the
    /// object is consumed, the file is closed on drop, making sure we cannot incorrectly write
    /// multiple times to the same file.
    pub fn write(mut self) -> Result<(), ArchiveError> {
        for path in self.paths.iter() {
            let header = Header::from_file_metadata(path)?;
            let mut handle = File::open(path).map_err(FileParseError::FileRead)?;
            self.file
                .write_all(&header.bytes)
                .map_err(ArchiveError::FileWrite)?;
            copy(&mut handle, &mut self.file).map_err(ArchiveError::FileWrite)?;
        }
        // This marks the end of the file.
        self.file
            .write_all(EOF_BLOCK)
            .map_err(ArchiveError::FileWrite)?;
        Ok(())
    }

    pub fn files_count(&self) -> usize {
        self.paths.len()
    }
}
