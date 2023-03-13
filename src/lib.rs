mod common;
mod reader;
mod writer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_testdata() -> Result<(), Box<dyn std::error::Error>> {
        let mut r = reader::Reader::new("testdata/test_archive.wpress")?;
        r.extract()?;
        Ok(())
    }
}
