use std::{
    error::Error,
    fmt,
    io::{Cursor, Seek, SeekFrom, Write},
    os::unix::prelude::MetadataExt,
    path::Path,
};

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

#[derive(Debug)]
pub enum ParseError {
    NameIsNone,
    NameLengthExceeded,
    SizeLengthExceeded,
    MtimeLengthExceeded,
    PrefixLengthExceeded,
    IncompleteHeader,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            Self::NameIsNone => "File has no associated name: expected Some, got None",
            Self::NameLengthExceeded => "Filename is longerthan the maximum of 255 bytes",
            Self::SizeLengthExceeded => "String representation of the file's size exceeds the maximum of 14 bytes",
            Self::MtimeLengthExceeded => "String representation of the file's UNIX modified time exceeds the maximum of 12 bytes",
            Self::PrefixLengthExceeded => "String representation of the file's parent directories exceeds the maximum of 4096 bytes",
                Self::IncompleteHeader => "While reading file: Header block ended prematurely."
        };
        write!(f, "{}", err)
    }
}

impl Error for ParseError {}

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Clone, Debug)]
pub struct Header {
    pub name: String,
    pub size: u64,
    pub mtime: i64,
    pub prefix: String,
    pub bytes: Vec<u8>,
}

fn read_block(block: &[u8], lower: usize, upper: usize) -> Result<String> {
    Ok(String::from_utf8(
        block[lower..upper]
            .iter()
            .take_while(|c| **c != 0)
            .copied()
            .collect(),
    )?)
}

impl Header {
    pub fn from_bytes(block: &[u8]) -> Result<Header> {
        Ok(Header {
            name: read_block(block, 0, FILENAME)?,
            size: read_block(block, SIZE_BEGIN, SIZE_END)?.parse()?,
            mtime: read_block(block, MTIME_BEGIN, MTIME_END)?.parse()?,
            prefix: read_block(block, PREFIX_BEGIN, HEADER_SIZE)?,
            bytes: block.to_owned(),
        })
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Header> {
        let metadata = std::fs::metadata(&path)?;
        let path = path.as_ref();

        let name = path.file_name().ok_or(ParseError::NameIsNone)?;
        FILENAME
            .checked_sub(name.len())
            .ok_or(ParseError::NameLengthExceeded)?;

        let name = name.to_string_lossy().to_string();

        let size = metadata.size();
        let size_str = size.to_string();
        SIZE.checked_sub(size_str.len())
            .ok_or(ParseError::SizeLengthExceeded)?;

        let mtime = metadata.mtime();
        let mtime_str = mtime.to_string();
        MTIME
            .checked_sub(mtime_str.len())
            .ok_or(ParseError::MtimeLengthExceeded)?;

        let prefix = path
            .parent()
            .map_or(String::from(""), |p| p.to_string_lossy().to_string());
        PREFIX
            .checked_sub(prefix.len())
            .ok_or(ParseError::PrefixLengthExceeded)?;

        let mut bytes = Cursor::new(vec![0u8; HEADER_SIZE]);
        bytes.write_all(name.as_bytes())?;
        bytes.seek(SeekFrom::Start(FILENAME as u64))?;
        bytes.write_all(size_str.as_bytes())?;
        bytes.seek(SeekFrom::Start(SIZE_END as u64))?;
        bytes.write_all(mtime_str.as_bytes())?;
        bytes.seek(SeekFrom::Start(MTIME_END as u64))?;
        bytes.write_all(prefix.as_bytes())?;

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
