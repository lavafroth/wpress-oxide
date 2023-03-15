use std::{error::Error, fmt, os::unix::prelude::MetadataExt, path::Path};

pub const HEADER_SIZE: usize = 4377;
const FILENAME_SIZE: usize = 255;
const CONTENT_SIZE: usize = 14;
const MTIME_SIZE: usize = 12;
const PREFIX_SIZE: usize = 4096;
const SIZE_BEGIN: usize = FILENAME_SIZE + CONTENT_SIZE;
const MTIME_BEGIN: usize = SIZE_BEGIN + MTIME_SIZE;
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

fn block_to_string(block: &[u8], lower: usize, upper: usize) -> Result<String> {
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
            name: block_to_string(block, 0, FILENAME_SIZE)?,
            size: block_to_string(block, FILENAME_SIZE, SIZE_BEGIN)?.parse()?,
            mtime: block_to_string(block, SIZE_BEGIN, MTIME_BEGIN)?.parse()?,
            prefix: block_to_string(block, MTIME_BEGIN, HEADER_SIZE)?,
            bytes: block.to_owned(),
        })
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Header> {
        let metadata = std::fs::metadata(&path)?;
        let path = path.as_ref();

        let name = path.file_name().ok_or(ParseError::NameIsNone)?;
        let name_len_diff = FILENAME_SIZE
            .checked_sub(name.len())
            .ok_or(ParseError::NameLengthExceeded)?;

        let name = name.to_string_lossy().to_string();

        let size = metadata.size();
        let size_str = size.to_string().into_bytes();
        let size_len_diff = CONTENT_SIZE
            .checked_sub(size_str.len())
            .ok_or(ParseError::SizeLengthExceeded)?;

        let mtime = metadata.mtime();
        let mtime_str = mtime.to_string().into_bytes();
        let mtime_len_diff = MTIME_SIZE
            .checked_sub(mtime_str.len())
            .ok_or(ParseError::MtimeLengthExceeded)?;

        let prefix = path
            .parent()
            .map_or(String::from(""), |p| p.to_string_lossy().to_string());
        let prefix_len_diff = PREFIX_SIZE
            .checked_sub(prefix.len())
            .ok_or(ParseError::PrefixLengthExceeded)?;

        let bytes: Vec<u8> = [
            name.clone().into_bytes(),
            vec![0; name_len_diff],
            size_str,
            vec![0; size_len_diff],
            mtime_str,
            vec![0; mtime_len_diff],
            prefix.clone().into_bytes(),
            vec![0; prefix_len_diff],
        ]
        .into_iter()
        .flatten()
        .collect();

        Ok(Header {
            name,
            size,
            mtime,
            prefix,
            bytes,
        })
    }
}
