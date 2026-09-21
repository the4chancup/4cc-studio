//! The UniformParameter container that holds a game's kit configs.
//!
//! Little-endian: u32 entry count, u32 entry-table offset (8 as written),
//! then per entry 12 bytes (u32 content offset, u32 content length, u32 name
//! offset; offsets absolute), a pool of NUL-terminated UTF-8 names, then the
//! contents each padded to 16. The writer sorts by name and pads nothing
//! else. Input may be WESYS-wrapped; [`UniformParameter::read`] unwraps it
//! first.

use std::collections::BTreeMap;

/// A UniformParameter file: entry contents keyed by entry name, in name order.
#[derive(Debug, Default)]
pub struct UniformParameter {
    entries: BTreeMap<String, Vec<u8>>,
}

/// Why a byte buffer is not a readable UniformParameter.
#[derive(Debug, thiserror::Error)]
pub enum UniparamError {
    /// The buffer ends before a structure that extends past it.
    #[error("uniform parameter file is truncated")]
    Truncated,
    /// An entry's content range points outside the file.
    #[error("entry points outside the file: {name}")]
    OutOfBounds {
        /// The entry whose range is out of bounds.
        name: String,
    },
    /// Two entries share a name.
    #[error("duplicate entry: {0}")]
    DuplicateEntry(String),
    /// An entry name is not valid UTF-8.
    #[error("entry name is not utf-8")]
    Utf8,
    /// The WESYS wrapper could not be unwrapped.
    #[error("wesys unwrap failed: {0}")]
    Wesys(#[from] wezlib::Error),
}

impl UniformParameter {
    /// An empty container.
    pub fn new() -> Self {
        UniformParameter {
            entries: BTreeMap::new(),
        }
    }

    /// Parses a buffer, unwrapping WESYS first. Names are read as
    /// NUL-terminated strings; content ranges are bounds-checked.
    pub fn read(bytes: &[u8]) -> Result<Self, UniparamError> {
        let bytes = wezlib::decompress_if_wrapped(bytes)?;
        let data: &[u8] = &bytes;
        if data.len() < 8 {
            return Err(UniparamError::Truncated);
        }
        let entry_count = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
        let entry_offset = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
        let table = data
            .get(entry_offset..entry_offset.saturating_add(entry_count.saturating_mul(12)))
            .ok_or(UniparamError::Truncated)?;

        let mut entries = BTreeMap::new();
        for record in table.as_chunks::<12>().0 {
            let content_offset = u32::from_le_bytes(record[0..4].try_into().unwrap()) as usize;
            let content_length = u32::from_le_bytes(record[4..8].try_into().unwrap()) as usize;
            let name_offset = u32::from_le_bytes(record[8..12].try_into().unwrap()) as usize;

            let rest = data.get(name_offset..).ok_or(UniparamError::Truncated)?;
            let nul = rest
                .iter()
                .position(|b| *b == 0)
                .ok_or(UniparamError::Truncated)?;
            let name_bytes = &rest[..nul];
            let name = String::from_utf8(name_bytes.to_vec()).map_err(|_| UniparamError::Utf8)?;

            let content = data
                .get(content_offset..content_offset.saturating_add(content_length))
                .ok_or_else(|| UniparamError::OutOfBounds { name: name.clone() })?;
            if entries.insert(name.clone(), content.to_vec()).is_some() {
                return Err(UniparamError::DuplicateEntry(name));
            }
        }
        Ok(UniformParameter { entries })
    }

    /// Serializes the container: name order, name pool directly after the
    /// entry table, contents each padded to 16. Not WESYS-wrapped; wrapping
    /// is the caller's job.
    pub fn write(&self) -> Vec<u8> {
        let mut name_pool = Vec::new();
        let mut content_pool = Vec::new();
        let mut records = Vec::with_capacity(self.entries.len());
        for (name, content) in &self.entries {
            let name_offset = name_pool.len() as u32;
            name_pool.extend_from_slice(name.as_bytes());
            name_pool.push(0);

            let content_offset = content_pool.len() as u32;
            content_pool.extend_from_slice(content);
            let pad = (16 - content_pool.len() % 16) % 16;
            content_pool.resize(content_pool.len() + pad, 0);

            records.push((content_offset, content.len() as u32, name_offset));
        }

        let name_pool_offset = 8u32 + 12 * self.entries.len() as u32;
        let content_pool_offset = name_pool_offset + name_pool.len() as u32;

        let mut output = Vec::new();
        output.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        output.extend_from_slice(&8u32.to_le_bytes());
        for (content_offset, content_length, name_offset) in records {
            output.extend_from_slice(&(content_offset + content_pool_offset).to_le_bytes());
            output.extend_from_slice(&content_length.to_le_bytes());
            output.extend_from_slice(&(name_offset + name_pool_offset).to_le_bytes());
        }
        output.extend_from_slice(&name_pool);
        output.extend_from_slice(&content_pool);
        output
    }

    /// Number of stored entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the container stores no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entries in name order.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.entries
            .iter()
            .map(|(name, content)| (name.as_str(), content.as_slice()))
    }

