mod common;
mod reader;
mod writer;

pub use crate::common::{BlockParseError, FileParseError, Header, LengthExceededError};
pub use crate::reader::Reader;
pub use crate::writer::Writer;

#[cfg(test)]
mod tests {
    use std::{error::Error, fs::remove_file};

    use super::*;

    #[test]
    fn create_archive() -> Result<(), Box<dyn Error>> {
        let mut w = Writer::new("tests/writer_output.wpress")?;
        w.add("tests/writer")?;
        w.write()?;
        remove_file("tests/writer_output.wpress")?;
        Ok(())
    }
}
