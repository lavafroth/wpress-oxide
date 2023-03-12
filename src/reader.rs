use std::error::Error;
use std::path::Path;
pub struct Reader {
    file: std::fs::File,
    count: usize,
}

impl Reader {
    fn new<P: AsRef<Path>>(path: P) -> Result<Reader, Box<dyn Error>> {
        let file = std::fs::File::open(path)?;
        Ok(Reader { file, count: 0 })
    }

    fn extract(&self) -> Result<i64, Box<dyn Error>> {
        /* TODO */
        Ok(0)
    }
}
