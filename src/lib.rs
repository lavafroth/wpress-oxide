mod common;
mod reader;
mod writer;

pub use crate::common::{Header, ParseError};
pub use crate::reader::Reader;
pub use crate::writer::Writer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_testdata() -> Result<(), Box<dyn std::error::Error>> {
        let mut r = Reader::new("test/test_archive.wpress")?;
        r.extract()?;
        std::fs::remove_dir_all("repos")?;
        Ok(())
    }

    #[test]
    fn extract_single_filename() -> Result<(), Box<dyn std::error::Error>> {
        let mut r = Reader::new("test/test_archive.wpress")?;
        r.extract_file("lipsum.txt", "test_single_filename")?;
        std::fs::remove_dir_all("test_single_filename")?;
        Ok(())
    }

    #[test]
    fn extract_single_path() -> Result<(), Box<dyn std::error::Error>> {
        let mut r = Reader::new("test/test_archive.wpress")?;
        r.extract_file("repos/wpress/testdata/lipsum.txt", "test_single_path")?;
        std::fs::remove_dir_all("test_single_path")?;
        Ok(())
    }
}
