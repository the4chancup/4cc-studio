//! In-memory virtual file tree and the canonical `ScopePath` / `RelativeScopePath` types.

use std::collections::BTreeMap;
use std::fmt;
use std::ops::Bound;

use unicode_normalization::UnicodeNormalization;

/// Canonical path of a file or folder inside an export or other scope root.
/// Forward slashes, no leading or trailing slash, original casing preserved.
/// Equality, `Hash` and `Ord` are exact (case-sensitive); case- and
/// Unicode-insensitive comparison goes through [`ScopePath::fold_key`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopePath(String);

impl ScopePath {
    /// Normalizes `\` to `/`, strips one trailing `/`, then validates every
    /// segment against [`PathError`].
    pub fn new(text: &str) -> Result<ScopePath, PathError> {
        Ok(ScopePath(normalize(text)?))
    }

    /// The canonical forward-slash path, e.g. `Kits/p1/kit.dds`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The path's segments in order.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }

    /// The last segment (file or folder name).
    pub fn name(&self) -> &str {
        // A validated path always has at least one non-empty segment.
        match self.0.rsplit('/').next() {
            Some(name) => name,
            None => &self.0,
        }
    }

    /// The parent path, or `None` for a single-segment path.
    pub fn parent(&self) -> Option<ScopePath> {
        let (parent, _) = self.0.rsplit_once('/')?;
        Some(ScopePath(parent.to_owned()))
    }

    /// Builds a path relative to this one as the scope root. This is the only
    /// constructor of [`RelativeScopePath`]; the text is validated exactly like
    /// [`ScopePath::new`].
    pub fn relative(&self, text: &str) -> Result<RelativeScopePath, PathError> {
        let normalized = normalize(text)?;
        Ok(RelativeScopePath {
            root: self.clone(),
            relative: normalized,
        })
    }

    /// NFC, then NTFS-style simple case folding (per-character uppercase, then
    /// lowercase), used for collision detection and case-insensitive lookup.
    /// Folding is per segment, so a `/` can never appear from normalization.
    pub fn fold_key(&self) -> String {
        fold_segments(self.0.split('/'))
    }
}

impl fmt::Display for ScopePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for ScopePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A canonical path relative to a validated scope root; carries its root so it
/// can only be joined back to it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelativeScopePath {
    root: ScopePath,
    relative: String,
}

impl RelativeScopePath {
    /// The scope root this path was built against.
    pub fn root(&self) -> &ScopePath {
        &self.root
    }

    /// The relative part (`face.fmdl`, `textures/kit.dds`).
    pub fn as_str(&self) -> &str {
        &self.relative
    }

    /// The relative path's segments in order.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.relative.split('/')
    }

    /// The full canonical path: this path joined back to its root. Infallible:
    /// both halves are already canonical.
    pub fn to_scope_path(&self) -> ScopePath {
        ScopePath(format!("{}/{}", self.root.as_str(), self.relative))
    }
}

/// Why a string is not a canonical path. Stable variants usable as finding
/// codes; no user prose beyond the `Display` text.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PathError {
    /// `""` or only slashes.
    #[error("path is empty")]
    Empty,
    /// Leading `/` or `\`, drive letter (`C:`), or UNC (`\\`).
    #[error("path is absolute")]
    Absolute,
    /// A `..` segment.
    #[error("path contains a `..` traversal segment")]
    Traversal,
    /// An empty segment between separators (`a//b`).
    #[error("path contains an empty segment")]
    EmptySegment,
    /// `.`, a control character, one of `< > : " | ? *`, or a segment ending
    /// with `.` or a space (Windows strips those, which would collide).
    #[error("path segment is not canonical: {segment:?}")]
    InvalidSegment {
        /// The offending segment.
        segment: String,
    },
}

