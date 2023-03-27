use std::{
    io::{Cursor, Seek, SeekFrom, Write},
    path::{Path, StripPrefixError},
    string::FromUtf8Error,
    time::SystemTime,
};
use thiserror::Error;

pub const HEADER_SIZE: usize = 4377;
const FILENAME: usize = 255;
const SIZE_BEGIN: usize = FILENAME;
const SIZE: usize = 14;
const MTIME: usize = 12;
const PREFIX: usize = 4096;
const SIZE_END: usize = FILENAME + SIZE;
const MTIME_BEGIN: usize = SIZE_END;
const MTIME_END: usize = SIZE_END + MTIME;
const PREFIX_BEGIN: usize = MTIME_END;
pub const EOF_BLOCK: &[u8] = &[0; HEADER_SIZE];

#[derive(Error, Debug)]
pub enum FileParseError {
    #[error("failed to read file metadata")]
    Metadata,
    #[error("failed to read file name")]
    EmptyName,
    #[error("failed to read last modified time for file")]
    ReadLastModified,
    #[error("failed to cast last modified date in terms of unix epoch for file")]
    UnixEpoch,
    #[error("{0}")]
    Length(#[from] LengthExceededError),
    #[error("{0}")]
    Header(#[from] HeaderError),
    #[error("failed reading from file: {0}")]
    FileRead(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("failed to strip path prefix and sanitize it: {0}")]
    PathSanitization(#[from] StripPrefixError),
    #[error("failed writing to file: {0}")]
    FileRead(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("failed to create archive file: {0}")]
    FileCreation(std::io::Error),
    #[error("failed to add file entry to archive: {0}")]
    EntryAddition(std::io::Error),
    #[error("failed to traverse and recursively add files to archive: {0}")]
    DirectoryTraversal(std::io::Error),
    #[error("{0}")]
    FileParse(#[from] FileParseError),
    #[error("failed writing to archive: {0}")]
    FileWrite(std::io::Error),
}

#[derive(Error, Debug)]
pub enum LengthExceededError {
    #[error("Filename is longer than the maximum of 255 bytes")]
    Name,
    #[error("String representation of the file's size exceeds the maximum of 14 bytes")]
    Size,
    #[error(
        "String representation of the file's UNIX modified time exceeds the maximum of 12 bytes"
    )]
    Mtime,
    #[error(
        "String representation of the file's parent directories exceeds the maximum of 4096 bytes"
    )]
    Prefix,
}
/// Metadata representation of a file with attributes necessary for an archive entry.
#[derive(Clone, Debug)]
pub struct Header {
    /// Base name of the file from an entry.
    pub name: String,
    /// Size of the file in bytes.
    pub size: u64,
    /// Last modified time relative to UNIX epochs.
    pub mtime: u64,
    /// Path of the file without the final component, its name.
    pub prefix: String,
    /// A representation of `name`, `size`, `mtime` and `perfix` in a blob of bytes.
    /// Each field is zero padded to meets predefined boundaries.
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum Field {
    Name,
    Size,
    Mtime,
    Prefix,
}

#[derive(Error, Debug)]
pub enum HeaderError {
    #[error("failed parsing block: {0}")]
    BlockParseError(#[from] BlockParseError),
    #[error("header ended prematurely")]
    IncompleteHeader,
}

#[derive(Error, Debug)]
pub enum BlockParseError {
    #[error("failed to parse field {0:?} from block as utf-8 string")]
    FromUtf8Error(Field),
    #[error("failed to parse field {0:?} from utf-8 string as unsigned 64 bit integer")]
    IntoU64Error(Field),
}

fn read_block(block: &[u8], lower: usize, upper: usize) -> Result<String, FromUtf8Error> {
    String::from_utf8(
        block[lower..upper]
            .iter()
            .take_while(|c| **c != 0)
            .copied()
            .collect(),
    )
}

impl Header {
    /// Parse an archive metadata entry for a file from a block of bytes.
    pub fn from_bytes(block: &[u8]) -> Result<Header, HeaderError> {
        Ok(Header {
            name: read_block(block, 0, FILENAME)
                .map_err(|_| BlockParseError::FromUtf8Error(Field::Name))?,
            size: read_block(block, SIZE_BEGIN, SIZE_END)
                .map_err(|_| BlockParseError::FromUtf8Error(Field::Size))?
                .parse()
                .map_err(|_| BlockParseError::IntoU64Error(Field::Size))?,
            mtime: read_block(block, MTIME_BEGIN, MTIME_END)
                .map_err(|_| BlockParseError::FromUtf8Error(Field::Mtime))?
                .parse()
                .map_err(|_| BlockParseError::IntoU64Error(Field::Mtime))?,
            prefix: read_block(block, PREFIX_BEGIN, HEADER_SIZE)
                .map_err(|_| BlockParseError::FromUtf8Error(Field::Prefix))?,
            bytes: block.to_owned(),
        })
    }

    /// Generate an archive metadata entry for a file given its path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Header, FileParseError> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path).map_err(|_| FileParseError::Metadata)?;

        let name = path.file_name().ok_or(FileParseError::EmptyName)?;
        FILENAME
            .checked_sub(name.len())
            .ok_or(LengthExceededError::Name)?;

        let name = name.to_string_lossy().to_string();

        let size = metadata.len();
        let size_str = size.to_string();
        SIZE.checked_sub(size_str.len())
            .ok_or(LengthExceededError::Size)?;

        let mtime = metadata
            .modified()
            .map_err(|_| FileParseError::ReadLastModified)?
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| FileParseError::UnixEpoch)?
            .as_secs();
        let mtime_str = mtime.to_string();
        MTIME
            .checked_sub(mtime_str.len())
            .ok_or(LengthExceededError::Mtime)?;

        let prefix = path
            .parent()
            .map_or(String::from(""), |p| p.to_string_lossy().to_string());
        PREFIX
            .checked_sub(prefix.len())
            .ok_or(LengthExceededError::Prefix)?;

        let mut bytes = Cursor::new(vec![0u8; HEADER_SIZE]);

        // If any of the following fails, panic. Something is very wrong.
        bytes.write_all(name.as_bytes()).unwrap();
        bytes.seek(SeekFrom::Start(FILENAME as u64)).unwrap();
        bytes.write_all(size_str.as_bytes()).unwrap();
        bytes.seek(SeekFrom::Start(SIZE_END as u64)).unwrap();
        bytes.write_all(mtime_str.as_bytes()).unwrap();
        bytes.seek(SeekFrom::Start(MTIME_END as u64)).unwrap();
        bytes.write_all(prefix.as_bytes()).unwrap();

        let bytes = bytes.into_inner();

        Ok(Header {
            name,
            size,
            mtime,
            prefix,
            bytes,
        })
    }
}
