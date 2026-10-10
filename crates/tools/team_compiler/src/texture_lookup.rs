//! Where a texture name resolves, nearest first (`aesthetics_export/player_folders.md`
//! "Subfolders"): a texture a model or a `.mtl` of a model folder names with no path resolves
//! in the file's own folder first, then in each parent up to the model folder, then among the
//! textures of the shared folders the model folder combines, which sit at the root of its
//! texture home; a lookup never goes down into a subfolder. The face task and the Models task
//! point texture paths with it, and the deep pass looks a `.mtl`'s paths up with it, so a path
//! the deep pass calls supplied is one the tasks point.

use std::collections::BTreeMap;

use vtree::ScopePath;

use crate::kit_variants::has_variant_among;

/// One folder's textures, each by its stem folded, with its path below its source folder
/// without its extension, as spelled (`PlayerFile::Texture`'s `below`): the path its converted
/// file has under the texture home (`paths::TextureHome::texture`).
pub(crate) type TexturePlace = BTreeMap<String, String>;

/// A model folder's textures by the folder directly holding each, for a texture name to
/// resolve nearest first (`places`).
#[derive(Debug, Default)]
pub(crate) struct TextureFolders {
    /// The model folder's own textures, per folder of its tree, keyed by that folder's path
    /// below the model folder, folded: `""` for the model folder itself, `jessie/body` for
    /// its subfolder `jessie/body/`.
    own: BTreeMap<String, TexturePlace>,
    /// The textures of the shared folders the model folder combines, each directly in its
    /// folder: they sit at the root of the texture home, as the root's own do.
    combined: TexturePlace,
}

impl TextureFolders {
    /// Adds the texture at `below`, its path below its source folder (`PlayerFile::Texture`'s
    /// `below`): one of the model folder's own when `own` is set, else a combined shared
    /// folder's. The first texture of a stem in a place keeps it: validation drops a folder
    /// holding two of one stem (`texture_stem_conflict`), and copies of a stem across the
    /// sources share one path, which the textures task fills.
    pub(crate) fn insert(&mut self, own: bool, below: &str) {
        let (directory, name) = split(below);
        let place = if own {
            let key = vtree::fold_name(directory.trim_end_matches('/'));
            self.own.entry(key).or_default()
        } else {
            &mut self.combined
        };
        place
            .entry(vtree::fold_name(name))
            .or_insert_with(|| below.to_owned());
    }

    /// The places a texture name of the file at `path` resolves in, nearest first: when `path`
    /// is below `folder`, the model folder, the file's own folder, then each parent up to
    /// `folder`; then the combined shared folders'. A file of another folder (a combined shared
    /// folder's, a `Common/` model brought in by a link) resolves as one directly in `folder`.
    pub(crate) fn nearest_first(&self, folder: &ScopePath, path: &ScopePath) -> Vec<&TexturePlace> {
        let directory = directory_below(folder, path);
        let segments: Vec<&str> = if directory.is_empty() {
            Vec::new()
        } else {
            directory.split('/').collect()
        };
        // The file's own folder first, the model folder itself last.
        (0..=segments.len())
            .rev()
            .filter_map(|depth| self.own.get(&segments[..depth].join("/")))
            .chain([&self.combined])
            .collect()
    }

    /// `nearest_first`'s places, each with `home`, the directory the texture home's root is
    /// named by, below which each texture sits at its path.
    pub(crate) fn places<'a>(
        &'a self,
        folder: &ScopePath,
        path: &ScopePath,
        home: &'a str,
    ) -> Vec<(&'a TexturePlace, &'a str)> {
        self.nearest_first(folder, path)
            .into_iter()
            .map(|place| (place, home))
            .collect()
    }
}

/// The folder holding the file at `path` below the model folder at `folder`, folded: `""` for
/// a file directly in it, or one not below it.
fn directory_below(folder: &ScopePath, path: &ScopePath) -> String {
    let path = path.fold_key();
    let Some(below) = path
        .strip_prefix(&folder.fold_key())
        .and_then(|rest| rest.strip_prefix('/'))
    else {
        return String::new();
    };
    below
        .rsplit_once('/')
        .map_or_else(String::new, |(directory, _)| directory.to_owned())
}

