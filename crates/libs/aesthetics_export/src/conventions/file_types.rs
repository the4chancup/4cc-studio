//! File kinds, classified from the name alone. The allowlist is, per folder
//! kind, the set of kinds it admits; `Other` is admitted nowhere, and
//! `strict_file_type_check` changes only the disposition. A stray `.txt`
//! suffix after a link or marker name is tolerated (Windows hides known
//! extensions, so users create `Crocs.boots.txt` with Notepad).

/// What a file in an export is, semantically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FileKind {
    /// A model source, in one of the three supported formats.
    Model(ModelFormat),
    /// An accepted image extension.
    Texture,
    /// A skeleton file (`.skl`).
    Skl,
    /// A Fox cloth/simulation file (`.fclo`).
    Fclo,
    /// An XML file (`.xml`).
    Xml,
    /// A pre-Fox material file (`.mtl`).
    Mtl,
    /// A glTF materials file (`materials.toml` or `<stem>.materials.toml`).
    MaterialsToml,
    /// A `.bin`: a game bin (`face_diff.bin`) or a glTF buffer; deep glTF
    /// parsing tells them apart.
    Bin,
    /// A shared-folder link file: `Crocs.boots`, `Longhair.face`, `Keeper gloves.gloves`.
    SharedLink(SharedKind),
    /// A `<name>.common` link: stands in for `Common/<name>`.
    CommonLink,
    /// A marker file (empty or a `.txt` spelling of one).
    Marker(Marker),
    /// A metadata file: known name, handled outside the allowlist.
    Metadata(MetadataFile),
    /// Any other kind: admitted nowhere.
    Other,
}

/// The three native model formats, by extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ModelFormat {
    /// `.fmdl` (Fox engine).
    Fmdl,
    /// `.model` (pre-Fox engine).
    PesModel,
    /// `.glb` or `.gltf`.
    Gltf,
}

/// A marker file's kind: the directive or fact it declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Marker {
    /// `ingame_face`: remove custom face parts, keep the rest.
    IngameFace,
    /// `fpc.on`: apply the FPC preset.
    FpcOn,
    /// `fpc.off`: un-apply the FPC preset.
    FpcOff,
    /// `pre-fox`: the kit's textures are drawn for the pre-Fox layout.
    PreFox,
    /// `fox`: the kit's textures are drawn for the Fox layout.
    Fox,
}

/// A metadata file by its known name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MetadataFile {
    /// `players.txt`: the authoritative roster.
    PlayersTxt,
    /// `refs.txt`: the legacy referee-roster alias.
    RefsTxt,
    /// `ref_lists.txt`: the referee lists file.
    RefLists,
    /// `notes.txt`: the export's free-text note.
    NotesTxt,
    /// `colors.txt`: team or kit menu colors.
    ColorsTxt,
    /// `icon.txt`: the kit's menu icon number.
    IconTxt,
    /// `config.toml`: the authored kit config.
    ConfigToml,
    /// `settings.toml`: savefile settings for a player.
    SettingsToml,
    /// `readme.txt`: free text for humans.
    Readme,
}

/// The shared folder a link file references: `Crocs.boots` → `Boots/Crocs/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SharedKind {
    /// The `.face` link kind (`Faces/<name>/`).
    Face,
    /// The `.boots` link kind (`Boots/<name>/`).
    Boots,
    /// The `.gloves` link kind (`Gloves/<name>/`).
    Gloves,
}

/// `<stem>.boots` / `.face` / `.gloves`: the extension gives the kind.
const SHARED_LINKS: [(&str, SharedKind); 3] = [
    (".boots", SharedKind::Boots),
    (".face", SharedKind::Face),
    (".gloves", SharedKind::Gloves),
];

/// Marker names: `pre-fox` before `fox` only because they read as a pair.
const MARKERS: [(&str, Marker); 5] = [
    ("ingame_face", Marker::IngameFace),
    ("fpc.on", Marker::FpcOn),
    ("fpc.off", Marker::FpcOff),
    ("pre-fox", Marker::PreFox),
    ("fox", Marker::Fox),
];

/// Known metadata names, whole-name case-insensitive.
const METADATA: [(&str, MetadataFile); 9] = [
    ("players.txt", MetadataFile::PlayersTxt),
    ("refs.txt", MetadataFile::RefsTxt),
    ("ref_lists.txt", MetadataFile::RefLists),
    ("notes.txt", MetadataFile::NotesTxt),
    ("colors.txt", MetadataFile::ColorsTxt),
    ("icon.txt", MetadataFile::IconTxt),
    ("config.toml", MetadataFile::ConfigToml),
    ("settings.toml", MetadataFile::SettingsToml),
    ("readme.txt", MetadataFile::Readme),
];

