mod common;
mod reader;
mod writer;

pub use crate::common::{Header, ParseError};
pub use crate::reader::Reader;
pub use crate::writer::Writer;

#[cfg(test)]
mod tests {
    use crate::common::Result;
    use std::fs::remove_file;

    use super::*;

    #[test]
    fn create_archive() -> Result<()> {
        let mut w = Writer::new("tests/writer_output.wpress")?;
        w.add("tests/writer")?;
        w.write()?;
        remove_file("tests/writer_output.wpress")?;
        Ok(())
    }
}
