//! EIX Portage Database Format - Rust Parser
//!
//! The portage.eix file is a binary database for fast access
//! to Gentoo Portage ebuild information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

/* Basic types */
pub type UChar = u8;
pub type UNumber = u32;
pub type Catsize = u32;
pub type Treesize = u32;
pub type OffsetType = i64;

/* Mask Flags constants */
pub const MASK_NONE: u8 = 0x00;
pub const MASK_PACKAGE: u8 = 0x01;
pub const MASK_PROFILE: u8 = 0x02;
pub const MASK_HARD: u8 = MASK_PACKAGE | MASK_PROFILE;
pub const MASK_SYSTEM: u8 = 0x04;
pub const MASK_WORLD: u8 = 0x08;
pub const MASK_WORLD_SETS: u8 = 0x10;
pub const MASK_IN_PROFILE: u8 = 0x20;
pub const MASK_MARKED: u8 = 0x40;

/* Magic Number and Version */
pub const MAGICNUMCHAR: u8 = 0xFF;

// The file starts with "eix" followed by a newline (0x0A)
pub const DB_MAGIC: &[u8] = b"eix\n";

// Current database version
pub const DB_VERSION_CURRENT: DBVersion = 39;

/*
 * DBHeader - The main structure for the database header
 *
 *
 * Offset 0x00: Magic "eix\n" (4 bytes)
 * Offset 0x04: Version as byte (e.g. 0x27 = 39)
 * Offset 0x05: Number of categories as compressed number
 * Then: Number of overlays as compressed number
 * Then: Overlay data (path, label for each overlay)
 * Then: String hashes (EAPI, License, Keywords, IUSE, Slot, Depend)
 * Then: Feature flags (bitmask)
 * Then: World sets
*/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBHeader {
    // Current database version
    pub version: DBVersion,

    // Number of categories
    pub size: Catsize,

    // Overlays (repository directories)
    pub overlays: Vec<OverlayIdent>,

    // String hash tables for compression
    #[serde(skip)]
    pub eapi_hash: StringHash,
    #[serde(skip)]
    pub license_hash: StringHash,
    #[serde(skip)]
    pub keywords_hash: StringHash,
    #[serde(skip)]
    pub iuse_hash: StringHash,
    #[serde(skip)]
    pub slot_hash: StringHash,
    #[serde(skip)]
    pub depend_hash: StringHash,

    // Feature flags (SAVE_BITMASK)
    pub use_depend: bool,       // SAVE_BITMASK_DEP
    pub use_required_use: bool, // SAVE_BITMASK_REQUIRED_USE
    pub use_src_uri: bool,      // SAVE_BITMASK_SRC_URI

    // World sets
    pub world_sets: Vec<String>,
}

pub type DBVersion = u32;

/*
 * OverlayIdent - Identification of an overlay/repository
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayIdent {
    pub path: String,  // Path to the overlay (e.g. "/usr/portage")
    pub label: String, // Label of the overlay (e.g. "gentoo")
    pub priority: i32, // Priority of the overlay
}

/*
 * StringHash - Hash table for string compression
 */
#[derive(Debug, Clone, Default)]
pub struct StringHash {
    index_to_string: Vec<String>,
    string_to_index: HashMap<String, usize>,
}

impl StringHash {
    pub fn new() -> Self {
        StringHash::default()
    }

    pub fn get_index(&self, s: &str) -> Option<usize> {
        self.string_to_index.get(s).copied()
    }

    pub fn get_string(&self, index: usize) -> Option<&str> {
        self.index_to_string.get(index).map(|s| s.as_str())
    }

    pub fn add(&mut self, s: String) -> usize {
        if let Some(&idx) = self.string_to_index.get(&s) {
            return idx;
        }
        let idx = self.index_to_string.len();
        self.string_to_index.insert(s.clone(), idx);
        self.index_to_string.push(s);
        idx
    }

    pub fn len(&self) -> usize {
        self.index_to_string.len()
    }
}

/*
 * Bitmask for saved features
 */
pub type SaveBitmask = UNumber;

