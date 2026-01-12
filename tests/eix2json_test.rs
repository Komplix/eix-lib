use eix::{Database, PackageReader, DB_VERSION_CURRENT, Package};
use std::fs::File;
use std::io::BufReader;

#[test]
fn test_eix2json_consistency() {
    let eix_path = "testdata/portage.eix";
    let json_path = "testdata/portage.json";

    // 1. Read EIX
    let mut db = Database::open_read(eix_path).expect("Failed to open eix file");
    let header = db.read_header(DB_VERSION_CURRENT).expect("Failed to read header");
    let mut reader = PackageReader::new(db, header);
    let mut packages = Vec::new();

    while let Ok(true) = reader.next_category() {
        while let Ok(Some(pkg)) = reader.read_package() {
            packages.push(pkg);
        }
    }

    // 2. Read reference JSON
    let json_file = File::open(json_path).expect("Failed to open reference json file");
    let reader = BufReader::new(json_file);
    let reference_packages: Vec<Package> = serde_json::from_reader(reader).expect("Failed to parse reference json");

    // 3. Compare
    assert_eq!(packages.len(), reference_packages.len(), "Number of packages differs");
    
    for (i, (pkg, ref_pkg)) in packages.iter().zip(reference_packages.iter()).enumerate() {
        assert_eq!(pkg.name, ref_pkg.name, "Package name mismatch at index {}", i);
        assert_eq!(pkg.category, ref_pkg.category, "Package category mismatch for {}", pkg.name);
        assert_eq!(pkg.versions.len(), ref_pkg.versions.len(), "Version count mismatch for {}", pkg.name);
        
        for (j, (v, ref_v)) in pkg.versions.iter().zip(ref_pkg.versions.iter()).enumerate() {
            assert_eq!(v.version_string, ref_v.version_string, "Version string mismatch for {} version index {}", pkg.name, j);
            assert_eq!(v.eapi, ref_v.eapi, "EAPI mismatch for {} version {}", pkg.name, v.version_string);
            assert_eq!(v.slot, ref_v.slot, "Slot mismatch for {} version {}", pkg.name, v.version_string);
            assert_eq!(v.reponame, ref_v.reponame, "Reponame mismatch for {} version {}", pkg.name, v.version_string);
        }
    }
}
