//! Reading `.zip` and `.7z` exports in place: the entry list eagerly, contents on demand,
//! over any `Read + Seek` source.

use std::collections::HashMap;
use std::io::{Read, Seek};

use sevenz_rust2::Password;

/// One file entry in an archive: forward slashes, no leading `/`, files only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The normalized path.
    pub path: String,
    /// The uncompressed size in bytes.
    pub size: u64,
}

/// What reading an archive can fail with.
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    /// An I/O failure on the underlying reader (or `open`'s `File::open`).
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The zip crate's own message.
    #[error("zip: {0}")]
    Zip(String),
    /// The 7z crate's own message.
    #[error("7z: {0}")]
    SevenZ(String),
    /// `open` was given a path whose extension is neither `zip` nor `7z`.
    #[error("unsupported archive extension {0:?}")]
    UnsupportedExtension(String),
    /// `read` was given a path `entries()` does not list.
    #[error("no such entry {0:?}")]
    NotFound(String),
    /// An entry name that cannot name a tree file (empty, `.`/`..` segments, drive prefix).
    #[error("invalid entry name {0:?}")]
    InvalidName(String),
    /// The archive or entry needs a password; there is no prompting here.
    #[error("archive is encrypted")]
    Encrypted,
}

/// A raw entry name to a tree path: `\` to `/`, a leading `/` or `./` stripped, `//`
/// collapsed. Anything that cannot name a tree file is `InvalidName`.
fn normalize(raw: &str) -> Result<String, ArchiveError> {
    let mut name = raw.replace('\\', "/");
    while name.starts_with("./") || name.starts_with('/') {
        name = name
            .strip_prefix("./")
            .map_or_else(|| name[1..].to_string(), str::to_string);
    }
    let mut segments: Vec<&str> = Vec::new();
    for segment in name.split('/') {
        match segment {
            "" => {}
            "." | ".." => return Err(ArchiveError::InvalidName(raw.to_string())),
            _ => segments.push(segment),
        }
    }
    let first = segments.first().copied().unwrap_or("");
    let mut chars = first.chars();
    if let (Some(drive), Some(':'), None) = (chars.next(), chars.next(), chars.next())
        && drive.is_ascii_alphabetic()
    {
        return Err(ArchiveError::InvalidName(raw.to_string()));
    }
    if segments.is_empty() {
        return Err(ArchiveError::InvalidName(raw.to_string()));
    }
    Ok(segments.join("/"))
}

/// Whether the raw name names a directory: the crate's flag or a trailing slash.
fn is_dir_name(raw: &str) -> bool {
    raw.ends_with('/') || raw.ends_with('\\')
}

/// A 7z crate error to ours: password and encryption-codec failures are `Encrypted`.
fn seven_error(error: sevenz_rust2::Error) -> ArchiveError {
    match &error {
        // An encrypted header surfaces as the AES codec being missing rather than as a
        // password request.
        sevenz_rust2::Error::PasswordRequired | sevenz_rust2::Error::MaybeBadPassword(_) => {
            ArchiveError::Encrypted
        }
        sevenz_rust2::Error::UnsupportedCompressionMethod(method) if method == "AES256_SHA256" => {
            ArchiveError::Encrypted
        }
        _ => ArchiveError::SevenZ(error.to_string()),
    }
}

enum Inner<R: Read + Seek> {
    Zip {
        archive: zip::ZipArchive<R>,
        /// Normalized path → (zip index, is encrypted).
        index: HashMap<String, (usize, bool)>,
    },
    SevenZ {
        reader: Box<sevenz_rust2::ArchiveReader<R>>,
        /// The whole archive decompressed on first `read` (a solid block decompresses
        /// whole; per-entry extraction is quadratic).
        contents: Option<HashMap<String, Vec<u8>>>,
    },
}

/// An open archive: the entry list is eager, contents are read on demand.
pub struct Archive<R: Read + Seek> {
    entries: Vec<Entry>,
    inner: Inner<R>,
}

impl<R: Read + Seek> Archive<R> {
    /// A zip: reads the central directory only.
    pub fn zip(reader: R) -> Result<Self, ArchiveError> {
        let mut archive =
            zip::ZipArchive::new(reader).map_err(|error| ArchiveError::Zip(error.to_string()))?;
        let mut entries = Vec::new();
        let mut index = HashMap::new();
        for i in 0..archive.len() {
            // The raw view leaves the data stream alone: encrypted entries still list.
            let file = archive
                .by_index_raw(i)
                .map_err(|error| ArchiveError::Zip(error.to_string()))?;
            if file.is_dir() || is_dir_name(file.name()) {
                continue;
            }
            let path = normalize(file.name())?;
            entries.push(Entry {
                path: path.clone(),
                size: file.size(),
            });
            index.insert(path, (i, file.encrypted()));
        }
        Ok(Archive {
            entries,
            inner: Inner::Zip { archive, index },
        })
    }

