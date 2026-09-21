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
    pub fn zip(reader: R) -> Result<Self, ArchiveError>;      // reads the central directory and each entry's local header, decompresses nothing
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
    UnsupportedExtension(String), NotFound(String), InvalidName(String), DuplicateName(String), Encrypted,
}
```

`zip` opens each entry's local header at open (the `zip` crate exposes an entry's size and
encryption flag only through its raw entry view), one seek per entry and no decompression; the
"central directory only" of the first plan text is not available through that crate's public
API, and the cost is one seek per entry against an archive that will be decompressed anyway.

`read` on a zip inflates that one entry (random access, which is what makes per-folder lazy
loading possible); on a 7z the first `read` decompresses the whole archive into memory and later
reads are lookups, because 7-Zip's default is one solid LZMA2 block and per-entry extraction of a
solid archive is quadratic. The Team compiler's memory budget charges a 7z export by the sum of
its `entries()` sizes for that reason (its plan already says so); dropping the `Archive` releases
the buffer. Names are normalized (`\` to `/`, a leading `/` or `./` stripped), directory entries
dropped, and a name with a `..` segment or a drive prefix (a first segment `X:...`, whatever
follows the colon) is `InvalidName` (a zip-slip name has no meaning in a tree that is never
written to disk, and rejecting it keeps the guarantee visible). Two entries that normalize to one
tree path (`a/b` and `a\b`, `./a/b`) are `DuplicateName`, the plan-wide rule that collisions are
rejected, never silently resolved; two entries with the *same* raw name in a zip are collapsed by
the `zip` crate's own name index before this crate sees them (an accepted limitation: no tool in
use writes such a zip).
Non-ASCII names: a zip stores names in UTF-8 when its flag bit 11 is set (PowerShell's
`Compress-Archive`, most modern tools) and otherwise in the writer's OEM code page, which the
`zip` crate reads as cp437 (7-Zip on a Western Windows writes `é` as `0x82`, which cp437 maps
back correctly; a name outside cp437 from a non-Western Windows will come out wrong, and no fix
exists without knowing the writer's code page). `.7z` names are always UTF-16 and safe.
Encrypted archives are `Encrypted`, never a prompt: with `sevenz-rust2` built without its AES
feature and always given the empty password, encryption of the header or of the entries surfaces
as the AES coder being an unsupported method, which is what the crate maps to `Encrypted` (the
crate's own password errors are unreachable in that configuration and are not matched). Codecs:
`zip` with Deflate and Store (the two every zip writer in use emits; Deflate64, bzip2 and LZMA
zips list and fail at `read` with the zip crate's message), `sevenz-rust2` with its built-in
LZMA, LZMA2 and BCJ filters (7-Zip's defaults; a PPMd or BZip2 7z lists and fails at the first
`read`, since the codec is only met when a block is decoded). Both crates without default
features, so no compression, encryption or time dependencies ride along.

Fixtures (`crates/libs/archives/tests/fixtures/`): one small export-like tree (`sample/`: two
player folders with real 176-byte, 476-byte and 960-byte PES files, an empty file, an empty folder,
a `Kits/Réf.txt` with a non-ASCII name) packed six ways: 7-Zip `.7z` (LZMA2 solid), 7-Zip
`.zip` (Deflate, names in the OEM code page without the UTF-8 flag), 7-Zip stored `.zip`,
PowerShell `Compress-Archive` (Deflate, UTF-8 flag), 7-Zip encrypted `.7z` (header encrypted
too), `.7z` with the header readable and the entries encrypted, and `.zip`; plus a PPMd `.7z` and
a BZip2 `.zip` for the codecs the crate does not carry. Every readable archive must list the same
six file entries with the same sizes and yield bytes equal to the tree's files; the encrypted
ones must fail with `Encrypted` at `read` (or at open when the header itself is encrypted); the
PPMd and BZip2 ones list the six entries and fail at `read` with the container crate's message,
their stored entries still readable.
