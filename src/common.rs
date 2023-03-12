use std::{error::Error, fmt, os::unix::prelude::MetadataExt, path::Path};

const HEADER_SIZE: usize = 4377;
const FILENAME_SIZE: usize = 255;
const CONTENT_SIZE: usize = 14;
const MTIME_SIZE: usize = 12;
const PREFIX_SIZE: usize = 4096;
const SIZE_BEGIN: usize = FILENAME_SIZE + CONTENT_SIZE;
const MTIME_BEGIN: usize = SIZE_BEGIN + MTIME_SIZE;

enum FileError {
    NameIsNone,
    NameLengthExceeded,
    SizeLengthExceeded,
    MtimeLengthExceeded,
    PrefixLengthExceeded,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            Self::NameIsNone => "File has no associated name: expected Some, got None",
            Self::NameLengthExceeded => "Filename is longerthan the maximum of 255 bytes",
            Self::SizeLengthExceeded => "String representation of the file's size exceeds the maximum of 14 bytes",
            Self::MtimeLengthExceeded => "String representation of the file's UNIX modified time exceeds the maximum of 12 bytes",
            Self::PrefixLengthExceeded => "String representation of the file's parent directories exceeds the maximum of 4096 bytes"
        };
        write!(f, "{}", err)
    }
}

impl fmt::Debug for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!()) // programmer-facing output
    }
}

impl std::error::Error for FileError {}

pub struct Header {
    pub name: String,
    pub size: u64,
    pub mtime: i64,
    pub prefix: String,
    bytes: Vec<u8>,
}

impl Header {
    pub fn from_bytes(block: &Vec<u8>) -> Result<Header, Box<dyn Error>> {
        Ok(Header {
            name: String::from_utf8_lossy(&block[0..FILENAME_SIZE]).to_string(),
            size: String::from_utf8_lossy(&block[FILENAME_SIZE..SIZE_BEGIN]).parse()?,
            mtime: String::from_utf8_lossy(&block[SIZE_BEGIN..MTIME_BEGIN]).parse()?,
            prefix: String::from_utf8_lossy(&block[MTIME_BEGIN..HEADER_SIZE]).to_string(),
            bytes: block.clone(),
        })
    }
    pub fn eof_block(self) -> Vec<u8> {
        vec![0; HEADER_SIZE]
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Header, Box<dyn Error>> {
        let metadata = std::fs::metadata(&path)?;
        let name = path.as_ref().file_name().ok_or(FileError::NameIsNone)?;
        let name_len_diff = FILENAME_SIZE as isize - name.len() as isize;
        if name_len_diff < 0 {
            Err(FileError::NameLengthExceeded)?;
        }
        let name_len_diff = name_len_diff as usize;
        let name = name.to_string_lossy().to_string();
        let size = metadata.size();
        let size_str = size.to_string();
        let size_str_len = size_str.len();
        let size_str_len_diff = CONTENT_SIZE as isize - size_str_len as isize;
        if size_str_len_diff < 0 {
            Err(FileError::SizeLengthExceeded)?;
        }
        let size_str_len_diff = size_str_len_diff as usize;
        let mtime = metadata.mtime();
        let mtime_str = mtime.to_string();
        let mtime_str_len = mtime_str.len();
        let mtime_str_len_diff = MTIME_SIZE as isize - mtime_str_len as isize;
        if mtime_str_len_diff < 0 {
            Err(FileError::MtimeLengthExceeded)?;
        }
        let mtime_str_len_diff = mtime_str_len_diff as usize;
        let prefix = path
            .as_ref()
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(String::from(""));

        let prefix_len_diff = PREFIX_SIZE as isize - prefix.len() as isize;
        if prefix_len_diff < 0 {
            Err(FileError::PrefixLengthExceeded)?;
        }

        let mut bytes = name.clone().into_bytes();
        bytes.extend(std::iter::repeat(0).take(name_len_diff));
        bytes.extend(size_str.into_bytes());
        bytes.extend(std::iter::repeat(0).take(size_str_len_diff));
        bytes.extend(mtime_str.into_bytes());
        bytes.extend(std::iter::repeat(0).take(mtime_str_len_diff));
        bytes.extend(prefix.clone().into_bytes());

        Ok(Header {
            name,
            size,
            mtime,
            prefix,
            bytes,
        })
    }
}
