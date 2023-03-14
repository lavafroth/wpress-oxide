use crate::common::{Header, ParseError, EOF_BLOCK, HEADER_SIZE};
use clean_path::Clean;
use std::error::Error;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::{fs, io};
pub struct Reader {
    file: std::fs::File,
    headers: Vec<Header>,
}

impl Reader {
    /// Creates a new `Reader` with the path supplied as the source file.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Reader, Box<dyn Error>> {
        let mut file = std::fs::File::open(path)?;
        let mut headers = Vec::new();
        loop {
            let mut buf = vec![0; HEADER_SIZE];
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

    /// Extracts all the files inside the wpress archive to the provided destination directory.
    pub fn extract_to<P: AsRef<Path>>(&mut self, destination: P) -> Result<(), Box<dyn Error>> {
        let destination = destination.as_ref();
        self.file.rewind()?;
        for header in self.headers.iter() {
            self.file.seek(io::SeekFrom::Current(HEADER_SIZE as i64))?;
            let mut clean_path = [&header.prefix, &header.name]
                .iter()
                .collect::<PathBuf>()
                .clean();
            if clean_path.starts_with("/") {
                clean_path = clean_path.strip_prefix("/")?.to_path_buf()
            }
            let path = Path::new(destination).join(clean_path);
            let dir = path.parent().unwrap_or(Path::new(destination));
            fs::create_dir_all(dir)?;
            let mut handle = fs::File::create(path)?;
            io::copy(&mut (&mut self.file).take(header.size), &mut handle)?;
        }
        Ok(())
    }

    /// Extracts all the files inside the wpress archive to the current directory.
    pub fn extract(&mut self) -> Result<(), Box<dyn Error>> {
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

    /// Extract a single file or path to a destination directory while preserving the directory
    /// hierarchy.
    pub fn extract_file<P: AsRef<Path>>(
        &mut self,
        filename: P,
        destination: P,
    ) -> Result<(), Box<dyn Error>> {
        let mut offset = 0;
        let filename = filename.as_ref();
        let destination = destination.as_ref();
        for header in self.headers.iter() {
            offset += HEADER_SIZE as u64;
            let original_path = [&header.prefix, &header.name].iter().collect::<PathBuf>();
            let mut cleaned = original_path.clean();
            if cleaned.starts_with("/") {
                cleaned = cleaned.strip_prefix("/")?.to_path_buf()
            }

            if Path::new(&header.name) == filename
                || cleaned == filename
                || original_path == filename
            {
                let path = destination.join(cleaned);
                let dir = path.parent().unwrap_or(destination);
                fs::create_dir_all(dir)?;
                let mut handle = fs::File::create(path)?;
                self.file.seek(SeekFrom::Start(offset))?;
                io::copy(&mut (&mut self.file).take(header.size), &mut handle)?;
                break;
            }

            offset += header.size;
        }
        Ok(())
    }
}
