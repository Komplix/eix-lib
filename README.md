# eix
[<img alt="github" src="https://img.shields.io/badge/github-Komplix%2Feix--lib-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/Komplix/eix-lib)
[<img alt="crates.io" src="https://img.shields.io/crates/v/eix.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/eix)
[![Build](https://github.com/Komplix/eix-lib/actions/workflows/build.yml/badge.svg)](https://github.com/Komplix/eix-lib/actions/workflows/build.yml)
![maintenance-status](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

A Rust library to parse and search the `Gentoo Linux` Portage Package Manager `eix` database format.

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
Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.



