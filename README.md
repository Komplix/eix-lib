# eix

A Rust library to parse the Gentoo `eix` database format.

## Usage

```rust
use eix::{Database, PackageReader};

fn main() -> std::io::Result<()> {
    let mut db = Database::open_read("/var/cache/eix/portage.eix")?;
    let header = db.read_header(0)?;
    let mut reader = PackageReader::new(db, header);

    while reader.next_category()? {
        let category = reader.current_category();
        while let Some(pkg) = reader.read_package()? {
            println!("{}/{}: {}", category, pkg.name, pkg.description);
        }
    }
    Ok(())
}
```

## License

MIT OR Apache 2.0