/// A texture's path below the texture home (`TexturePlace`'s value) as its directory there,
/// `""` or ending in `/` (`jessie/`), and its name: `("jessie/", "skin")` for `jessie/skin`.
pub(crate) fn split(below: &str) -> (&str, &str) {
    match below.rfind('/') {
        Some(slash) => (&below[..=slash], &below[slash + 1..]),
        None => ("", below),
    }
}

/// The path below the texture home of the first texture of `place` that is a variant of the
/// kit reference `stem` (`pants_kitN`; `has_variant_among`, on the name as spelled, a token
/// being spelled exactly), which a path naming the reference is pointed beside: the game
/// respells the reference for the kit picked. `None` when `stem` is no kit reference or `place`
/// holds no variant of its set.
pub(crate) fn variant<'a>(place: &'a TexturePlace, stem: &str) -> Option<&'a str> {
    place
        .values()
        .find(|below| has_variant_among(stem, [split(below).1]))
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(text: &str) -> ScopePath {
        ScopePath::new(text).unwrap()
    }

    /// The textures of slot 05: `skin` in the root and in `jessie/`, `hair` in `jessie/body/`,
    /// and a combined shared face's `skin` and `cap`.
    fn slot_05() -> TextureFolders {
        let mut folders = TextureFolders::default();
        folders.insert(true, "skin");
        folders.insert(true, "Jessie/Skin");
        folders.insert(true, "jessie/body/hair");
        folders.insert(false, "skin");
        folders.insert(false, "cap");
        folders
    }

    /// What the name `stem` resolves to for the file at `file` of slot 05: the first place
    /// holding it, as its path below the home.
    fn resolved(folders: &TextureFolders, file: &str, stem: &str) -> Option<String> {
        folders
            .places(&path("Players/05 - A"), &path(file), "home/")
            .into_iter()
            .find_map(|(place, _)| place.get(&vtree::fold_name(stem)).cloned())
    }

    #[test]
    fn a_name_resolves_in_the_file_s_folder_then_each_parent_then_the_combined_folders() {
        let folders = slot_05();
        let deep = "Players/05 - A/jessie/body/x.mtl";
        assert_eq!(
            resolved(&folders, deep, "skin").as_deref(),
            Some("Jessie/Skin")
        );
        assert_eq!(
            resolved(&folders, deep, "hair").as_deref(),
            Some("jessie/body/hair")
        );
        assert_eq!(resolved(&folders, deep, "cap").as_deref(), Some("cap"));
        let root = "Players/05 - A/face_high.mtl";
        assert_eq!(resolved(&folders, root, "skin").as_deref(), Some("skin"));
        // Never down into a subfolder.
        assert_eq!(resolved(&folders, root, "hair"), None);
        // A shared folder's or a Common file resolves as the root's.
        assert_eq!(
            resolved(&folders, "Faces/Long/hair.mtl", "skin").as_deref(),
            Some("skin")
        );
        assert_eq!(resolved(&folders, "Faces/Long/hair.mtl", "hair"), None);
    }

    #[test]
    fn a_place_s_texture_splits_into_its_directory_below_the_home_and_its_name() {
        assert_eq!(split("jessie/body/skin"), ("jessie/body/", "skin"));
        assert_eq!(split("skin"), ("", "skin"));
    }

    #[test]
    fn a_kit_reference_finds_a_variant_of_its_set() {
        let mut folders = TextureFolders::default();
        folders.insert(true, "jessie/pants_kit2");
        let places = folders.places(
            &path("Players/05 - A"),
            &path("Players/05 - A/jessie/x.mtl"),
            "home/",
        );
        assert_eq!(
            variant(places[0].0, "pants_kitN"),
            Some("jessie/pants_kit2")
        );
        assert_eq!(variant(places[0].0, "socks_kitN"), None);
        assert_eq!(variant(places[0].0, "pants_kit2"), None);
    }
}
