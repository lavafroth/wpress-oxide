![](assets/wpress-oxide.png)

# wpress-oxide ![build](https://github.com/lavafroth/wpress-oxide/actions/workflows/rust.yml/badge.svg) [![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/lavafroth/wpress-oxide)](https://rust-reportcard.xuri.me/report/github.com/lavafroth/wpress-oxide)

A rust library to interact with the wpress archive format.

#### Quick start

To get started, add this library to your project.

```
cargo add --git https://github.com/lavafroth/wpress-oxide
```

#### Extracting a wpress archive

```rust
use wpress_oxide::Reader;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  let reader = Reader::new("the_archive_name.wpress")?;
  reader.extract()?;
  Ok(())
}
```
