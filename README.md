### wpress-oxide

![build](https://github.com/lavafroth/wpress-oxide/actions/workflows/rust.yml/badge.svg)

A rust library to interact with the wpress file format.

#### Quick start

To get started, add this library to your project.

```
cargo add --git https://github.com/lavafroth/wpress-oxide
```

#### Extracting a wpress archive

```rust
use wpress_oxide::Reader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let reader = Reader::new("the_archive_name.wpress")?;
  reader.extract()?;
  Ok(())
}
```
