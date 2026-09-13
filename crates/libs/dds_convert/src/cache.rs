//! The session conversion cache: finished container bytes keyed by source
//! hash, source format and target.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{CachePolicy, ConvertError, SourceFormat, SourceHash, Target};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey {
    hash: SourceHash,
    format: SourceFormat,
    target: Target,
}

/// The session cache in front of `decode` + `convert`: finished container
/// bytes by (source hash, source format, target). `Bypass` never inserts
/// and never reads.
pub struct Converter {
    entries: Mutex<HashMap<CacheKey, Arc<[u8]>>>,
}

impl Converter {
    /// An empty cache.
    pub fn new() -> Self {
        Converter {
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Decodes `bytes` as `format`, converts for `target`, and caches the
    /// result under `hash` when `cache` is `Use`. `hash` is the caller's
    /// [`crate::source_hash`] of `bytes`, computed once at materialization.
    pub fn convert(
        &self,
        hash: SourceHash,
        bytes: &[u8],
        format: SourceFormat,
        target: Target,
        cache: CachePolicy,
    ) -> Result<Arc<[u8]>, ConvertError> {
        if cache == CachePolicy::Bypass {
            let decoded = crate::decode(bytes, format)?;
            return Ok(crate::convert(&decoded, target)?.into());
        }
        let key = CacheKey {
            hash,
            format,
            target,
        };
        if let Some(hit) = self.entries.lock().unwrap().get(&key).cloned() {
            log::debug!("conversion cache hit");
            return Ok(hit);
        }
        let decoded = crate::decode(bytes, format)?;
        let output: Arc<[u8]> = crate::convert(&decoded, target)?.into();
        log::debug!("conversion cache miss, storing {} bytes", output.len());
        self.entries.lock().unwrap().insert(key, output.clone());
        Ok(output)
    }

    /// Bytes currently retained, for the pipeline's memory budget.
    pub fn retained_bytes(&self) -> usize {
        self.entries
            .lock()
            .unwrap()
            .values()
            .map(|entry| entry.len())
            .sum()
    }

    /// Drops every retained entry.
    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}