/// Why an insert was refused. The tree never replaces silently.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InsertError {
    /// Same fold key and same spelling: the file is already in the tree.
    #[error("file already exists in the tree")]
    Duplicate,
    /// Same fold key but a different spelling (case or Unicode normalization
    /// form); on a real filesystem these names would collide.
    #[error("path collides with existing entry {existing}")]
    Collision {
        /// The stored path that shares the fold key.
        existing: ScopePath,
    },
    /// Inserting `a/b` when file `a` exists (by fold key), or `a` when files
    /// under `a/` exist.
    #[error("path conflicts with existing entry {existing}")]
    FileFolderConflict {
        /// The stored file path in the way.
        existing: ScopePath,
    },
}

/// One immediate child of a folder: a stored file or an implicit folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// A file stored in the tree, with its real casing.
    File(ScopePath),
    /// An implicit folder containing at least one file below it.
    Folder(ScopePath),
}

/// In-memory file tree: files only, folders are implicit from the paths.
/// Lookups are case- and Unicode-normalization-insensitive; stored paths keep
/// their real casing.
#[derive(Debug)]
pub struct VirtualTree<T> {
    /// Fold key -> (real-cased path, value). Sorted by fold key, so children
    /// and subtree scans are prefix range walks.
    files: BTreeMap<String, (ScopePath, T)>,
}

impl<T> VirtualTree<T> {
    /// An empty tree.
    pub fn new() -> Self {
        VirtualTree {
            files: BTreeMap::new(),
        }
    }

    /// Number of files stored.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether the tree stores no files.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Stores `value` at `path`. Refuses with [`InsertError`] rather than
    /// silently replacing or colliding.
    pub fn insert(&mut self, path: ScopePath, value: T) -> Result<(), InsertError> {
        let key = path.fold_key();
        if let Some((existing, _)) = self.files.get(&key) {
            return Err(if existing == &path {
                InsertError::Duplicate
            } else {
                InsertError::Collision {
                    existing: existing.clone(),
                }
            });
        }
        // A stored file at any ancestor fold key blocks a deeper insert.
        let mut ancestor = String::new();
        for segment in key.split('/') {
            if !ancestor.is_empty() {
                ancestor.push('/');
            }
            ancestor.push_str(segment);
            if ancestor.len() == key.len() {
                break;
            }
            if let Some((existing, _)) = self.files.get(&ancestor) {
                return Err(InsertError::FileFolderConflict {
                    existing: existing.clone(),
                });
            }
        }
        // Files already stored under this path block it becoming a file.
        let prefix = format!("{key}/");
        if let Some((_, (existing, _))) = self.with_prefix(prefix).next() {
            return Err(InsertError::FileFolderConflict {
                existing: existing.clone(),
            });
        }
        // Folders are implicit: a stored file under a fold-equal but
        // differently spelled ancestor would give the folder an ambiguous name
        // on disk, so that is a collision too.
        let real_segments: Vec<&str> = path.as_str().split('/').collect();
        let fold_segments: Vec<&str> = key.split('/').collect();
        for depth in 1..real_segments.len() {
            let prefix = format!("{}/", fold_segments[..depth].join("/"));
            if let Some((_, (existing, _))) = self.with_prefix(prefix).next() {
                let existing_prefix = existing
                    .segments()
                    .take(depth)
                    .collect::<Vec<_>>()
                    .join("/");
                if existing_prefix != real_segments[..depth].join("/") {
                    return Err(InsertError::Collision {
                        existing: existing.clone(),
                    });
                }
            }
        }
        self.files.insert(key, (path, value));
        Ok(())
    }

    /// The value at `path`, looked up case- and normalization-insensitively.
    pub fn get(&self, path: &ScopePath) -> Option<&T> {
        self.files.get(&path.fold_key()).map(|(_, value)| value)
    }

    /// Mutable access to the value at `path`.
    pub fn get_mut(&mut self, path: &ScopePath) -> Option<&mut T> {
        self.files.get_mut(&path.fold_key()).map(|(_, value)| value)
    }

    /// Removes the file at `path`, returning its value if present.
    pub fn remove(&mut self, path: &ScopePath) -> Option<T> {
        self.files.remove(&path.fold_key()).map(|(_, value)| value)
    }

    /// Whether a file is stored at `path`.
    pub fn contains_file(&self, path: &ScopePath) -> bool {
        self.files.contains_key(&path.fold_key())
    }