    /// The content stored under `name`.
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.entries.get(name).map(Vec::as_slice)
    }

    /// Inserts or replaces `name`, returning the replaced content if any.
    pub fn insert(&mut self, name: String, content: Vec<u8>) -> Option<Vec<u8>> {
        self.entries.insert(name, content)
    }

    /// Removes `name`, returning its content if present.
    pub fn remove(&mut self, name: &str) -> Option<Vec<u8>> {
        self.entries.remove(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KONAMI: &[u8] = include_bytes!("../tests/fixtures/konami_pes21_UniformParameter.bin");
    const SAMPLE: &[u8] = include_bytes!("../tests/fixtures/pft_sample.bin");
    const KIT_CONFIG_100: &[u8] = include_bytes!(
        "../../kit_config/tests/fixtures/konami_pes21_model144_100_DEF_1st_realUni.bin"
    );

    fn sample_entries() -> [(String, Vec<u8>); 3] {
        [
            ("u0702p1.bin".to_owned(), (1u8..=29).collect()),
            ("u0701g1.bin".to_owned(), vec![0x11; 16]),
            ("u0701p1.bin".to_owned(), vec![0x22; 100]),
        ]
    }

    #[test]
    fn konami_fixture_reads_wrapped() {
        let up = UniformParameter::read(KONAMI).unwrap();
        assert_eq!(up.len(), 2174);
        let names: Vec<&str> = up.entries().map(|(name, _)| name).collect();
        assert_eq!(names.first(), Some(&"0_DEF_1st.bin"));
        assert_eq!(names.last(), Some(&"referee_EU_4.bin"));
        for (_, content) in up.entries() {
            assert!(content.len() == 96 || content.len() == 120);
        }
        assert_eq!(up.get("100_DEF_1st_realUni.bin").unwrap(), KIT_CONFIG_100);
    }

    #[test]
    fn sample_reads_and_rewrites_identically() {
        let up = UniformParameter::read(SAMPLE).unwrap();
        assert_eq!(up.len(), 3);
        for (name, expected) in &sample_entries() {
            assert_eq!(up.get(name).unwrap(), expected.as_slice());
        }
        // Insert in a different order: the writer sorts by name anyway.
        let mut rebuilt = UniformParameter::new();
        for (name, content) in [
            sample_entries()[2].clone(),
            sample_entries()[0].clone(),
            sample_entries()[1].clone(),
        ] {
            rebuilt.insert(name, content);
        }
        assert_eq!(rebuilt.write(), SAMPLE);
    }

    #[test]
    fn konami_round_trip_keeps_entries() {
        let up = UniformParameter::read(KONAMI).unwrap();
        let reread = UniformParameter::read(&up.write()).unwrap();
        let before: Vec<(&str, &[u8])> = up.entries().collect();
        let after: Vec<(&str, &[u8])> = reread.entries().collect();
        assert_eq!(before, after);
        // Konami's layout differs from the reference writer's (table order, a
        // 16-aligned content pool: `format_crates.md` "`uniparam`"), so the
        // standard here is entry equality; byte identity is tested against
        // the reference writer's sample above.
    }

    #[test]
    fn corrupt_inputs_error() {
        assert!(matches!(
            UniformParameter::read(&SAMPLE[..4]),
            Err(UniparamError::Truncated)
        ));

        // Name offset past the end (first record at offset 8).
        let mut bad_name = SAMPLE.to_vec();
        bad_name[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            UniformParameter::read(&bad_name),
            Err(UniparamError::Truncated)
        ));

        // Content range past the end.
        let mut bad_content = SAMPLE.to_vec();
        bad_content[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            UniformParameter::read(&bad_content),
            Err(UniparamError::OutOfBounds { .. })
        ));

        // Second record's name offset pointed at the first record's name.
        let mut dup = SAMPLE.to_vec();
        let name_offset: [u8; 4] = dup[16..20].try_into().unwrap();
        dup[28..32].copy_from_slice(&name_offset);
        assert!(matches!(
            UniformParameter::read(&dup),
            Err(UniparamError::DuplicateEntry(_))
        ));
    }

    #[test]
    fn empty_container_writes_the_eight_header_bytes() {
        let up = UniformParameter::new();
        assert!(up.is_empty());
        // Entry count 0, table offset 8: an empty container is exactly the header.
        let bytes = up.write();
        assert_eq!(bytes, [0, 0, 0, 0, 8, 0, 0, 0]);
        let reread = UniformParameter::read(&bytes).unwrap();
        assert!(reread.is_empty());
        assert_eq!(reread.len(), 0);
    }

    #[test]
    fn remove_returns_content_and_drops_the_entry() {
        let mut up = UniformParameter::new();
        up.insert("a.bin".to_owned(), vec![1, 2, 3]);
        up.insert("b.bin".to_owned(), vec![4]);
        assert!(!up.is_empty());

        assert_eq!(up.remove("a.bin"), Some(vec![1, 2, 3]));
        assert_eq!(up.get("a.bin"), None);
        assert_eq!(up.len(), 1);
        assert_eq!(up.remove("a.bin"), None);
        assert_eq!(up.remove("b.bin"), Some(vec![4]));
        assert!(up.is_empty());
    }
}