pub const SAVE_BITMASK_DEP: SaveBitmask = 0x01;
pub const SAVE_BITMASK_REQUIRED_USE: SaveBitmask = 0x02;
pub const SAVE_BITMASK_SRC_URI: SaveBitmask = 0x04;

/*
 * BasicPart - A part of a version string
 */
#[derive(Debug, Clone, Serialize)]
pub struct BasicPart {
    pub part_type: PartType,
    pub part_content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PartType {
    Garbage = 0,
    Alpha = 1,
    Beta = 2,
    Pre = 3,
    Rc = 4,
    Revision = 5,
    InterRev = 6,
    Patch = 7,
    Character = 8,
    Primary = 9,
    First = 10,
}

impl PartType {
    pub fn from_u64(v: u64) -> Self {
        match v {
            1 => PartType::Alpha,
            2 => PartType::Beta,
            3 => PartType::Pre,
            4 => PartType::Rc,
            5 => PartType::Revision,
            6 => PartType::InterRev,
            7 => PartType::Patch,
            8 => PartType::Character,
            9 => PartType::Primary,
            10 => PartType::First,
            _ => PartType::Garbage,
        }
    }
}

/*
 * Package - Representation of a package
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub category: String,
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub licenses: String,
    pub versions: Vec<Version>,
}

/*
 * Version - A specific version of a package
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    #[serde(rename = "version")]
    pub version_string: String,
    #[serde(skip)]
    pub parts: Vec<BasicPart>,
    pub eapi: String,
    pub mask_flags: u8,
    pub properties_flags: u8,
    pub restrict_flags: u64,
    pub keywords: Vec<String>,
    pub slot: String,
    pub overlay_key: u64,
    pub reponame: String,
    pub priority: i32,
    pub iuse: Vec<String>,
    pub required_use: Vec<String>,
    pub depend: Option<Depend>,
    pub src_uri: Option<String>,
}

impl Version {
    pub fn is_installed(&self) -> bool {
        (self.mask_flags & MASK_IN_PROFILE) != 0 || (self.mask_flags & MASK_MARKED) != 0
    }

    pub fn get_full_version(&self) -> String {
        let mut s = String::new();
        for part in &self.parts {
            match part.part_type {
                PartType::First | PartType::Character | PartType::Garbage => {
                    s.push_str(&part.part_content);
                }
                PartType::Alpha => {
                    s.push_str("_alpha");
                    s.push_str(&part.part_content);
                }
                PartType::Beta => {
                    s.push_str("_beta");
                    s.push_str(&part.part_content);
                }
                PartType::Pre => {
                    s.push_str("_pre");
                    s.push_str(&part.part_content);
                }
                PartType::Rc => {
                    s.push_str("_rc");
                    s.push_str(&part.part_content);
                }
                PartType::Patch => {
                    s.push_str("_p");
                    s.push_str(&part.part_content);
                }
                PartType::Revision => {
                    s.push_str("-r");
                    s.push_str(&part.part_content);
                }
                PartType::InterRev | PartType::Primary => {
                    s.push('.');
                    s.push_str(&part.part_content);
                }
            }
        }
        s
    }
}

/*
 * Depend - Dependencies of a package
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Depend {
    pub depend: Vec<String>,
    pub rdepend: Vec<String>,
    pub pdepend: Vec<String>,
    pub bdepend: Vec<String>,
    pub idepend: Vec<String>,
}

/*
 * Database - The main I/O class
 */
pub struct Database {
    reader: BufReader<File>,
}

impl Database {
    /// Opens a database for reading
    pub fn open_read<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Database { reader })
    }

    /// Reads a single byte
    pub fn read_uchar(&mut self) -> io::Result<UChar> {
        let mut buf = [0u8; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    /// Reads a number in eix format (variable length)
    ///
    /// Format:
    /// - Values 0-254: directly as one byte
    /// - Value 255 (0xFF = MAGICNUMCHAR): escape for multi-byte
    /// - After 0xFF: further 0xFF = more bytes follow
    /// - After 0xFF: 0x00 = means the value is 255 itself
    /// - After 0xFF: other value = start of the multi-byte number
    pub fn read_num(&mut self) -> io::Result<u64> {
        let ch = self.read_uchar()?;

        // Most common case: number < 255
        if ch != MAGICNUMCHAR {
            return Ok(ch as u64);
        }

        // Multi-byte case
        let mut to_get = 1usize;
        let mut result: u64;

        // Count further MAGICNUMCHAR
        loop {
            let c = self.read_uchar()?;

            if c == MAGICNUMCHAR {
                to_get += 1;
                continue;
            }

            if c != 0 {
                result = c as u64;
            } else {
                // Leading 0 after MAGICNUMCHAR means MAGICNUMCHAR itself
                result = MAGICNUMCHAR as u64;
                to_get -= 1;
            }
            break;
        }

        // Read remaining bytes
        for _ in 0..to_get {
            let byte = self.read_uchar()?;
            result = (result << 8) | (byte as u64);
        }

        Ok(result)
    }

    /// Reads a string (length + data)
    /// Format: <length> <data bytes>
    /// where length is encoded in eix number format
    pub fn read_string(&mut self) -> io::Result<String> {
        let len = self.read_num()? as usize;
        if len == 0 {
            return Ok(String::new());
        }

        let mut buf = vec![0u8; len];
        self.reader.read_exact(&mut buf)?;

        String::from_utf8(buf).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid UTF-8 in string: {}", e),
            )
        })
    }

    /// Reads a string from a hash (index → string)
    pub fn read_hash_string(&mut self, hash: &StringHash) -> io::Result<String> {
        let index = self.read_num()? as usize;
        hash.get_string(index)
            .map(|s| s.to_string())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid hash index: {} (hash size: {})", index, hash.len()),
                )
            })
    }

    /// Reads a string hash (list of strings)
    /// Format <number> <1st string>  ... <nth string>
    /// <number> is the number of strings in the hash
    /// where <number> is encoded in eix number format
    fn read_hash(&mut self) -> io::Result<StringHash> {
        let count = self.read_num()? as usize;
        let mut hash = StringHash::new();

        for _ in 0..count {
            let s = self.read_string()?;
            hash.add(s);
        }

        Ok(hash)
    }

    /// Reads a list of strings from a hash (WordVec)
    pub fn read_hash_words(&mut self, hash: &StringHash) -> io::Result<Vec<String>> {
        let count = self.read_num()? as usize;
        let mut words = Vec::with_capacity(count);
        for _ in 0..count {
            words.push(self.read_hash_string(hash)?);
        }
        Ok(words)
    }

    /// Reads a single part of a version
    pub fn read_part(&mut self) -> io::Result<BasicPart> {
        let val = self.read_num()?;
        let part_type = PartType::from_u64(val % 32);
        let len = (val / 32) as usize;
        let mut part_content = String::new();
        if len > 0 {
            let mut buf = vec![0u8; len];
            self.reader.read_exact(&mut buf)?;
            part_content = String::from_utf8(buf).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid UTF-8 in Part: {}", e),
                )
            })?;
        }
        Ok(BasicPart {
            part_type,
            part_content,
        })
    }

    /// Reads the database header
    pub fn read_header(&mut self, min_version: DBVersion) -> io::Result<DBHeader> {
        // 1. Read magic string (4 bytes)
        let mut magic = vec![0u8; DB_MAGIC.len()];
        self.reader.read_exact(&mut magic)?;
        if magic != DB_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid magic: expected {:?}, got {:?}", DB_MAGIC, magic),
            ));
        }

        // 2. Read version (eix compressed number)
        let version = self.read_num()? as DBVersion;
        if version < min_version {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Database version {} too old (minimum: {})",
                    version, min_version
                ),
            ));
        }

        // 3. Read number of categories (eix compressed number)
        let size = self.read_num()? as Catsize;

        // 4. Read number of overlays (compressed number)
        let overlay_count = self.read_num()? as usize;

        // 5. Read overlays
        let mut overlays = Vec::with_capacity(overlay_count);
        for i in 0..overlay_count {
            let path = self.read_string()?;
            let label = self.read_string()?;
            overlays.push(OverlayIdent {
                path,
                label,
                priority: i as i32,
            });
        }

        // 6-10. Read string hashes
        let eapi_hash = self.read_hash()?;
        let license_hash = self.read_hash()?;
        let keywords_hash = self.read_hash()?;
        let iuse_hash = self.read_hash()?;
        let slot_hash = self.read_hash()?;

        // 11. Read world sets (IMPORTANT: before the bitmask!)
        let world_set_count = self.read_num()? as usize;
        let mut world_sets = Vec::with_capacity(world_set_count);
        for _ in 0..world_set_count {
            world_sets.push(self.read_string()?);
        }

        // 12. Read feature flags
        let bitmask = self.read_num()? as SaveBitmask;
        let use_depend = (bitmask & SAVE_BITMASK_DEP) != 0;
        let use_required_use = (bitmask & SAVE_BITMASK_REQUIRED_USE) != 0;
        let use_src_uri = (bitmask & SAVE_BITMASK_SRC_URI) != 0;

        // 13. Read depend hash (only if enabled in bitmask)
        let depend_hash = if use_depend {
            // eix writes a length (offset) before the hash here
            let _len = self.read_num()?;
            self.read_hash()?
        } else {
            StringHash::new()
        };

        Ok(DBHeader {
            version,
            size,
            overlays,
            eapi_hash,
            license_hash,
            keywords_hash,
            iuse_hash,
            slot_hash,
            depend_hash,
            use_depend,
            use_required_use,
            use_src_uri,
            world_sets,
        })
    }
}

