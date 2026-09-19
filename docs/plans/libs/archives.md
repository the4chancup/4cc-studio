# 4cc Studio — Library crates plan: archives

Part of the [Library crates plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## `libs/archives`: reading `.zip` and `.7z` exports

Members submit exports as folders, `.zip` or `.7z`; the Team compiler reads them in place
(Team compiler plan, "Load": structure eagerly, contents lazily per folder, the export never
written to) and the Team creator and Export upgrader extract them. `archives` is the one place
that knows the two containers, over any `Read + Seek` source so the crate stays `wasm32`-clean,
with the disk-path opener behind `cfg(not(target_arch = "wasm32"))`:

```rust
pub struct Entry { pub path: String, pub size: u64 }   // forward slashes, no leading `/`, files only
pub struct Archive<R: Read + Seek> { /* zip or 7z state */ }
impl<R: Read + Seek> Archive<R> {
    pub fn zip(reader: R) -> Result<Self, ArchiveError>;      // reads the central directory only
    pub fn seven_z(reader: R) -> Result<Self, ArchiveError>;  // reads the header only
    pub fn entries(&self) -> &[Entry];
    pub fn read(&mut self, path: &str) -> Result<Vec<u8>, ArchiveError>;
}
#[cfg(not(target_arch = "wasm32"))]
impl Archive<std::fs::File> {
    pub fn open(path: &Path) -> Result<Self, ArchiveError>;   // by extension, case-insensitive
}
pub enum ArchiveError {
    Io(std::io::Error), Zip(String), SevenZ(String),          // the container crate's own message
    UnsupportedExtension(String), NotFound(String), InvalidName(String), Encrypted,
}
```

`read` on a zip inflates that one entry (random access, which is what makes per-folder lazy
loading possible); on a 7z the first `read` decompresses the whole archive into memory and later
reads are lookups, because 7-Zip's default is one solid LZMA2 block and per-entry extraction of a
solid archive is quadratic. The Team compiler's memory budget charges a 7z export by the sum of
its `entries()` sizes for that reason (its plan already says so); dropping the `Archive` releases
the buffer. Names are normalized (`\` to `/`, a leading `/` or `./` stripped), directory entries
dropped, and a name with a `..` segment or a drive prefix is `InvalidName` (a zip-slip name has no
meaning in a tree that is never written to disk, and rejecting it keeps the guarantee visible).
Non-ASCII names: a zip stores names in UTF-8 when its flag bit 11 is set (PowerShell's
`Compress-Archive`, most modern tools) and otherwise in the writer's OEM code page, which the
`zip` crate reads as cp437 (7-Zip on a Western Windows writes `é` as `0x82`, which cp437 maps
back correctly; a name outside cp437 from a non-Western Windows will come out wrong, and no fix
exists without knowing the writer's code page). `.7z` names are always UTF-16 and safe.
Encrypted archives are `Encrypted`, never a prompt. Codecs: `zip` with Deflate and Store (the two
every zip writer in use emits; Deflate64, bzip2 and LZMA zips are read errors, reported as such),
`sevenz-rust2` with its built-in LZMA, LZMA2 and BCJ filters (7-Zip's defaults; PPMd and BZip2
archives are read errors). Both crates without default features, so no compression, encryption or
time dependencies ride along.

Fixtures (`crates/libs/archives/tests/fixtures/`): one small export-like tree (`sample/`: two
player folders with real 176-byte, 476-byte and 960-byte PES files, an empty file, an empty folder,
a `Kits/Réf.txt` with a non-ASCII name) packed six ways: 7-Zip `.7z` (LZMA2 solid), 7-Zip
`.zip` (Deflate, names in the OEM code page without the UTF-8 flag), 7-Zip stored `.zip`,
PowerShell `Compress-Archive` (Deflate, UTF-8 flag), and 7-Zip encrypted `.7z` (header encrypted
too) and `.zip`. Every readable archive must list the same six file entries with the same sizes
and yield bytes equal to the tree's files; the encrypted ones must fail with `Encrypted` at
`read` (or at open when the header itself is encrypted).
