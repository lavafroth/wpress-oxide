use crate::common::{Header, ParseError, Result, EOF_BLOCK, HEADER_SIZE};
use clean_path::Clean;
use std::{
    fs::{create_dir_all, File},
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

pub struct Reader {
    file: std::fs::File,
    headers: Vec<Header>,
}

fn trim_clean<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let cleaned = path.as_ref().clean();
    if cleaned.starts_with("/") {
        return Ok(cleaned.strip_prefix("/")?.to_path_buf());
    }
    Ok(cleaned)
}

impl Reader {
    /// Creates a new `Reader` with the path supplied as the source file.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Reader> {
        let mut file = std::fs::File::open(path)?;
        let mut headers = Vec::new();
        let mut buf = vec![0; HEADER_SIZE];
        loop {
            if HEADER_SIZE != file.read(&mut buf)? {
                Err(ParseError::IncompleteHeader)?;
            }
            if EOF_BLOCK == buf {
                break;
            }
            let header = Header::from_bytes(&buf)?;
            let next_header = header.size as i64;
            headers.push(header);
            file.seek(SeekFrom::Current(next_header))?;
        }
        Ok(Reader { file, headers })
    }

    /// Extracts all the files inside the archive to the provided destination directory.
    pub fn extract_to<P: AsRef<Path>>(&mut self, destination: P) -> Result<()> {
        let destination = destination.as_ref();
        self.file.rewind()?;
        for header in self.headers.iter() {
            self.file.seek(io::SeekFrom::Current(HEADER_SIZE as i64))?;
            let clean = trim_clean([&header.prefix, &header.name].iter().collect::<PathBuf>())?;
            let path = Path::new(destination).join(clean);
            let dir = path.parent().unwrap_or(Path::new(destination));
            create_dir_all(dir)?;
            let mut handle = File::create(path)?;
            io::copy(&mut (&mut self.file).take(header.size), &mut handle)?;
        }
        Ok(())
    }

    /// Extracts all the files inside the archive to the current directory.
    pub fn extract(&mut self) -> Result<()> {
        self.extract_to(".")
    }

    /// Returns number of files in the current archive.
    pub fn files_count(&self) -> usize {
        self.headers.len()
    }

    /// Returns a borrowed vector of headers or metadata about the files in the archive.
    pub fn headers(&self) -> &Vec<Header> {
        &self.headers
    }

    /// Returns a copied vector of headers or metadata about the files in the archive.
    pub fn headers_owned(&self) -> Vec<Header> {
        self.headers.clone()
    }

    /// Extract a single file or path to a destination directory preserving the directory hierarchy.
    pub fn extract_file<P: AsRef<Path>>(&mut self, filename: P, destination: P) -> Result<()> {
        let mut offset = 0;
        let q = filename.as_ref();
        let destination = destination.as_ref();
        for header in self.headers.iter() {
            offset += HEADER_SIZE as u64;
            let original_path = [&header.prefix, &header.name].iter().collect::<PathBuf>();
            let clean = trim_clean(&original_path)?;
            if Path::new(&header.name) == q || clean == q || original_path == q {
                let path = destination.join(clean);
                let dir = path.parent().unwrap_or(destination);
                create_dir_all(dir)?;
                let mut handle = File::create(path)?;
                self.file.seek(SeekFrom::Start(offset))?;
                io::copy(&mut (&mut self.file).take(header.size), &mut handle)?;
                break;
            }

            offset += header.size;
        }
        Ok(())
    }
}