/// Extensions, last `.`-separated segment: each with its `FileKind`.
const EXTENSIONS: [(&str, FileKind); 19] = [
    ("fmdl", FileKind::Model(ModelFormat::Fmdl)),
    ("model", FileKind::Model(ModelFormat::PesModel)),
    ("glb", FileKind::Model(ModelFormat::Gltf)),
    ("gltf", FileKind::Model(ModelFormat::Gltf)),
    ("dds", FileKind::Texture),
    ("ftex", FileKind::Texture),
    ("png", FileKind::Texture),
    ("jpg", FileKind::Texture),
    ("jpeg", FileKind::Texture),
    ("bmp", FileKind::Texture),
    ("webp", FileKind::Texture),
    ("tga", FileKind::Texture),
    ("tif", FileKind::Texture),
    ("tiff", FileKind::Texture),
    ("skl", FileKind::Skl),
    ("fclo", FileKind::Fclo),
    ("xml", FileKind::Xml),
    ("mtl", FileKind::Mtl),
    ("bin", FileKind::Bin),
];

/// Whether `name` is a root `logo*` texture name: `logo`-prefixed (ASCII-
/// case-insensitively) and an accepted image extension.
pub(crate) fn is_logo_texture(name: &str) -> bool {
    name.get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("logo"))
        && classify(name) == FileKind::Texture
}

/// A shared-folder link's (kind, target name): `Crocs.boots` and the tolerated
/// `Crocs.boots.txt` alike give `(Boots, "Crocs")`.
pub(crate) fn shared_link_name(name: &str) -> Option<(SharedKind, String)> {
    let name = strip_suffix_ci(name, ".txt").unwrap_or(name);
    for (suffix, kind) in SHARED_LINKS {
        if let Some(stem) = strip_suffix_ci(name, suffix)
            && !stem.is_empty()
        {
            return Some((kind, stem.to_owned()));
        }
    }
    None
}

/// A `.common` link's target name: `torso.fmdl.common` and the tolerated
/// `torso.fmdl.common.txt` alike give `torso.fmdl`.
pub(crate) fn common_link_name(name: &str) -> Option<String> {
    let name = strip_suffix_ci(name, ".txt").unwrap_or(name);
    strip_suffix_ci(name, ".common")
        .filter(|stem| !stem.is_empty())
        .map(str::to_owned)
}

impl SharedKind {
    /// The lowercase kind name (`shared_link_duplicate`'s context value).
    pub(crate) fn name(self) -> &'static str {
        match self {
            SharedKind::Face => "face",
            SharedKind::Boots => "boots",
            SharedKind::Gloves => "gloves",
        }
    }
}

/// `name` minus `suffix`, when the tail equals it ASCII-case-insensitively.
fn strip_suffix_ci<'a>(name: &'a str, suffix: &str) -> Option<&'a str> {
    let cut = name.len().checked_sub(suffix.len())?;
    // `get` is None off a char boundary: a non-ASCII tail is no ASCII suffix.
    if name.get(cut..)?.eq_ignore_ascii_case(suffix) {
        Some(&name[..cut])
    } else {
        None
    }
}

/// `.common` links, shared-folder links, markers — the groups the `.txt`
/// tolerance applies to.
fn link_or_marker(name: &str) -> Option<FileKind> {
    if let Some(stem) = strip_suffix_ci(name, ".common")
        && !stem.is_empty()
    {
        return Some(FileKind::CommonLink);
    }
    for (suffix, kind) in SHARED_LINKS {
        if let Some(stem) = strip_suffix_ci(name, suffix)
            && !stem.is_empty()
        {
            return Some(FileKind::SharedLink(kind));
        }
    }
    for (marker_name, marker) in MARKERS {
        if name.eq_ignore_ascii_case(marker_name) {
            return Some(FileKind::Marker(marker));
        }
    }
    None
}

