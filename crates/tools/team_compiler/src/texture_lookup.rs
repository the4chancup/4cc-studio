//! Where a texture name resolves, nearest first (`aesthetics_export/player_folders.md`
//! "Subfolders"): a texture a model or a `.mtl` of a model folder names with no path resolves
//! in the file's own folder first, then in each parent up to the model folder, among the
//! folder's own textures and links and those of the shared folders it combines at each
//! place — a combined folder's files stand at their paths below the model folder, a combined
//! `jessie/skin.dds` beside the model's own `jessie/` files, the folder's own winning a path
//! both hold; a lookup by name never goes down into a subfolder. One named with a path below
//! the file's folder (`./textures/skin`, `path_below`) resolves at that path alone (`at`).
//! The face task and the Models task
//! point texture paths with it, and the deep pass looks a `.mtl`'s paths up with it, so a path
//! the deep pass calls supplied is one the tasks point.

use std::collections::BTreeMap;

use vtree::ScopePath;

use crate::kit_variants::{has_variant_among, kit_lookup_key};
use crate::paths::TextureDirectory;

/// One folder's textures, each by its lookup key (`kit_lookup_key`: the stem folded, or the
/// set's folded `kitN` reference for a `kitF` fallback, which a path never finds by its own
/// spelling). Each value is the texture's source path below its folder without its
/// extension, as spelled (`PlayerFile::Texture`'s `below`): the path its converted file has
/// under the texture home (`paths::TextureHome::texture`), which for a `kitF` entry
/// (`pants_kitF`, filed under its set's `kitN` key) never is — a consumer points a
/// directory at it, never writes its name.
pub(crate) type TexturePlace = BTreeMap<String, String>;

/// Which home a place's textures sit in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlaceHome {
    /// The model folder's texture home: its own textures and its combined folders'.
    Folder,
    /// The team's Common output: the textures its links stand for.
    Common,
}

impl PlaceHome {
    /// The directory a place of this home names its textures by: the model folder's `home`,
    /// or `common`, the team's Common output.
    pub(crate) fn directory<'a>(
        self,
        home: &'a TextureDirectory,
        common: &'a TextureDirectory,
    ) -> &'a TextureDirectory {
        match self {
            PlaceHome::Folder => home,
            PlaceHome::Common => common,
        }
    }
}

/// A model folder's textures by the folder directly holding each, and its texture links the
/// same way, for a texture name to resolve nearest first (`places`): a `.common` texture link
/// counts as the texture it stands for being present in the folder holding the link
/// (`model_format.md` "Link files"), so its place is that folder's and its home the team's
/// Common output.
#[derive(Debug, Default)]
pub(crate) struct TextureFolders {
    /// The model folder's own textures, per folder of its tree, keyed by that folder's path
    /// below the model folder, folded: `""` for the model folder itself, `jessie/body` for
    /// its subfolder `jessie/body/`.
    own: BTreeMap<String, TexturePlace>,
    /// The model folder's own texture links, keyed as `own` is: a link mirrors `Common/`'s
    /// tree, so the folder holding it is its target's directory below `Common/`
    /// (`jessie/hair.dds.common` sits in `jessie/`).
    links: BTreeMap<String, TexturePlace>,
    /// The textures of the shared folders the model folder combines, keyed as `own` is, by
    /// each file's directory below its shared folder: a combined folder's files stand at
    /// their paths below the model folder, its `jessie/` the same `jessie/` there.
    combined: BTreeMap<String, TexturePlace>,
    /// The combined shared folders' texture links, keyed as `combined` is.
    combined_links: BTreeMap<String, TexturePlace>,
}

impl TextureFolders {
    /// Adds the texture at `below`, its path below its source folder (`PlayerFile::Texture`'s
    /// `below`): one of the model folder's own when `own` is set, else a combined shared
    /// folder's, at the directory `below` sits in. The first texture of a stem in a place
    /// keeps it: validation drops a folder holding two of one stem in one directory
    /// (`texture_stem_conflict`), and copies of a stem across the sources share one path,
    /// which the textures task fills.
    pub(crate) fn insert(&mut self, own: bool, below: &str) {
        self.place(own, false, below);
    }