/*
 * PackageReader - Iterator over packages in the database
 */
pub struct PackageReader {
    db: Database,
    header: DBHeader,
    frames: Treesize,
    cat_size: Treesize,
    cat_name: String,
}

impl Database {
    pub fn read_version(&mut self, hdr: &DBHeader) -> io::Result<Version> {
        let mut eapi = String::new();
        if hdr.version >= 36 {
            eapi = self.read_hash_string(&hdr.eapi_hash)?;
        }

        let mask_flags = self.read_uchar()?;
        let properties_flags = self.read_uchar()?;
        let restrict_flags = self.read_num()?;

        // HashedWords  Full keywords string of the ebuild
        let keywords = self.read_hash_words(&hdr.keywords_hash)?;

        // Vector       VersionPart_\s
        let part_count = self.read_num()? as usize;
        let mut parts = Vec::with_capacity(part_count);
        for _ in 0..part_count {
            parts.push(self.read_part()?);
        }

        // HashedString Slot name. The slot name "0" is stored as ""
        let slot = self.read_hash_string(&hdr.slot_hash)?;

        // Number       Index of the portage overlay (in the overlays block)
        let overlay_key = self.read_num()?;

        let overlay = hdr.overlays.get(overlay_key as usize).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid overlay key: {}", overlay_key),
            )
        })?;
        let reponame = overlay.label.clone();
        let priority = overlay.priority;

        // HashedWords  Useflags of this version
        let iuse = self.read_hash_words(&hdr.iuse_hash)?;

        // The following occurs only if REQUIRED_USE is stored

        // HashedWords  REQUIRED_USE of this version
        let mut required_use = Vec::new();
        if hdr.use_required_use {
            required_use = self.read_hash_words(&hdr.iuse_hash)?;
        }

        // The following occurs only if dependencies are stored

        let mut depend = None;
        if hdr.use_depend {
            // Number       Length of the next four entries in bytes
            let _len = self.read_num()?; // Offset
            let mut dep = Depend {
                depend: self.read_hash_words(&hdr.depend_hash)?,
                rdepend: self.read_hash_words(&hdr.depend_hash)?,
                pdepend: self.read_hash_words(&hdr.depend_hash)?,
                bdepend: Vec::new(),
                idepend: Vec::new(),
            };
            if hdr.version > 31 {
                dep.bdepend = self.read_hash_words(&hdr.depend_hash)?;
            }
            if hdr.version > 38 {
                dep.idepend = self.read_hash_words(&hdr.depend_hash)?;
            }
            depend = Some(dep);
        }

        // The following occurs only if SRC_URI is stored

        // String       SRC_URI
        let mut src_uri = None;
        if hdr.use_src_uri {
            src_uri = Some(self.read_string()?);
        }

        // finished reading version

        Ok(Version {
            version_string: String::new(),
            parts,
            eapi,
            mask_flags,
            properties_flags,
            restrict_flags,
            keywords,
            slot,
            overlay_key,
            reponame,
            priority,
            iuse,
            required_use,
            depend,
            src_uri,
        })
    }
}

