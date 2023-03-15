mod common;
mod reader;
mod writer;

pub use crate::common::{Header, ParseError};
pub use crate::reader::Reader;
pub use crate::writer::Writer;

#[cfg(test)]
mod tests {
    use crate::common::Result;
    use std::fs::{remove_dir_all, remove_file};

    use super::*;

    #[test]
    fn extract_testdata() -> Result<()> {
        let mut r = Reader::new("tests/reader/archive.wpress")?;
        r.extract_to("tests/reader_output_0")?;
        remove_dir_all("tests/reader_output_0")?;
        Ok(())
    }

    #[test]
    // extracts all files with the name file.txt
    fn extract_single_filename() -> Result<()> {
        let mut r = Reader::new("tests/reader/archive.wpress")?;
        r.extract_file("file.txt", "tests/reader_output_1")?;
        remove_dir_all("tests/reader_output_1")?;
        Ok(())
    }

    #[test]
    fn extract_single_path() -> Result<()> {
        let mut r = Reader::new("tests/reader/archive.wpress")?;
        r.extract_file(
            "tests/writer/directory/subdirectory/file.txt",
            "tests/reader_output_2",
        )?;
        remove_dir_all("tests/reader_output_2")?;
        Ok(())
    }

    #[test]
    fn create_archive() -> Result<()> {
        let mut w = Writer::new("tests/writer_output.wpress")?;
        w.add("tests/writer")?;
        w.write()?;
        remove_file("tests/writer_output.wpress")?;
        Ok(())
    }
}
