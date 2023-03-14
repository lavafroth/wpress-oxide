mod common;
mod reader;
mod writer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_testdata() -> Result<(), Box<dyn std::error::Error>> {
        let mut r = reader::Reader::new("test/test_archive.wpress")?;
        r.extract()?;
        Ok(())
    }

    #[test]
    fn extract_single_filename() -> Result<(), Box<dyn std::error::Error>> {
        let mut r = reader::Reader::new("test/test_archive.wpress")?;
        r.extract_file("lipsum.txt", "lorem_ipsum_name")?;
        Ok(())
    }

    #[test]
    fn extract_single_path() -> Result<(), Box<dyn std::error::Error>> {
        let mut r = reader::Reader::new("test/test_archive.wpress")?;
        r.extract_file("repos/wpress/testdata/lipsum.txt", "lorem_ipsum_path")?;
        Ok(())
    }
}