    /// Adds the texture `.common` link standing for the `Common/` texture at `below`, its
    /// path below `Common/` without its extension (`PlayerFile::CommonTexture`): one of the
    /// model folder's own when `own` is set, else a combined shared folder's. The folder a
    /// name resolves the link in is its target's directory (`split`'s first), as for a
    /// texture there (`insert`); the first link of a stem in a place keeps it, as a texture's
    /// does.
    pub(crate) fn insert_link(&mut self, own: bool, below: &str) {
        self.place(own, true, below);
    }

    /// `insert` and `insert_link`'s one place insert: `link` picks the texture-link maps.
    fn place(&mut self, own: bool, link: bool, below: &str) {
        let (directory, name) = split(below);
        let key = vtree::fold_name(directory.trim_end_matches('/'));
        let places = match (own, link) {
            (true, false) => &mut self.own,
            (true, true) => &mut self.links,
            (false, false) => &mut self.combined,
            (false, true) => &mut self.combined_links,
        };
        places
            .entry(key)
            .or_default()
            // A `kitF` file goes in under its set's `kitN` key, which a path naming its own
            // spelling never finds while the set's reference does (`variant`).
            .entry(kit_lookup_key(name))
            .or_insert_with(|| below.to_owned());
    }

    /// The places a texture name of the file at `path` resolves in, nearest first, each with
    /// its home: `folder` is the file's own source folder, the model folder for the folder's
    /// own files and the shared folder's path for a combined folder's (`Common/` for a Common
    /// file, `common_folder`), and the file's own folder below it first, then each parent up
    /// to it, each directory's own textures, links, combined textures and combined links in
    /// order, so the model folder's own wins a path it and a combined folder both hold. A
    /// file not below `folder` resolves as one directly in it.
    pub(crate) fn nearest_first(
        &self,
        folder: &ScopePath,
        path: &ScopePath,
    ) -> Vec<(&TexturePlace, PlaceHome)> {
        let directory = directory_below(folder, path).unwrap_or_default();
        let segments: Vec<&str> = if directory.is_empty() {
            Vec::new()
        } else {
            directory.split('/').collect()
        };
        // The file's own folder first, the source folder itself last.
        (0..=segments.len())
            .rev()
            .flat_map(|depth| self.folder(&segments[..depth].join("/")))
            .collect()
    }