    /// Whether at least one file lives under `folder/`.
    pub fn contains_folder(&self, folder: &ScopePath) -> bool {
        let prefix = format!("{}/", folder.fold_key());
        self.with_prefix(prefix).next().is_some()
    }

    /// Immediate children of `folder` (`None` = the root): each file once and
    /// each implicit folder once, real-cased, sorted by fold key.
    pub fn children(&self, folder: Option<&ScopePath>) -> Vec<Entry> {
        let depth = folder.map(|f| f.segments().count()).unwrap_or(0);
        let prefix = match folder {
            Some(f) => format!("{}/", f.fold_key()),
            None => String::new(),
        };
        let mut children: BTreeMap<String, Entry> = BTreeMap::new();
        for (key, (path, _)) in self.with_prefix(prefix.clone()) {
            let rest = &key[prefix.len()..];
            let first_segment = rest.split('/').next().unwrap_or_default();
            children.entry(first_segment.to_owned()).or_insert_with(|| {
                // The child's real casing comes from the stored path's segment.
                let child_path = ScopePath(
                    path.segments()
                        .take(depth + 1)
                        .collect::<Vec<_>>()
                        .join("/"),
                );
                if rest.contains('/') {
                    Entry::Folder(child_path)
                } else {
                    Entry::File(child_path)
                }
            });
        }
        children.into_values().collect()
    }

    /// Every file under `folder` (`None` = whole tree), recursive, sorted by
    /// fold key.
    pub fn files_under(
        &self,
        folder: Option<&ScopePath>,
    ) -> impl Iterator<Item = (&ScopePath, &T)> {
        let prefix = match folder {
            Some(f) => format!("{}/", f.fold_key()),
            None => String::new(),
        };
        self.with_prefix(prefix)
            .map(|(_, (path, value))| (path, value))
    }

    /// All files, sorted by fold key.
    pub fn iter(&self) -> impl Iterator<Item = (&ScopePath, &T)> {
        self.files.values().map(|(path, value)| (path, value))
    }

    /// Every stored entry whose fold key starts with `prefix`, in key order: a
    /// bounded range walk, since keys with a common prefix are contiguous in
    /// the map.
    fn with_prefix<'a>(
        &'a self,
        prefix: String,
    ) -> impl Iterator<Item = (&'a String, &'a (ScopePath, T))> + 'a {
        self.files
            .range::<str, _>((Bound::Included(prefix.as_str()), Bound::Unbounded))
            .take_while(move |(key, _)| key.starts_with(prefix.as_str()))
    }
}

