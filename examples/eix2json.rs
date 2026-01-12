use eix::{Database, PackageReader, DB_VERSION_CURRENT};
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <eix-file> [output-json]", args[0]);
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
    let mut packages = Vec::new();

    while let Ok(true) = reader.next_category() {
        while let Ok(Some(pkg)) = reader.read_package() {
            packages.push(pkg);
        }
    }

    if args.len() > 2 {
        let output_path = &args[2];
        let file = match File::create(output_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error creating {}: {}", output_path, e);
                process::exit(1);
            }
        };
        let writer = BufWriter::new(file);
        if let Err(e) = serde_json::to_writer_pretty(writer, &packages) {
            eprintln!("Error writing JSON: {}", e);
            process::exit(1);
        }
    } else {
        if let Err(e) = serde_json::to_writer_pretty(std::io::stdout(), &packages) {
            eprintln!("Error writing JSON: {}", e);
            process::exit(1);
        }
    }
}