/// One file name's `FileKind`, ASCII-case-insensitive throughout.
pub fn classify(file_name: &str) -> FileKind {
    // The `.txt` tolerance: only when the name without it is a link or marker.
    if let Some(stripped) = strip_suffix_ci(file_name, ".txt")
        && let Some(kind) = link_or_marker(stripped)
    {
        return kind;
    }
    if let Some(kind) = link_or_marker(file_name) {
        return kind;
    }
    for (metadata_name, metadata) in METADATA {
        if file_name.eq_ignore_ascii_case(metadata_name) {
            return FileKind::Metadata(metadata);
        }
    }
    if file_name.eq_ignore_ascii_case("materials.toml")
        || strip_suffix_ci(file_name, ".materials.toml").is_some()
    {
        return FileKind::MaterialsToml;
    }
    let extension = file_name
        .rsplit_once('.')
        .map(|(_, extension)| extension)
        .unwrap_or("");
    for (known, kind) in EXTENSIONS {
        if extension.eq_ignore_ascii_case(known) {
            return kind;
        }
    }
    FileKind::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_covers_the_convention_table() {
        let cases: [(&str, FileKind); 58] = [
            ("Crocs.boots", FileKind::SharedLink(SharedKind::Boots)),
            ("Crocs.BOOTS.txt", FileKind::SharedLink(SharedKind::Boots)),
            ("Longhair.face", FileKind::SharedLink(SharedKind::Face)),
            (
                "Keeper gloves.gloves",
                FileKind::SharedLink(SharedKind::Gloves),
            ),
            (".boots", FileKind::Other),
            ("torso.fmdl.common", FileKind::CommonLink),
            ("torso.fmdl.common.txt", FileKind::CommonLink),
            ("hair.png.common", FileKind::CommonLink),
            ("ボ.common.txt", FileKind::CommonLink),
            (".common", FileKind::Other),
            (".common.txt", FileKind::Other),
            ("ingame_face", FileKind::Marker(Marker::IngameFace)),
            ("INGAME_FACE.txt", FileKind::Marker(Marker::IngameFace)),
            ("fpc.on", FileKind::Marker(Marker::FpcOn)),
            ("fpc.off.txt", FileKind::Marker(Marker::FpcOff)),
            ("pre-fox", FileKind::Marker(Marker::PreFox)),
            ("Fox.txt", FileKind::Marker(Marker::Fox)),
            ("players.txt", FileKind::Metadata(MetadataFile::PlayersTxt)),
            ("README.TXT", FileKind::Metadata(MetadataFile::Readme)),
            ("ref_lists.txt", FileKind::Metadata(MetadataFile::RefLists)),
            ("refs.txt", FileKind::Metadata(MetadataFile::RefsTxt)),
            ("notes.txt", FileKind::Metadata(MetadataFile::NotesTxt)),
            ("colors.txt", FileKind::Metadata(MetadataFile::ColorsTxt)),
            ("icon.txt", FileKind::Metadata(MetadataFile::IconTxt)),
            ("config.toml", FileKind::Metadata(MetadataFile::ConfigToml)),
            (
                "settings.toml",
                FileKind::Metadata(MetadataFile::SettingsToml),
            ),
            ("materials.toml", FileKind::MaterialsToml),
            ("body.materials.toml", FileKind::MaterialsToml),
            ("face_high.fmdl", FileKind::Model(ModelFormat::Fmdl)),
            ("boots.MODEL", FileKind::Model(ModelFormat::PesModel)),
            ("x.glb", FileKind::Model(ModelFormat::Gltf)),
            ("x.gltf", FileKind::Model(ModelFormat::Gltf)),
            ("x.dds", FileKind::Texture),
            ("x.ftex", FileKind::Texture),
            ("x.png", FileKind::Texture),
            ("x.jpg", FileKind::Texture),
            ("x.jpeg", FileKind::Texture),
            ("x.bmp", FileKind::Texture),
            ("x.webp", FileKind::Texture),
            ("x.tga", FileKind::Texture),
            ("x.tif", FileKind::Texture),
            ("x.tiff", FileKind::Texture),
            ("x.skl", FileKind::Skl),
            ("x.fclo", FileKind::Fclo),
            ("face_diff.bin", FileKind::Bin),
            ("x.xml", FileKind::Xml),
            ("x.mtl", FileKind::Mtl),
            ("foo.txt", FileKind::Other),
            ("x.exe", FileKind::Other),
            ("boots.txt", FileKind::Other),
            ("notes.txt.txt", FileKind::Other),
            ("settings.toml.txt", FileKind::Other),
            ("config.toml.txt", FileKind::Other),
            ("x.bat", FileKind::Other),
            ("no_extension", FileKind::Other),
            ("ボボ", FileKind::Other),
            ("スパイク.boots", FileKind::SharedLink(SharedKind::Boots)),
            ("髪.png", FileKind::Texture),
        ];
        for (name, expected) in cases {
            assert_eq!(classify(name), expected, "{name:?}");
        }
    }
}
