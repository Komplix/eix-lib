use eix::{Database, PackageReader, DB_VERSION_CURRENT};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <eix-file>", args[0]);
        process::exit(1);
    }

    let input_path = &args[1];
    
    let mut db = match Database::open_read(input_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error opening {}: {}", input_path, e);
            process::exit(1);
        }
    };

    let header = match db.read_header(DB_VERSION_CURRENT) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error reading header: {}", e);
            process::exit(1);
        }
    };

    let mut reader = PackageReader::new(db, header);
    println!("name version mask_flags properties_flags restrict_flags priority slot overlay repo");

    while let Ok(true) = reader.next_category() {
        while let Ok(Some(pkg)) = reader.read_package() {

            for v in pkg.versions {
                println!(
                    "{}/{} {} {} {} {} {} {} {} {}",
                    pkg.category,
                    pkg.name,
                    v.version_string,
                    v.mask_flags,
                    v.properties_flags,
                    v.restrict_flags,
                    v.priority,
                    v.slot,
                    v.overlay_key,
                    v.reponame
                );
            }
        }
    }
}