impl<T> Default for VirtualTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Applies the shared normalization: `\` to `/`, one trailing `/` stripped,
/// every segment validated. Returns the canonical text.
fn normalize(text: &str) -> Result<String, PathError> {
    let normalized = text.replace('\\', "/");
    if normalized.is_empty() || normalized.chars().all(|c| c == '/') {
        return Err(PathError::Empty);
    }
    if normalized.starts_with('/') {
        return Err(PathError::Absolute);
    }
    let path = normalized.strip_suffix('/').unwrap_or(&normalized);
    validate_segments(path)?;
    Ok(path.to_owned())
}

/// Validates every `/`-separated segment of an already slash-normalized path.
fn validate_segments(path: &str) -> Result<(), PathError> {
    // A drive-letter first segment is an absolute path, reported separately
    // from the general ':' rejection below.
    let first = path.split('/').next().unwrap_or_default();
    if first.len() == 2
        && first.ends_with(':')
        && first.starts_with(|c: char| c.is_ascii_alphabetic())
    {
        return Err(PathError::Absolute);
    }
    for segment in path.split('/') {
        if segment.is_empty() {
            return Err(PathError::EmptySegment);
        }
        if segment == ".." {
            return Err(PathError::Traversal);
        }
        let invalid = segment == "."
            || segment
                .chars()
                .any(|c| c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
            || segment.ends_with('.')
            || segment.ends_with(' ');
        if invalid {
            return Err(PathError::InvalidSegment {
                segment: segment.to_owned(),
            });
        }
    }
    Ok(())
}

/// Folds each segment to NFC + NTFS-style simple case folding and joins with
/// `/`. NTFS compares names per character after uppercasing, so the fold is
/// per-character simple uppercase (a multi-char mapping like `ß` → `SS` keeps
/// the original character, mirroring NTFS keeping `ß` distinct from `ss`),
/// then lowercase — which also merges `ς`/`σ` the way NTFS does.
fn fold_segments<'a>(segments: impl Iterator<Item = &'a str>) -> String {
    segments
        .map(|segment| {
            UnicodeNormalization::nfc(segment.chars())
                .map(|c| {
                    let mut upper = c.to_uppercase();
                    match (upper.next(), upper.next()) {
                        (Some(single), None) => single,
                        _ => c,
                    }
                })
                .collect::<String>()
                .to_lowercase()
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(text: &str) -> ScopePath {
        ScopePath::new(text).unwrap()
    }

    #[test]
    fn new_normalizes_backslashes_and_strips_one_trailing_slash() {
        assert_eq!(path(r"Kits\p1\kit.dds").as_str(), "Kits/p1/kit.dds");
        assert_eq!(path("a/b/").as_str(), "a/b");
    }

    #[test]
    fn new_rejects_empty_and_slash_only() {
        assert_eq!(ScopePath::new(""), Err(PathError::Empty));
        assert_eq!(ScopePath::new("/"), Err(PathError::Empty));
        assert_eq!(ScopePath::new("\\\\"), Err(PathError::Empty));
    }

    #[test]
    fn new_rejects_absolute_paths() {
        assert_eq!(ScopePath::new("/a"), Err(PathError::Absolute));
        assert_eq!(ScopePath::new("C:/a"), Err(PathError::Absolute));
        assert_eq!(ScopePath::new(r"C:\a"), Err(PathError::Absolute));
        assert_eq!(ScopePath::new(r"\\server\a"), Err(PathError::Absolute));
    }

    #[test]
    fn new_rejects_traversal_and_bad_segments() {
        assert_eq!(ScopePath::new("a/../b"), Err(PathError::Traversal));
        assert_eq!(ScopePath::new("a//b"), Err(PathError::EmptySegment));
        // Two trailing slashes leave one empty segment.
        assert_eq!(ScopePath::new("a//"), Err(PathError::EmptySegment));
        for bad in [
            "a/b.", "a/b ", "a/b\u{7}", "a<b", "a>b", "a:b", "a\"b", "a|b", "a?b", "a*b",
        ] {
            assert_eq!(
                ScopePath::new(bad),
                Err(PathError::InvalidSegment {
                    segment: bad.rsplit('/').next().unwrap().to_owned()
                }),
                "{bad:?}"
            );
        }
        // The offending segment is reported, not the last one.
        assert_eq!(
            ScopePath::new("a/./b"),
            Err(PathError::InvalidSegment {
                segment: ".".to_owned()
            })
        );
    }

    #[test]
    fn name_parent_and_segments() {
        let p = path("Kits/p1/kit.dds");
        assert_eq!(p.name(), "kit.dds");
        assert_eq!(p.parent().unwrap().as_str(), "Kits/p1");
        assert_eq!(p.parent().unwrap().parent().unwrap().as_str(), "Kits");
        assert!(p.parent().unwrap().parent().unwrap().parent().is_none());
        assert_eq!(p.segments().collect::<Vec<_>>(), ["Kits", "p1", "kit.dds"]);
    }

    #[test]
    fn relative_validates_like_scope_path() {
        let root = path("player1");
        assert_eq!(root.relative(""), Err(PathError::Empty));
        assert_eq!(root.relative("/a"), Err(PathError::Absolute));
        assert_eq!(root.relative("a/../b"), Err(PathError::Traversal));
        assert_eq!(root.relative("a//b"), Err(PathError::EmptySegment));
        let rel = root.relative(r"textures\kit.dds").unwrap();
        assert_eq!(rel.as_str(), "textures/kit.dds");
        assert_eq!(rel.root(), &root);
        assert_eq!(rel.segments().collect::<Vec<_>>(), ["textures", "kit.dds"]);
    }

    #[test]
    fn to_scope_path_joins_back_to_its_root() {
        let root = path("player1");
        let rel = root.relative("face.fmdl").unwrap();
        assert_eq!(rel.to_scope_path().as_str(), "player1/face.fmdl");
    }

    #[test]
    fn insert_detects_folder_spelling_collisions() {
        let mut tree = VirtualTree::new();
        tree.insert(path("Kits/a.dds"), ()).unwrap();
        assert_eq!(
            tree.insert(path("kits/b.dds"), ()),
            Err(InsertError::Collision {
                existing: path("Kits/a.dds")
            })
        );
        // Same folder spelling is fine.
        assert_eq!(tree.insert(path("Kits/b.dds"), ()), Ok(()));
        // NFC vs NFD folder spellings collide too.
        let mut accents = VirtualTree::new();
        accents.insert(path("c\u{e9}/a"), ()).unwrap();
        assert_eq!(
            accents.insert(path("ce\u{301}/b"), ()),
            Err(InsertError::Collision {
                existing: path("c\u{e9}/a")
            })
        );
        // The check applies at every depth.
        let mut deep = VirtualTree::new();
        deep.insert(path("a/B/x"), ()).unwrap();
        assert_eq!(
            deep.insert(path("a/b/y"), ()),
            Err(InsertError::Collision {
                existing: path("a/B/x")
            })
        );
    }

    #[test]
    fn fold_key_ignores_case_and_normalization_form() {
        assert_eq!(path("Kit.DDS").fold_key(), path("kit.dds").fold_key());
        // NFC 'é' vs NFD 'e' + combining acute.
        assert_eq!(path("c\u{e9}/f").fold_key(), path("ce\u{301}/f").fold_key());
    }

    #[test]
    fn get_is_case_insensitive_and_keeps_real_casing() {
        let mut tree = VirtualTree::new();
        tree.insert(path("Kits/p1/Kit.DDS"), 7u32).unwrap();
        let found = tree.get(&path("kits/P1/kit.dds")).unwrap();
        assert_eq!(*found, 7);
        let stored = tree.iter().next().unwrap().0;
        assert_eq!(stored.as_str(), "Kits/p1/Kit.DDS");
        assert_eq!(tree.len(), 1);
        assert!(!tree.is_empty());
        *tree.get_mut(&path("KITS/p1/KIT.DDS")).unwrap() = 9;
        assert_eq!(*tree.get(&path("kits/p1/kit.dds")).unwrap(), 9);
    }

    #[test]
    fn insert_refuses_duplicates_collisions_and_conflicts() {
        let mut tree = VirtualTree::new();
        tree.insert(path("a/b.txt"), ()).unwrap();
        assert_eq!(
            tree.insert(path("a/b.txt"), ()),
            Err(InsertError::Duplicate)
        );
        assert_eq!(
            tree.insert(path("A/B.TXT"), ()),
            Err(InsertError::Collision {
                existing: path("a/b.txt")
            })
        );
        // NFD vs NFC spelling of the same name is also a collision.
        let mut accents = VirtualTree::new();
        accents.insert(path("c\u{e9}.txt"), ()).unwrap();
        assert_eq!(
            accents.insert(path("ce\u{301}.txt"), ()),
            Err(InsertError::Collision {
                existing: path("c\u{e9}.txt")
            })
        );
        // File blocks a folder at the same spot, and the reverse.
        assert_eq!(
            tree.insert(path("a/b.txt/c"), ()),
            Err(InsertError::FileFolderConflict {
                existing: path("a/b.txt")
            })
        );
        let mut reverse = VirtualTree::new();
        reverse.insert(path("a/b.txt"), ()).unwrap();
        assert_eq!(
            reverse.insert(path("a"), ()),
            Err(InsertError::FileFolderConflict {
                existing: path("a/b.txt")
            })
        );
    }

    #[test]
    fn children_lists_files_and_implicit_folders_once_sorted() {
        let mut tree = VirtualTree::new();
        for (p, v) in [
            ("a/x.txt", 1u32),
            ("a/sub/deep.txt", 2),
            ("b/y.txt", 3),
            ("root.txt", 4),
        ] {
            tree.insert(path(p), v).unwrap();
        }
        // Sorted by fold key: a, b, root.txt.
        assert_eq!(
            tree.children(None),
            vec![
                Entry::Folder(path("a")),
                Entry::Folder(path("b")),
                Entry::File(path("root.txt")),
            ]
        );
        let names: Vec<String> = tree
            .children(None)
            .into_iter()
            .map(|e| match e {
                Entry::File(p) | Entry::Folder(p) => p.as_str().to_owned(),
            })
            .collect();
        assert_eq!(names, ["a", "b", "root.txt"]);
        assert_eq!(
            tree.children(Some(&path("A"))),
            vec![Entry::Folder(path("a/sub")), Entry::File(path("a/x.txt"))]
        );
        assert!(tree.children(Some(&path("a/sub"))).len() == 1);
    }

    #[test]
    fn files_under_and_iter_are_recursive_and_sorted() {
        let mut tree = VirtualTree::new();
        for (p, v) in [("a/x.txt", 1u32), ("a/sub/deep.txt", 2), ("b/y.txt", 3)] {
            tree.insert(path(p), v).unwrap();
        }
        let under_a: Vec<&str> = tree
            .files_under(Some(&path("A")))
            .map(|(p, _)| p.as_str())
            .collect();
        assert_eq!(under_a, ["a/sub/deep.txt", "a/x.txt"]);
        let all: Vec<&str> = tree.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(all, ["a/sub/deep.txt", "a/x.txt", "b/y.txt"]);
        assert_eq!(tree.files_under(None).count(), 3);
    }

    #[test]
    fn remove_and_contains_folder() {
        let mut tree = VirtualTree::new();
        tree.insert(path("a/b.txt"), 1u32).unwrap();
        assert!(tree.contains_folder(&path("A")));
        assert!(!tree.contains_folder(&path("a/b.txt")));
        assert!(tree.contains_file(&path("A/B.TXT")));
        assert_eq!(tree.remove(&path("A/B.TXT")), Some(1));
        assert!(!tree.contains_folder(&path("a")));
        assert_eq!(tree.remove(&path("a/b.txt")), None);
        assert!(tree.is_empty());
    }

    #[test]
    fn fold_key_uses_ntfs_style_simple_case_folding() {
        // NTFS uppercases per character to compare names, so final sigma and
        // medial sigma are the same file name.
        assert_eq!(
            path("\u{3c2}.dds").fold_key(),
            path("\u{3c3}.dds").fold_key()
        );
        let mut tree = VirtualTree::new();
        tree.insert(path("\u{3c2}.dds"), ()).unwrap();
        assert_eq!(
            tree.insert(path("\u{3c3}.dds"), ()),
            Err(InsertError::Collision {
                existing: path("\u{3c2}.dds")
            })
        );
        // 'ß' uppercases to multi-char "SS", which NTFS does not apply, so
        // 'ß' and 'ss' stay distinct file names.
        assert_ne!(path("\u{df}.dds").fold_key(), path("ss.dds").fold_key());
        let mut sharp = VirtualTree::new();
        sharp.insert(path("\u{df}.dds"), ()).unwrap();
        assert_eq!(sharp.insert(path("ss.dds"), ()), Ok(()));
    }

    #[test]
    fn prefix_walks_stay_bounded_on_a_larger_tree() {
        let mut tree = VirtualTree::new();
        for folder in 0..50 {
            for file in 0..40 {
                tree.insert(path(&format!("f{folder:02}/p{file:02}.txt")), file)
                    .unwrap();
            }
        }
        assert_eq!(tree.len(), 2000);
        assert_eq!(tree.children(None).len(), 50);
        assert_eq!(tree.files_under(Some(&path("F10"))).count(), 40);
        assert_eq!(tree.children(Some(&path("f10"))).len(), 40);
    }
}
