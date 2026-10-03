//! Texture conversion for the Fox engine, which reads FTEX, and a model folder's textures as
//! one unit (`team_compiler/pipeline.md` "3. Per-model-folder parallel steps", step 6).

use std::collections::BTreeMap;

use anyhow::Context;

use super::{Entry, TaskFiles, take};
use crate::plan::ModelFolder;
use crate::plan::subset::{FolderModels, PlayerFile, TextureFormat, player_file};

/// The textures of `folder`, converted from their bytes in `files` into the folder's texture
/// home for team `team_id`, by stem: one entry per texture, however many of the folder's
/// models use it and however many ids its packages are emitted under.
pub(super) fn folder_textures(
    folder: &ModelFolder,
    team_id: u16,
    files: &mut TaskFiles,
) -> anyhow::Result<Vec<Entry>> {
    let models = FolderModels::of(&folder.path, &folder.files);
    let mut textures = BTreeMap::new();
    for file in &folder.files {
        if let Some(PlayerFile::Texture(stem, format)) = player_file(&folder.path, file, &models) {
            textures.insert(stem, to_ftex(format, file.path.name(), take(files, file))?);
        }
    }
    Ok(textures
        .into_iter()
        .map(|(stem, bytes)| (folder.textures.texture(team_id, &stem), bytes))
        .collect())
}

/// The texture file `name`, in `format`, as FTEX: an FTEX passes through as it is, a DDS is
/// converted.
pub(super) fn to_ftex(
    format: TextureFormat,
    name: &str,
    bytes: Vec<u8>,
) -> anyhow::Result<Vec<u8>> {
    match format {
        TextureFormat::Ftex => Ok(bytes),
        TextureFormat::Dds => ftex::dds_to_ftex(&bytes, ftex::ColorSpace::Normal)
            .with_context(|| format!("{name}: cannot convert to FTEX")),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn a_dds_is_converted_and_an_ftex_passes_through() {
        let dds = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/tracer/studio/egg Tracer/Kits/g1/kit.dds"),
        )
        .unwrap();
        let converted = to_ftex(TextureFormat::Dds, "kit.dds", dds.clone()).unwrap();
        assert_eq!(
            converted,
            ftex::dds_to_ftex(&dds, ftex::ColorSpace::Normal).unwrap()
        );
        assert_eq!(
            to_ftex(TextureFormat::Ftex, "kit.ftex", converted.clone()).unwrap(),
            converted
        );
        let error = to_ftex(TextureFormat::Dds, "kit.dds", b"not a DDS".to_vec()).unwrap_err();
        assert_eq!(format!("{error}"), "kit.dds: cannot convert to FTEX");
    }
}