    /// A 7z: reads the header only.
    pub fn seven_z(reader: R) -> Result<Self, ArchiveError> {
        let reader =
            sevenz_rust2::ArchiveReader::new(reader, Password::empty()).map_err(seven_error)?;
        let mut entries = Vec::new();
        for file in &reader.archive().files {
            if file.is_directory() || is_dir_name(file.name()) {
                continue;
            }
            entries.push(Entry {
                path: normalize(file.name())?,
                size: file.size(),
            });
        }
        Ok(Archive {
            entries,
            inner: Inner::SevenZ {
                reader: Box::new(reader),
                contents: None,
            },
        })
    }

    /// The file entries, in archive order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// One entry's bytes. `path` is compared as given — pass what `entries()` returned.
    /// On a 7z the first call decompresses the whole archive into memory and later reads
    /// are lookups.
    pub fn read(&mut self, path: &str) -> Result<Vec<u8>, ArchiveError> {
        match &mut self.inner {
            Inner::Zip { archive, index } => {
                let Some(&(i, encrypted)) = index.get(path) else {
                    return Err(ArchiveError::NotFound(path.to_string()));
                };
                if encrypted {
                    return Err(ArchiveError::Encrypted);
                }
                let mut file = archive
                    .by_index(i)
                    .map_err(|error| ArchiveError::Zip(error.to_string()))?;
                // No capacity hint: the header's size is untrusted input.
                let mut contents = Vec::new();
                file.read_to_end(&mut contents)?;
                Ok(contents)
            }
            Inner::SevenZ { reader, contents } => {
                if contents.is_none() {
                    let mut all: HashMap<String, Vec<u8>> = HashMap::new();
                    let result = reader.for_each_entries(|entry, data| {
                        if entry.is_directory() {
                            return Ok(true);
                        }
                        let mut bytes = Vec::new();
                        data.read_to_end(&mut bytes)?;
                        all.insert(entry.name().to_string(), bytes);
                        Ok(true)
                    });
                    result.map_err(seven_error)?;
                    let mut normalized = HashMap::with_capacity(all.len());
                    for (raw, bytes) in all {
                        normalized.insert(normalize(&raw)?, bytes);
                    }
                    *contents = Some(normalized);
                }
                let contents = contents.as_ref().expect("filled above");
                contents
                    .get(path)
                    .cloned()
                    .ok_or_else(|| ArchiveError::NotFound(path.to_string()))
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Archive<std::fs::File> {
    /// Opens an archive file by its extension, case-insensitive: `.zip`, `.7z`.
    pub fn open(path: &std::path::Path) -> Result<Self, ArchiveError> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_lowercase();
        match extension.as_str() {
            "zip" => Archive::zip(std::fs::File::open(path)?),
            "7z" => Archive::seven_z(std::fs::File::open(path)?),
            _ => Err(ArchiveError::UnsupportedExtension(extension)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const SAMPLE_7Z: &[u8] = include_bytes!("../tests/fixtures/sample.7z");
    const SAMPLE_7ZIP_ZIP: &[u8] = include_bytes!("../tests/fixtures/sample_7zip.zip");
    const SAMPLE_STORE_ZIP: &[u8] = include_bytes!("../tests/fixtures/sample_store.zip");
    const SAMPLE_PS_ZIP: &[u8] = include_bytes!("../tests/fixtures/sample_ps.zip");
    const ENCRYPTED_7Z: &[u8] = include_bytes!("../tests/fixtures/encrypted.7z");
    const ENCRYPTED_ZIP: &[u8] = include_bytes!("../tests/fixtures/encrypted.zip");

    /// The six expected entries, sorted by path.
    fn expected() -> Vec<Entry> {
        let mut list = [
            ("Sample Export/Faces/Player One/face.dds", 176),
            ("Sample Export/Faces/Player One/materials.mtl", 476),
            ("Sample Export/Faces/Player Two/empty.txt", 0),
            ("Sample Export/Faces/Player Two/face_diff.bin", 960),
            ("Sample Export/Kits/R\u{e9}f.txt", 30),
            ("Sample Export/Note.txt", 74),
        ]
        .into_iter()
        .map(|(path, size)| Entry {
            path: path.to_string(),
            size,
        })
        .collect::<Vec<_>>();
        list.sort_by(|a, b| a.path.cmp(&b.path));
        list
    }

    /// The tree's file bytes for an entry path under `Sample Export/`.
    fn sample_bytes(path: &str) -> Vec<u8> {
        let file = path.strip_prefix("Sample Export/").expect("top folder");
        let full = format!(
            "{}/tests/fixtures/sample/{file}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read(&full).expect("sample file")
    }

    #[test]
    fn every_readable_archive_lists_the_tree() {
        let seven_z = Archive::seven_z(Cursor::new(SAMPLE_7Z)).expect("7z");
        let zips = [
            Archive::zip(Cursor::new(SAMPLE_7ZIP_ZIP)).expect("7zip zip"),
            Archive::zip(Cursor::new(SAMPLE_STORE_ZIP)).expect("store zip"),
            Archive::zip(Cursor::new(SAMPLE_PS_ZIP)).expect("ps zip"),
        ];
        let mut sorted = seven_z.entries().to_vec();
        sorted.sort_by(|a, b| a.path.cmp(&b.path));
        assert_eq!(sorted, expected(), "7z");
        for archive in &zips {
            let mut sorted = archive.entries().to_vec();
            sorted.sort_by(|a, b| a.path.cmp(&b.path));
            assert_eq!(sorted, expected());
        }
    }

    #[test]
    fn every_readable_archive_yields_the_bytes() {
        for archive in [
            Archive::seven_z(Cursor::new(SAMPLE_7Z)).expect("7z"),
            Archive::zip(Cursor::new(SAMPLE_7ZIP_ZIP)).expect("7zip zip"),
            Archive::zip(Cursor::new(SAMPLE_STORE_ZIP)).expect("store zip"),
            Archive::zip(Cursor::new(SAMPLE_PS_ZIP)).expect("ps zip"),
        ]
        .iter_mut()
        {
            let paths: Vec<String> = archive
                .entries()
                .iter()
                .map(|entry| entry.path.clone())
                .collect();
            for path in &paths {
                assert_eq!(
                    archive.read(path).ok().as_deref(),
                    Some(sample_bytes(path).as_slice()),
                    "{path}"
                );
            }
            assert!(matches!(
                archive.read("Sample Export/Empty"),
                Err(ArchiveError::NotFound(_))
            ));
            assert!(matches!(
                archive.read("no/such.txt"),
                Err(ArchiveError::NotFound(_))
            ));
        }
    }

    #[test]
    fn encrypted_archives_are_refused() {
        match Archive::seven_z(Cursor::new(ENCRYPTED_7Z)) {
            Err(ArchiveError::Encrypted) => {}
            Ok(mut archive) => {
                let path = archive.entries()[0].path.clone();
                assert!(matches!(archive.read(&path), Err(ArchiveError::Encrypted)));
            }
            Err(error) => panic!("encrypted 7z: {error}"),
        }
        let mut zip = Archive::zip(Cursor::new(ENCRYPTED_ZIP)).expect("encrypted zip lists");
        assert_eq!(zip.entries().len(), 6);
        let path = zip.entries()[0].path.clone();
        assert!(matches!(zip.read(&path), Err(ArchiveError::Encrypted)));
    }

    #[test]
    fn names_are_normalized() {
        assert_eq!(normalize("a\\b/c.txt").ok().as_deref(), Some("a/b/c.txt"));
        assert!(matches!(
            normalize("./x/./y"),
            Err(ArchiveError::InvalidName(_))
        ));
        assert_eq!(normalize("/x/y").ok().as_deref(), Some("x/y"));
        assert!(matches!(
            normalize("x/../y"),
            Err(ArchiveError::InvalidName(_))
        ));
        assert!(matches!(
            normalize("C:/x"),
            Err(ArchiveError::InvalidName(_))
        ));
        assert!(matches!(normalize(""), Err(ArchiveError::InvalidName(_))));
        assert_eq!(normalize("a//b").ok().as_deref(), Some("a/b"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn open_dispatches_on_extension() {
        let dir = env!("CARGO_MANIFEST_DIR");
        let seven_z = Archive::open(std::path::Path::new(&format!(
            "{dir}/tests/fixtures/sample.7z"
        )))
        .expect("sample.7z");
        assert_eq!(seven_z.entries().len(), 6);
        let zip = Archive::open(std::path::Path::new(&format!(
            "{dir}/tests/fixtures/sample_ps.zip"
        )))
        .expect("sample_ps.zip");
        assert_eq!(zip.entries().len(), 6);
        assert!(matches!(
            Archive::open(std::path::Path::new(&format!("{dir}/tests/fixtures/README.md"))),
            Err(ArchiveError::UnsupportedExtension(ext)) if ext == "md"
        ));
    }

    #[test]
    fn garbage_is_an_error() {
        assert!(matches!(
            Archive::zip(Cursor::new(b"not an archive")),
            Err(ArchiveError::Zip(_))
        ));
        assert!(matches!(
            Archive::seven_z(Cursor::new(b"not an archive")),
            Err(ArchiveError::SevenZ(_))
        ));
        assert!(Archive::seven_z(Cursor::new(&SAMPLE_7Z[..40])).is_err());
    }
}