impl PackageReader {
    pub fn new(db: Database, header: DBHeader) -> Self {
        let frames = header.size;
        PackageReader {
            db,
            header,
            frames,
            cat_size: 0,
            cat_name: String::new(),
        }
    }

    /// Moves to the next category
    pub fn next_category(&mut self) -> io::Result<bool> {
        if self.frames == 0 {
            return Ok(false);
        }

        self.cat_name = self.db.read_string()?;
        self.cat_size = self.db.read_num()? as Treesize;
        self.frames -= 1;

        Ok(true)
    }

    pub fn current_category(&self) -> &str {
        &self.cat_name
    }

    /// Reads the next package in the current category
    pub fn read_package(&mut self) -> io::Result<Option<Package>> {
        if self.cat_size == 0 {
            return Ok(None);
        }

        // eix writes a length (offset) before each package
        let _pkg_len = self.db.read_num()?;

        let name = self.db.read_string()?;
        let description = self.db.read_string()?;
        let homepage = self.db.read_string()?;
        let licenses = self.db.read_hash_string(&self.header.license_hash)?;

        let version_count = self.db.read_num()? as usize;
        let mut versions = Vec::with_capacity(version_count);
        for _ in 0..version_count {
            let mut v = self.db.read_version(&self.header)?;
            v.version_string = v.get_full_version();
            versions.push(v);
        }

        self.cat_size -= 1;

        Ok(Some(Package {
            name,
            description,
            homepage,
            licenses,
            versions,
            category: self.cat_name.clone(),
        }))
    }
}