    /// The places of the folder at `directory`, a path below the model folder (`""` for the
    /// model folder itself, `jessie` for a combined folder's `jessie/` too, its files standing
    /// at their paths below the model folder): the folder's own textures, its links, the
    /// combined folders' textures and their links, each with its home.
    pub(crate) fn folder(&self, directory: &str) -> Vec<(&TexturePlace, PlaceHome)> {
        let key = vtree::fold_name(directory.trim_end_matches('/'));
        [
            self.own.get(&key).map(|place| (place, PlaceHome::Folder)),
            self.links.get(&key).map(|place| (place, PlaceHome::Common)),
            self.combined
                .get(&key)
                .map(|place| (place, PlaceHome::Folder)),
            self.combined_links
                .get(&key)
                .map(|place| (place, PlaceHome::Common)),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    /// `nearest_first`'s places, each with the directory its home names it by: `home`, the
    /// model folder's texture home, or `common`, the team's Common output.
    pub(crate) fn places<'a>(
        &'a self,
        folder: &ScopePath,
        path: &ScopePath,
        home: &'a TextureDirectory,
        common: &'a TextureDirectory,
    ) -> Vec<(&'a TexturePlace, &'a TextureDirectory)> {
        self.nearest_first(folder, path)
            .into_iter()
            .map(|(place, which)| (place, which.directory(home, common)))
            .collect()
    }

    /// The places of the folder `subdirectory` (`path_below`'s) below the folder of the file at
    /// `file`, which a reference of that file naming a path resolves in alone: `folder` is the
    /// file's own source folder, as `nearest_first` takes it. Empty when `file` is not below
    /// `folder`, when a segment of `subdirectory` is `.` or `..` or empty,
    /// or when nothing sits there.
    pub(crate) fn at(
        &self,
        folder: &ScopePath,
        file: &ScopePath,
        subdirectory: &str,
    ) -> Vec<(&TexturePlace, PlaceHome)> {
        let Some(directory) = directory_below(folder, file) else {
            return Vec::new();
        };
        let subdirectory = subdirectory.strip_suffix('/').unwrap_or(subdirectory);
        if subdirectory
            .split('/')
            .any(|segment| matches!(segment, "" | "." | ".."))
        {
            return Vec::new();
        }
        let key = if directory.is_empty() {
            subdirectory.to_owned()
        } else {
            format!("{directory}/{subdirectory}")
        };
        self.folder(&key)
    }
}

/// The subdirectory a texture reference names below its file's folder (`model_format.md`
/// "Stem-based texture references"): for a directory starting with `./` and carrying more,
/// what follows the `./` (`./textures/` → `textures/`, `./a/b/` → `a/b/`). `None` for `./`,
/// an empty directory, or any other (a game path: `/Assets/...`, `model/character/...`),
/// whose references are looked up by name.
pub(crate) fn path_below(directory: &str) -> Option<&str> {
    directory
        .strip_prefix("./")
        .filter(|subdirectory| !subdirectory.is_empty())
}

/// The folder holding the file at `path` below the model folder at `folder`, folded: `""` for
/// a file directly in it; `None` for one not below it.
fn directory_below(folder: &ScopePath, path: &ScopePath) -> Option<String> {
    let path = path.fold_key();
    let below = path
        .strip_prefix(&folder.fold_key())
        .and_then(|rest| rest.strip_prefix('/'))?;
    Some(
        below
            .rsplit_once('/')
            .map_or_else(String::new, |(directory, _)| directory.to_owned()),
    )
}

/// A texture's path below the texture home (`TexturePlace`'s value) as its directory there,
/// `""` or ending in `/` (`jessie/`), and its name: `("jessie/", "skin")` for `jessie/skin`.
pub(crate) fn split(below: &str) -> (&str, &str) {
    match below.rfind('/') {
        Some(slash) => (&below[..=slash], &below[slash + 1..]),
        None => ("", below),
    }
}

/// The path below the texture home of the first texture of `place` that is a member of the
/// kit reference `stem`'s set (`pants_kitN`; `has_variant_among`, on the name as spelled, a
/// token being spelled exactly) — a variant of it, or its `kitF` fallback — which a path
/// naming the reference is pointed beside: the game
/// respells the reference for the kit picked. `None` when `stem` is no kit reference or `place`
/// holds no member of its set.
pub(crate) fn variant<'a>(place: &'a TexturePlace, stem: &str) -> Option<&'a str> {
    place
        .values()
        .find(|below| has_variant_among(stem, [split(below).1]))
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::plan::roles::common_folder;

    fn path(text: &str) -> ScopePath {
        ScopePath::new(text).unwrap()
    }

    /// The textures of slot 05: `skin` in the root and in `jessie/`, `hair` in `jessie/body/`,
    /// and a combined shared face's `skin` and `cap` at the root and `skin` and `x` in
    /// `jessie/` and `jessie/sub/`, each at its path below its folder.
    fn slot_05() -> TextureFolders {
        let mut folders = TextureFolders::default();
        folders.insert(true, "skin");
        folders.insert(true, "Jessie/Skin");
        folders.insert(true, "jessie/body/hair");
        folders.insert(false, "skin");
        folders.insert(false, "cap");
        folders.insert(false, "jessie/skin");
        folders.insert(false, "jessie/sub/x");
        folders
    }

    /// What the name `stem` resolves to for the file at `file` of the folder at `folder`,
    /// its own source folder: the first place holding it, as its path below the home.
    fn resolved(folders: &TextureFolders, folder: &str, file: &str, stem: &str) -> Option<String> {
        let home = TextureDirectory::plain("home/".to_owned());
        let common = TextureDirectory::plain("common/".to_owned());
        folders
            .places(&path(folder), &path(file), &home, &common)
            .into_iter()
            .find_map(|(place, _)| place.get(&vtree::fold_name(stem)).cloned())
    }

    #[test]
    fn a_name_resolves_in_the_file_s_folder_then_each_parent() {
        let folders = slot_05();
        let player = "Players/05 - A";
        let deep = "Players/05 - A/jessie/body/x.mtl";
        assert_eq!(
            resolved(&folders, player, deep, "skin").as_deref(),
            Some("Jessie/Skin")
        );
        assert_eq!(
            resolved(&folders, player, deep, "hair").as_deref(),
            Some("jessie/body/hair")
        );
        assert_eq!(
            resolved(&folders, player, deep, "cap").as_deref(),
            Some("cap")
        );
        let root = "Players/05 - A/face_high.mtl";
        assert_eq!(
            resolved(&folders, player, root, "skin").as_deref(),
            Some("skin")
        );
        // Never down into a subfolder.
        assert_eq!(resolved(&folders, player, root, "hair"), None);
    }

    #[test]
    fn a_combined_folder_s_file_resolves_in_its_own_directory_then_each_parent() {
        // A combined `jessie/` part's `skin` is the combined folder's own `jessie/skin`
        // when the player holds none there, before the root's.
        let mut folders = TextureFolders::default();
        folders.insert(true, "skin");
        folders.insert(false, "jessie/skin");
        let file = "Faces/Round/jessie/hair_high.fmdl";
        assert_eq!(
            resolved(&folders, "Faces/Round", file, "skin").as_deref(),
            Some("jessie/skin")
        );

        // The player's own `jessie/skin` wins over the combined folder's at the path both
        // hold, as his root's wins at the root.
        let folders = slot_05();
        assert_eq!(
            resolved(&folders, "Faces/Round", file, "skin").as_deref(),
            Some("Jessie/Skin")
        );
        // And his own `jessie/skin` is what a file of his at `jessie/` resolves, where the
        // combined folder's `skin` at the root is the fallback.
        assert_eq!(
            resolved(
                &folders,
                "Players/05 - A",
                "Players/05 - A/jessie/x.mtl",
                "skin"
            )
            .as_deref(),
            Some("Jessie/Skin")
        );
    }

    #[test]
    fn a_combined_file_s_path_below_resolves_at_that_path() {
        // A combined `jessie/` `.mtl` naming `./sub/` resolves at `jessie/sub/` below the
        // player folder, where the shared folder's `jessie/sub/` files stand.
        let mut folders = TextureFolders::default();
        folders.insert(false, "jessie/sub/x");
        let places = folders.at(
            &path("Faces/Round"),
            &path("Faces/Round/jessie/x.mtl"),
            "sub/",
        );
        assert_eq!(places.len(), 1);
        let (place, home) = places[0];
        assert_eq!(home, PlaceHome::Folder);
        assert_eq!(place.values().collect::<Vec<_>>(), ["jessie/sub/x"]);
    }

    #[test]
    fn a_directory_starting_with_dot_slash_and_carrying_more_names_a_path_below() {
        assert_eq!(path_below("./textures/"), Some("textures/"));
        assert_eq!(path_below("./a/b/"), Some("a/b/"));
        for directory in [
            "./",
            "",
            "/Assets/pes16/model/character/common/740/sourceimages/",
            "model/character/uniform/common/740/",
            "textures/",
        ] {
            assert_eq!(path_below(directory), None, "{directory}");
        }
    }

    #[test]
    fn a_path_below_a_file_s_folder_resolves_at_that_place_alone() {
        let mut folders = slot_05();
        folders.insert(true, "jessie/textures/skin");
        let folder = path("Players/05 - A");
        let in_jessie = path("Players/05 - A/jessie/x.mtl");
        let places = folders.at(&folder, &in_jessie, "textures/");
        assert_eq!(places.len(), 1);
        let (place, home) = places[0];
        assert_eq!(home, PlaceHome::Folder);
        assert_eq!(place.values().collect::<Vec<_>>(), ["jessie/textures/skin"]);
        // Folded as the folders' keys are.
        assert_eq!(folders.at(&folder, &in_jessie, "Textures/"), places);
        // From the root, `body/` is not below it: `jessie/body/` is.
        let root = path("Players/05 - A/face_high.mtl");
        assert!(folders.at(&folder, &root, "body/").is_empty());
        let places = folders.at(&folder, &root, "jessie/body/");
        assert_eq!(places.len(), 1);
        assert_eq!(
            places[0].0.get("hair").map(String::as_str),
            Some("jessie/body/hair")
        );
        // A file not below the folder, and a segment that is not a folder's name.
        assert!(
            folders
                .at(&folder, &path("Faces/Long/textures/x.mtl"), "textures/")
                .is_empty()
        );
        let in_body = path("Players/05 - A/jessie/body/x.mtl");
        for subdirectory in ["/", "../x/", "../textures/", "./textures/", "textures//"] {
            assert!(
                folders.at(&folder, &in_body, subdirectory).is_empty(),
                "{subdirectory}"
            );
        }
        // From the root, `/` would otherwise name the root's own folder, which holds `skin`.
        assert!(folders.at(&folder, &root, "/").is_empty());
    }

    #[test]
    fn a_link_counts_as_the_texture_present_in_its_own_folder() {
        // `jessie/hair.dds.common` stands for `Common/jessie/hair.dds` in `jessie/`'s
        // namespace: a model there naming `hair` finds the link, in the Common home, before
        // the root's own `hair.dds`.
        let mut folders = TextureFolders::default();
        folders.insert(true, "hair");
        folders.insert_link(true, "jessie/hair");
        let model = path("Players/05 - A/jessie/hair_high.fmdl");
        let places = folders.nearest_first(&path("Players/05 - A"), &model);
        let first = places
            .iter()
            .find_map(|(place, home)| place.get("hair").map(|below| (below, home)));
        assert_eq!(first, Some((&"jessie/hair".to_owned(), &PlaceHome::Common)));

        // A root's link yields to `jessie/`'s own texture of the stem, as the root's own
        // does: each folder's textures before its links.
        let mut folders = TextureFolders::default();
        folders.insert(true, "jessie/hair");
        folders.insert_link(true, "hair");
        let places = folders.nearest_first(&path("Players/05 - A"), &model);
        let first = places
            .iter()
            .find_map(|(place, home)| place.get("hair").map(|below| (below, home)));
        assert_eq!(first, Some((&"jessie/hair".to_owned(), &PlaceHome::Folder)));

        // And a link of the folder's comes before a combined shared folder's texture of the
        // stem, as the folder's own texture already does.
        let mut folders = TextureFolders::default();
        folders.insert_link(true, "hair");
        folders.insert(false, "hair");
        let root = path("Players/05 - A/face_high.fmdl");
        let places = folders.nearest_first(&path("Players/05 - A"), &root);
        let first = places
            .iter()
            .find_map(|(place, home)| place.get("hair").map(|below| (below, home)));
        assert_eq!(first, Some((&"hair".to_owned(), &PlaceHome::Common)));
    }

    #[test]
    fn a_path_below_finds_the_link_at_that_path_in_the_common_home() {
        // `at` of a root file for `jessie/` gives its links after its textures: the link
        // `./jessie/hair` resolves to.
        let mut folders = TextureFolders::default();
        folders.insert_link(true, "jessie/hair");
        let places = folders.at(
            &path("Players/05 - A"),
            &path("Players/05 - A/face_high.fmdl"),
            "jessie/",
        );
        assert_eq!(places.len(), 1);
        let (place, home) = places[0];
        assert_eq!(home, PlaceHome::Common);
        assert_eq!(place.values().collect::<Vec<_>>(), ["jessie/hair"]);
    }

    #[test]
    fn the_common_folder_matches_whatever_case_the_export_spells_it_in() {
        // `common_folder` is compared folded (`directory_below`), so a `common/` export's
        // `.mtl` resolves below `Common/` as a `Common/` one's does.
        let mut folders = TextureFolders::default();
        folders.insert(true, "jessie/hair");
        let places = folders.nearest_first(&common_folder(), &path("common/jessie/hair.mtl"));
        assert_eq!(
            places[0].0.get("hair").map(String::as_str),
            Some("jessie/hair")
        );
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
        let home = TextureDirectory::plain("home/".to_owned());
        let common = TextureDirectory::plain("common/".to_owned());
        let places = folders.places(
            &path("Players/05 - A"),
            &path("Players/05 - A/jessie/x.mtl"),
            &home,
            &common,
        );
        assert_eq!(
            variant(places[0].0, "pants_kitN"),
            Some("jessie/pants_kit2")
        );
        assert_eq!(variant(places[0].0, "socks_kitN"), None);
        assert_eq!(variant(places[0].0, "pants_kit2"), None);
    }

    #[test]
    fn a_kit_reference_finds_the_set_s_fallback_which_its_own_name_does_not_find() {
        let mut folders = TextureFolders::default();
        folders.insert(true, "jessie/pants_kitF");
        let home = TextureDirectory::plain("home/".to_owned());
        let common = TextureDirectory::plain("common/".to_owned());
        let places = folders.places(
            &path("Players/05 - A"),
            &path("Players/05 - A/jessie/x.mtl"),
            &home,
            &common,
        );
        let place = places[0].0;
        // `kitF` goes by its set's `kitN` key: a `kitN` reference finds the set, a `kitF`
        // name finds nothing.
        assert_eq!(variant(place, "pants_kitN"), Some("jessie/pants_kitF"));
        assert_eq!(
            place.get("pants_kitn"),
            Some(&"jessie/pants_kitF".to_owned())
        );
        assert_eq!(place.get("pants_kitf"), None);
        assert_eq!(variant(place, "pants_kitF"), None);
    }
}
