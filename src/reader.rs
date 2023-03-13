use crate::common::{FileError, Header, HEADER_SIZE};
use clean_path::Clean;
use std::error::Error;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::{fs, io};
pub struct Reader {
    file: std::fs::File,
    count: usize,
}

impl Reader {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Reader, Box<dyn Error>> {
        let file = std::fs::File::open(path)?;
        Ok(Reader { file, count: 0 })
    }

    pub fn extract(&mut self) -> Result<usize, Box<dyn Error>> {
        self.file.rewind()?;
        loop {
            let block = self.header_block()?;
            if Header::eof_block() == block {
                break;
            }
            let header = Header::from_bytes(&block)?;
            let path = Path::new(".").join(
                [&header.prefix, &header.name]
                    .iter()
                    .collect::<PathBuf>()
                    .clean()
                    .strip_prefix("/")?,
            );
            let dir = path.parent().unwrap_or(Path::new("."));
            fs::create_dir_all(dir)?;
            let mut handle = fs::File::create(path)?;
            let mut size = header.size;
            while size != 0 {
                let to_read = std::cmp::min(size, 512);
                io::copy(&mut (&mut self.file).take(to_read), &mut handle)?;
                size -= to_read;
            }
            self.count += 1;
        }
        Ok(self.count)
    }

    pub fn header_block(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf = vec![0; HEADER_SIZE];
        if HEADER_SIZE != self.file.read(&mut buf)? {
            Err(FileError::IncompleteHeader)?;
        }
        Ok(buf)
    }
}