// For tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_bytes() {
        assert_eq!(DB_MAGIC, b"eix\n");
        assert_eq!(DB_MAGIC.len(), 4);
    }

    #[test]
    fn test_version() {
        assert_eq!(DB_VERSION_CURRENT, 39);
    }

    #[test]
    fn test_string_hash() {
        let mut hash = StringHash::new();
        let idx1 = hash.add("test".to_string());
        let idx2 = hash.add("another".to_string());
        let idx3 = hash.add("test".to_string());

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx1, idx3);
        assert_eq!(hash.len(), 2);

        assert_eq!(hash.get_string(0), Some("test"));
        assert_eq!(hash.get_string(1), Some("another"));
        assert_eq!(hash.get_string(2), None);

        assert_eq!(hash.get_index("test"), Some(0));
        assert_eq!(hash.get_index("another"), Some(1));
        assert_eq!(hash.get_index("nonexistent"), None);
    }

    #[test]
    fn test_part_type_from_u64() {
        assert_eq!(PartType::from_u64(1), PartType::Alpha);
        assert_eq!(PartType::from_u64(5), PartType::Revision);
        assert_eq!(PartType::from_u64(10), PartType::First);
        assert_eq!(PartType::from_u64(0), PartType::Garbage);
        assert_eq!(PartType::from_u64(99), PartType::Garbage);
    }

    // Mock Database for testing read_num and other methods
    struct MockDatabase {
        data: Vec<u8>,
        pos: usize,
    }

    impl MockDatabase {
        fn new(data: Vec<u8>) -> Self {
            MockDatabase { data, pos: 0 }
        }

        fn read_uchar(&mut self) -> io::Result<u8> {
            if self.pos >= self.data.len() {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
            }
            let val = self.data[self.pos];
            self.pos += 1;
            Ok(val)
        }

        // Adapted read_num for tests without needing a real file
        fn read_num(&mut self) -> io::Result<u64> {
            let ch = self.read_uchar()?;
            if ch != MAGICNUMCHAR {
                return Ok(ch as u64);
            }
            let mut to_get = 1usize;
            let mut result: u64;
            loop {
                let c = self.read_uchar()?;
                if c == MAGICNUMCHAR {
                    to_get += 1;
                    continue;
                }
                if c != 0 {
                    result = c as u64;
                } else {
                    result = MAGICNUMCHAR as u64;
                    to_get -= 1;
                }
                break;
            }
            for _ in 0..to_get {
                let byte = self.read_uchar()?;
                result = (result << 8) | (byte as u64);
            }
            Ok(result)
        }
    }

    #[test]
    fn test_read_num() {
        let cases = vec![
            (0x00, vec![0x00]),
            (0xFE, vec![0xFE]),
            (0xFF, vec![0xFF, 0x00]),
            (0x0100, vec![0xFF, 0x01, 0x00]),
            (0x01FF, vec![0xFF, 0x01, 0xFF]),
            (0xFEFF, vec![0xFF, 0xFE, 0xFF]),
            (0xFF00, vec![0xFF, 0xFF, 0x00, 0x00]),
            (0xFF01, vec![0xFF, 0xFF, 0x00, 0x01]),
            (0x010000, vec![0xFF, 0xFF, 0x01, 0x00, 0x00]),
            (0xABCDEF, vec![0xFF, 0xFF, 0xAB, 0xCD, 0xEF]),
            (0xFFABCD, vec![0xFF, 0xFF, 0xFF, 0x00, 0xAB, 0xCD]),
            (0x01ABCDEF, vec![0xFF, 0xFF, 0xFF, 0x01, 0xAB, 0xCD, 0xEF]),
        ];

        for (expected, bytes) in cases {
            let mut db = MockDatabase::new(bytes.clone());

            let result = db.read_num().expect(&format!("Failed to read {:?}", bytes));
            assert_eq!(
                result, expected,
                "Case {:?} failed: expected 0x{:X}, got 0x{:X}",
                bytes, expected, result
            );
        }
    }

    #[test]
    fn test_version_full_string() {
        let v = Version {
            version_string: "1.2.3".to_string(),
            parts: vec![
                BasicPart {
                    part_type: PartType::First,
                    part_content: "1".to_string(),
                },
                BasicPart {
                    part_type: PartType::Primary,
                    part_content: "2".to_string(),
                },
                BasicPart {
                    part_type: PartType::Primary,
                    part_content: "3".to_string(),
                },
                BasicPart {
                    part_type: PartType::Alpha,
                    part_content: "1".to_string(),
                },
                BasicPart {
                    part_type: PartType::Revision,
                    part_content: "1".to_string(),
                },
            ],
            eapi: "8".to_string(),
            mask_flags: 0,
            properties_flags: 0,
            restrict_flags: 0,
            keywords: vec![],
            slot: "0".to_string(),
            overlay_key: 0,
            reponame: "gentoo".to_string(),
            priority: 0,
            iuse: vec![],
            required_use: vec![],
            depend: None,
            src_uri: None,
        };
        assert_eq!(v.get_full_version(), "1.2.3_alpha1-r1");
    }

    #[test]
    fn test_version_is_installed() {
        let mut v = Version {
            version_string: String::new(),
            parts: vec![],
            eapi: String::new(),
            mask_flags: 0,
            properties_flags: 0,
            restrict_flags: 0,
            keywords: vec![],
            slot: String::new(),
            overlay_key: 0,
            reponame: String::new(),
            priority: 0,
            iuse: vec![],
            required_use: vec![],
            depend: None,
            src_uri: None,
        };
        assert!(!v.is_installed());

        v.mask_flags = MASK_IN_PROFILE;
        assert!(v.is_installed());

        v.mask_flags = MASK_MARKED;
        assert!(v.is_installed());
    }
}
