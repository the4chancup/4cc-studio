//! A member's own `face.xml` in a pre-Fox face folder, the authority on what the face loads in
//! place of the one the compiler generates (`team_compiler/messages.md` "User-supplied
//! `face.xml`"): read (`parse`), each `path` and `material` resolved to the file it names
//! (`reference`, `resolve`), and checked (`check`) against the "XML/MTL content checks"
//! table, errors for what is known not to work and warnings for what the compiler cannot vouch
//! for. What it does not know is kept as the member wrote it (unknown elements and attributes),
//! for the face task to write back. The deep pass checks with it.

use std::fmt;

use aesthetics_export::{
    Disposition, FileDescriptor, FileKind, KitToken, ModelFormat, classify, common_link_name,
    kit_token,
};
use pes_version::PesVersion;
use vtree::ScopePath;

use crate::deep::relative;
use crate::face_diff::{self, FaceDiffError};
use crate::face_xml::is_generated_type;
use crate::messages::Code;
use crate::plan::roles::{common_file, file_stem, in_folder_or_face, is_direct_root_folder_file};

/// A finding `check` makes on the folder holding the xml: the code, what is done about it
/// (`DropFolder` for an Error, `Keep` for a Warning or an Info) and its context.
pub(crate) type XmlFinding = (Code, Disposition, Vec<(&'static str, String)>);

/// A member's own `face.xml`: the children of its `<config>` root, in source order. Comments
/// and processing instructions are not kept: the compiler re-serializes the file it emits.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct UserFaceXml {
    /// `<config>`'s child elements, in source order.
    pub(crate) children: Vec<Child>,
}

/// One child element of a `face.xml`'s `<config>`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Child {
    /// A `<model>`: one model the face loads.
    Model(ModelElement),
    /// A `<dif>`: the face diff it holds, decoded and checked as a `face_diff.xml`'s is.
    Dif(Vec<u8>),
    /// Any other element, kept verbatim (`xml_element_unknown`).
    Other(Element),
}

/// A `<model>` element: its attributes in source order, names and values as written.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ModelElement {
    /// The attributes (name, value), in source order.
    pub(crate) attributes: Vec<(String, String)>,
}

impl ModelElement {
    /// The value of the attribute `name`, when the element has it.
    pub(crate) fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(attribute, _)| attribute == name)
            .map(|(_, value)| value.as_str())
    }
}

/// An element the compiler does not know, as written: its name, attributes in source order,
/// text (trimmed) and child elements.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Element {
    /// The element's name.
    pub(crate) name: String,
    /// Its attributes (name, value), in source order.
    pub(crate) attributes: Vec<(String, String)>,
    /// Its text, the text nodes among its children joined and trimmed.
    pub(crate) text: String,
    /// Its child elements, in source order.
    pub(crate) children: Vec<Element>,
}

/// What a `path` or `material` value names.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Reference {
    /// `./<name>`: a file of the face, its `*` read as `win32` (`./face_high_*.model` names
    /// `face_high_win32.model`).
    Local(String),
    /// `model/character/uniform/common/<segment>/<name>`: a file directly in the export's
    /// `Common/`, its `*` read as `win32`. The segment stands for the team ID and must be three
    /// characters long (`xml_common_path_invalid`).
    Common {
        /// The folder segment after `common/`, as written.
        segment: String,
        /// The file name, `*` read as `win32`.
        file_name: String,
    },
    /// Any other form, which the compiler cannot resolve (`xml_path_unchecked`): a face Common
    /// path, a `./` path into a subfolder, a bare name. Holds the value as written.
    Unchecked(String),
}

/// The game folder a `Common` reference names its file in, before the team's segment.
const UNIFORM_COMMON: &str = "model/character/uniform/common/";

/// What the `path` or `material` value `value` names (`Reference`).
pub(crate) fn reference(value: &str) -> Reference {
    if let Some(name) = value.strip_prefix("./")
        && !name.contains('/')
    {
        return Reference::Local(name.replace('*', "win32"));
    }
    if let Some(rest) = value.strip_prefix(UNIFORM_COMMON)
        && let Some((segment, name)) = rest.split_once('/')
        && !name.contains('/')
    {
        return Reference::Common {
            segment: segment.to_owned(),
            file_name: name.replace('*', "win32"),
        };
    }
    Reference::Unchecked(value.to_owned())
}

/// Why a `face.xml` cannot be read.
#[derive(Debug)]
pub(crate) enum XmlError {
    /// The file is not UTF-8 text (`xml_broken`).
    Utf8,
    /// The file is not well-formed XML (`xml_broken`), with the `line:column` its text ends
    /// at, where an error the parser places nowhere (a file cut short, an unclosed root, no
    /// root at all) is found.
    Xml {
        /// The parser's error.
        error: roxmltree::Error,
        /// The position just past the text's last character, `line:column` from `1:1`.
        end: String,
    },
    /// The root element, named here, is not `<config>` (`xml_root_tag_invalid`).
    Root(String),
    /// A `<dif>` that is not a face diff the game can read (`face_diff_invalid`).
    Dif(FaceDiffError),
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmlError::Utf8 => write!(f, "the file is not UTF-8 text"),
            // Most of the parser's messages carry their own `line:column`; the three that do
            // not are found at the text's end, where the parser gave up.
            XmlError::Xml { error, end } => match error {
                roxmltree::Error::NoRootNode
                | roxmltree::Error::UnclosedRootNode
                | roxmltree::Error::UnexpectedEndOfStream => {
                    write!(f, "the file is not well-formed XML: {error} at {end}")
                }
                _ => write!(f, "the file is not well-formed XML: {error}"),
            },
            XmlError::Root(name) => write!(f, "its root element is <{name}>, not <config>"),
            XmlError::Dif(error) => write!(f, "its <dif>: {error}"),
        }
    }
}

/// The member's `face.xml` `bytes`: UTF-8 text (a byte order mark skipped), well-formed XML
/// whose root is `<config>`, each child element read as a `Child`, a `<dif>`'s face diff
/// decoded and checked (`face_diff::from_dif`).
pub(crate) fn parse(bytes: &[u8]) -> Result<UserFaceXml, XmlError> {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|_| XmlError::Utf8)?;
    let document = roxmltree::Document::parse(text).map_err(|error| XmlError::Xml {
        error,
        end: end_position(text),
    })?;
    let root = document.root_element();
    let name = root.tag_name().name();
    if name != "config" {
        return Err(XmlError::Root(name.to_owned()));
    }
    let children = root
        .children()
        .filter(|node| node.is_element())
        .map(child)
        .collect::<Result<Vec<Child>, XmlError>>()?;
    Ok(UserFaceXml { children })
}

/// The position just past the last character of `text`, as `line:column` counted from `1:1`
/// in characters, the way the parser counts the positions it reports.
fn end_position(text: &str) -> String {
    let line = text.matches('\n').count() + 1;
    let last_line = text.rsplit_once('\n').map_or(text, |(_, last)| last);
    format!("{line}:{}", last_line.chars().count() + 1)
}

/// The `Child` the element `node` of `<config>` is.
fn child(node: roxmltree::Node) -> Result<Child, XmlError> {
    match node.tag_name().name() {
        "model" => Ok(Child::Model(ModelElement {
            attributes: attributes(node),
        })),
        "dif" => face_diff::from_dif(node)
            .map(Child::Dif)
            .map_err(XmlError::Dif),
        _ => Ok(Child::Other(element(node))),
    }
}

/// The element `node` as written, its child elements with it.
fn element(node: roxmltree::Node) -> Element {
    let text: String = node
        .children()
        .filter(|child| child.is_text())
        .filter_map(|child| child.text())
        .collect();
    Element {
        name: node.tag_name().name().to_owned(),
        attributes: attributes(node),
        text: text.trim().to_owned(),
        children: node
            .children()
            .filter(|child| child.is_element())
            .map(element)
            .collect(),
    }
}

/// The attributes of `node` (name, value), in source order.
fn attributes(node: roxmltree::Node) -> Vec<(String, String)> {
    node.attributes()
        .map(|attribute| (attribute.name().to_owned(), attribute.value().to_owned()))
        .collect()
}

/// The files a face folder's `face.xml` may name.
pub(crate) struct FaceFiles<'a> {
    /// The folder's own files; a `./` reference looks among those directly in it or in its
    /// `face/`.
    pub(crate) own: &'a [FileDescriptor],
    /// The files of the shared face folder the player links, which a `./` reference looks
    /// among after his own; empty when he links none.
    pub(crate) linked_face: &'a [FileDescriptor],
    /// The export's `Common/` files; a Common reference looks among those directly in it.
    pub(crate) common: &'a [FileDescriptor],
    /// The face folder holding the xml.
    pub(crate) folder: &'a ScopePath,
}

/// The file `reference` names among `files`, of `kind` (`FileKind::Model(PesModel)` for a
/// `path`, `FileKind::Mtl` for a `material`), its name compared case-folded: a `Local` one
/// among the folder's own files directly in it or in `face/`, a `.mtl.common` link there
/// counting as a `.mtl` of its linked name and standing for the `Common/` file it names, then
/// among the linked shared face's; a `Common` one directly in `Common/`. A name with a `kitN`
/// token names its set, found when a variant of it is there, the lowest one returned. `None`
/// when none is there, and for an `Unchecked` reference.
pub(crate) fn resolve<'a>(
    reference: &Reference,
    files: &FaceFiles<'a>,
    kind: FileKind,
) -> Option<&'a FileDescriptor> {
    match reference {
        Reference::Local(name) => {
            let own: Vec<(String, &FileDescriptor)> = files
                .own
                .iter()
                .filter(|file| in_folder_or_face(files.folder, file))
                .filter_map(|file| candidate(file, kind, files.common))
                .collect();
            let linked: Vec<(String, &FileDescriptor)> = files
                .linked_face
                .iter()
                .filter(|file| file.kind == kind)
                .map(|file| (file.path.name().to_owned(), file))
                .collect();
            named(&own, name).or_else(|| named(&linked, name))
        }
        Reference::Common { file_name, .. } => {
            let common: Vec<(String, &FileDescriptor)> = files
                .common
                .iter()
                .filter(|file| file.kind == kind && is_direct_root_folder_file(&file.path))
                .map(|file| (file.path.name().to_owned(), file))
                .collect();
            named(&common, file_name)
        }
        Reference::Unchecked(_) => None,
    }
}

/// The name `file`, one of the folder's own, goes by for a reference to a file of `kind`, with
/// the file it stands for: its own name, or for a `.mtl` a `.mtl.common` link's linked name
/// and the `common` file it names (as `mtl_search` counts one). `None` for a file of another
/// kind.
fn candidate<'a>(
    file: &'a FileDescriptor,
    kind: FileKind,
    common: &'a [FileDescriptor],
) -> Option<(String, &'a FileDescriptor)> {
    if file.kind == kind {
        return Some((file.path.name().to_owned(), file));
    }
    if kind != FileKind::Mtl || file.kind != FileKind::CommonLink {
        return None;
    }
    let linked = common_link_name(file.path.name())?;
    if classify(&linked) != FileKind::Mtl {
        return None;
    }
    let target = common_file(common, &linked)?;
    Some((linked, target))
}

/// The file among `candidates` (name, file) that the referenced name `referenced` names: the
/// one of its name, case-folded, else, for a `kitN` reference, the lowest variant of its set.
fn named<'a>(
    candidates: &[(String, &'a FileDescriptor)],
    referenced: &str,
) -> Option<&'a FileDescriptor> {
    let key = vtree::fold_name(referenced);
    if let Some((_, file)) = candidates
        .iter()
        .find(|(name, _)| vtree::fold_name(name) == key)
    {
        return Some(file);
    }
    candidates
        .iter()
        .filter_map(|(name, file)| variant_of(referenced, name).map(|kit| (kit, *file)))
        .min_by_key(|(kit, _)| *kit)
        .map(|(_, file)| file)
}

/// The kit number of the file named `name` when it is a variant of the set the `kitN` name
/// `referenced` names (`pants_kit2.model` of `pants_kitN.model`), the stems compared
/// case-folded; `None` otherwise, and when `referenced` is no kit reference.
pub(crate) fn variant_of(referenced: &str, name: &str) -> Option<u8> {
    let Some((KitToken::Reference, set)) = kit_token(file_stem(referenced)) else {
        return None;
    };
    let Some((KitToken::Variant(kit), held_set)) = kit_token(file_stem(name)) else {
        return None;
    };
    (vtree::fold_name(&held_set) == vtree::fold_name(&set)).then_some(kit)
}

/// The `<model>` attributes the compiler knows.
const KNOWN_ATTRIBUTES: [&str; 5] = ["level", "type", "path", "material", "ratio"];

/// The prefixes PES 2016 needs a model's file name to start with to load it.
const PES16_PREFIXES: [&str; 3] = ["face_high_", "hair_high_", "oral_"];

/// The findings of the member's `face.xml` `xml`, named `name` below its folder, among the
/// files `files` it may name, for the target `version` (`messages.md` "XML/MTL content
/// checks"), in document order, each `<model>` counted from 1 as `entry`:
///
/// - Errors, dropping the folder: a `<model>` without `type` (`xml_model_type_missing`) or
///   without `path` (`xml_model_path_missing`, and no other finding on its path); a Common
///   reference whose segment is not three characters (`xml_common_path_invalid`); a `./` or
///   Common reference naming no file `resolve` finds (`xml_model_not_found`); on PES 2016 a
///   model file name starting with none of `face_high_`, `hair_high_`, `oral_`, in any case
///   (`xml_oral_prefix_missing`); a `<dif>` beside a `face_diff.xml` (`xml_dif_conflict`; a
///   `face_diff.bin` beside it is the dual-engine layout, no finding).
/// - Warnings, kept: an unknown element (`xml_element_unknown`) or `<model>` attribute
///   (`xml_attribute_unknown`), a type the compiler does not generate (`xml_type_unknown`), a
///   `ratio` that is no finite number (`xml_ratio_invalid`), a reference the compiler cannot
///   resolve (`xml_path_unchecked`), more than one `face_neck` entry
///   (`xml_face_neck_multiple`, once, after the entries), and each `.model` of the folder's
///   own, directly in it or in `face/`, that no `path` names (`xml_model_unlisted`, last, in
///   the files' order; a `kitN` reference names every variant of its set).
/// - Info: a `level` other than `0` (`xml_level_lod`).
///
/// An entry without `material` is not a finding: it is written without one. An empty `type`
/// or `path` is a missing one. No bytes are read: the file lists are enough.
pub(crate) fn check(
    xml: &UserFaceXml,
    name: &str,
    files: &FaceFiles,
    version: PesVersion,
) -> Vec<XmlFinding> {
    let mut findings = Vec::new();
    let mut entry = 0;
    let mut face_necks = 0;
    for child in &xml.children {
        match child {
            Child::Model(model) => {
                entry += 1;
                if model.attribute("type") == Some("face_neck") {
                    face_necks += 1;
                }
                findings.extend(model_findings(model, entry, files, version));
            }
            Child::Dif(_) if holds_face_diff_xml(files) => {
                findings.push(error(Code::XmlDifConflict, vec![("file", name.to_owned())]))
            }
            Child::Dif(_) => {}
            Child::Other(element) => findings.push(kept(
                Code::XmlElementUnknown,
                vec![("element", element.name.clone())],
            )),
        }
    }
    if face_necks > 1 {
        findings.push(kept(
            Code::XmlFaceNeckMultiple,
            vec![("count", face_necks.to_string())],
        ));
    }
    let paths: Vec<&str> = xml
        .children
        .iter()
        .filter_map(|child| match child {
            Child::Model(model) => model.attribute("path"),
            Child::Dif(_) | Child::Other(_) => None,
        })
        .collect();
    for file in files.own {
        let unlisted = file.kind == FileKind::Model(ModelFormat::PesModel)
            && in_folder_or_face(files.folder, file)
            && !paths.iter().any(|path| names_file(path, file.path.name()));
        if unlisted {
            findings.push(kept(
                Code::XmlModelUnlisted,
                vec![("file", relative(&file.path, files.folder))],
            ));
        }
    }
    findings
}

/// The findings of the `<model>` `model`, the `entry`th, for `check`.
fn model_findings(
    model: &ModelElement,
    entry: usize,
    files: &FaceFiles,
    version: PesVersion,
) -> Vec<XmlFinding> {
    let mut findings = Vec::new();
    let present = |name: &str| model.attribute(name).filter(|value| !value.is_empty());
    let xml_type = present("type");
    let path = present("path");
    if xml_type.is_none() {
        let mut context = vec![("entry", entry.to_string())];
        context.extend(path.map(|path| ("path", path.to_owned())));
        findings.push(error(Code::XmlModelTypeMissing, context));
    }
    if path.is_none() {
        let mut context = vec![("entry", entry.to_string())];
        context.extend(xml_type.map(|xml_type| ("type", xml_type.to_owned())));
        findings.push(error(Code::XmlModelPathMissing, context));
    }
    for (attribute, _) in &model.attributes {
        if !KNOWN_ATTRIBUTES.contains(&attribute.as_str()) {
            findings.push(kept(
                Code::XmlAttributeUnknown,
                vec![
                    ("entry", entry.to_string()),
                    ("attribute", attribute.clone()),
                ],
            ));
        }
    }
    if let Some(xml_type) = xml_type
        && !is_generated_type(xml_type)
    {
        findings.push(kept(
            Code::XmlTypeUnknown,
            vec![("type", xml_type.to_owned())],
        ));
    }
    if let Some(level) = model.attribute("level")
        && level != "0"
    {
        findings.push(kept(Code::XmlLevelLod, vec![("level", level.to_owned())]));
    }
    if let Some(ratio) = model.attribute("ratio")
        && !ratio.trim().parse::<f64>().is_ok_and(f64::is_finite)
    {
        findings.push(kept(
            Code::XmlRatioInvalid,
            vec![("ratio", ratio.to_owned())],
        ));
    }
    if let Some(path) = path {
        if version == PesVersion::Pes16 && !has_pes16_prefix(path) {
            findings.push(error(
                Code::XmlOralPrefixMissing,
                vec![("path", path.to_owned())],
            ));
        }
        findings.extend(reference_finding(
            "path",
            path,
            files,
            FileKind::Model(ModelFormat::PesModel),
        ));
    }
    if let Some(material) = present("material") {
        findings.extend(reference_finding(
            "material",
            material,
            files,
            FileKind::Mtl,
        ));
    }
    findings
}

/// The finding on the `attribute` (`path` or `material`) of value `value`, naming a file of
/// `kind`: `xml_path_unchecked` for a form the compiler cannot resolve,
/// `xml_common_path_invalid` for a Common reference whose segment is not three characters,
/// `xml_model_not_found` when the file it names is not among `files`.
fn reference_finding(
    attribute: &'static str,
    value: &str,
    files: &FaceFiles,
    kind: FileKind,
) -> Option<XmlFinding> {
    let context = vec![
        ("attribute", attribute.to_owned()),
        ("value", value.to_owned()),
    ];
    let reference = reference(value);
    match &reference {
        Reference::Unchecked(_) => return Some(kept(Code::XmlPathUnchecked, context)),
        Reference::Common { segment, .. } if segment.chars().count() != 3 => {
            return Some(error(Code::XmlCommonPathInvalid, context));
        }
        Reference::Local(_) | Reference::Common { .. } => {}
    }
    resolve(&reference, files, kind)
        .is_none()
        .then(|| error(Code::XmlModelNotFound, context))
}

/// Whether the model file name the `path` value `path` ends with (after its last `/`, the
/// whole value when it has none) starts, in any case, with a prefix PES 2016 needs.
fn has_pes16_prefix(path: &str) -> bool {
    let name = path.rsplit_once('/').map_or(path, |(_, name)| name);
    let name = name.to_ascii_lowercase();
    PES16_PREFIXES.iter().any(|prefix| name.starts_with(prefix))
}

/// Whether the `path` or `material` value `path` names the file `file_name` of the folder: a
/// `./` reference to its name, case-folded, or to the `kitN` set it is a variant of.
pub(crate) fn names_file(path: &str, file_name: &str) -> bool {
    let Reference::Local(referenced) = reference(path) else {
        return false;
    };
    vtree::fold_name(&referenced) == vtree::fold_name(file_name)
        || variant_of(&referenced, file_name).is_some()
}

/// Whether the folder holds a `face_diff.xml`, directly in it or in `face/`: a second source
/// beside a `<dif>`. A `face_diff.bin` is none: beside a `<dif>` it is the dual-engine layout,
/// the `<dif>` going out on PES 15-17 and the bin on PES 18-21.
fn holds_face_diff_xml(files: &FaceFiles) -> bool {
    files.own.iter().any(|file| {
        file.path.name().eq_ignore_ascii_case("face_diff.xml")
            && in_folder_or_face(files.folder, file)
    })
}

/// An Error finding with `context`: the folder is dropped.
fn error(code: Code, context: Vec<(&'static str, String)>) -> XmlFinding {
    (code, Disposition::DropFolder, context)
}

/// A Warning or Info finding with `context`: the folder is kept.
fn kept(code: Code, context: Vec<(&'static str, String)>) -> XmlFinding {
    (code, Disposition::Keep, context)
}

#[cfg(test)]
mod tests {
    use base64::Engine;

    use super::*;

    /// The one real member `face.xml` in the fixtures, the pre-Fox tracer's Fumos face.
    const FUMOS_XML: &[u8] = include_bytes!(
        "../tests/fixtures/tracer_prefox/old/jp Tracer/Faces/XXX20 - Fumos/face.xml"
    );

    /// The files beside it, as its folder lists them.
    const FUMOS_FILES: [&str; 20] = [
        "boots.mtl",
        "eye_occlusion.dds",
        "face.dds",
        "face.mtl",
        "face.xml",
        "face_high_win32.model",
        "face_normal.dds",
        "face_normal_detail.dds",
        "face_specular_roughness.dds",
        "glove_c.dds",
        "glove_l.mtl",
        "glove_n.dds",
        "glove_r.mtl",
        "glove_sr.dds",
        "k2012_c.dds",
        "k2012_n.dds",
        "k2012_sr.dds",
        "oral_boots_win32.model",
        "oral_glove_l_win32.model",
        "oral_glove_r_win32.model",
    ];

    /// The folder the tests' files sit in.
    const FOLDER: &str = "Players/05 - A";

    /// The descriptor of the export file at `path`, classified by its name.
    fn descriptor(path: &str) -> FileDescriptor {
        let path = ScopePath::new(path).unwrap();
        FileDescriptor {
            size: 1,
            kind: classify(path.name()),
            source: path.clone(),
            path,
        }
    }

    /// The descriptors of `names`, each below `folder`.
    fn files_in(folder: &str, names: &[&str]) -> Vec<FileDescriptor> {
        names
            .iter()
            .map(|name| descriptor(&format!("{folder}/{name}")))
            .collect()
    }

    /// The export's `Common/` files the tests name.
    fn common() -> Vec<FileDescriptor> {
        files_in("Common", &["legs.model", "body.mtl"])
    }

    /// Each finding as one line: code, disposition, context.
    fn lines(findings: Vec<XmlFinding>) -> Vec<String> {
        findings
            .into_iter()
            .map(|(code, disposition, context)| {
                let context: Vec<String> = context
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect();
                format!(
                    "{} [{disposition:?}] ({})",
                    code.as_str(),
                    context.join(", ")
                )
            })
            .collect()
    }

    /// `check`'s findings, as `lines`, on the xml `text` in `FOLDER` holding `own` and linking
    /// the shared face `Faces/Round` holding `hair_high.model`, for `version`.
    fn checked(text: &str, own: &[&str], version: PesVersion) -> Vec<String> {
        let own = files_in(FOLDER, own);
        let linked_face = files_in("Faces/Round", &["hair_high.model"]);
        let common = common();
        let folder = ScopePath::new(FOLDER).unwrap();
        let files = FaceFiles {
            own: &own,
            linked_face: &linked_face,
            common: &common,
            folder: &folder,
        };
        lines(check(
            &parse(text.as_bytes()).unwrap(),
            "face.xml",
            &files,
            version,
        ))
    }

    /// The face model's entry, which every hand-written document holds first, its files
    /// `face_high.model` and `face_high.mtl` beside it: it gives no finding.
    const FACE: &str = r#"<model level="0" type="face_neck" path="./face_high.model" material="./face_high.mtl"/>"#;

    /// The face's own files, which `FACE` lists.
    const FACE_FILES: [&str; 2] = ["face_high.model", "face_high.mtl"];

    /// `checked` on a `<config>` holding `FACE` then `rest`, for PES 17, the folder holding
    /// `FACE_FILES` and `own`.
    fn with_face(rest: &str, own: &[&str]) -> Vec<String> {
        let own: Vec<&str> = FACE_FILES
            .iter()
            .copied()
            .chain(own.iter().copied())
            .collect();
        checked(
            &format!("<config>{FACE}{rest}</config>"),
            &own,
            PesVersion::Pes17,
        )
    }

    #[test]
    fn the_fumos_xml_reads_as_four_models_and_its_face_diff() {
        let xml = parse(FUMOS_XML).unwrap();
        let paths: Vec<Option<&str>> = xml
            .children
            .iter()
            .filter_map(|child| match child {
                Child::Model(model) => Some(model.attribute("path")),
                Child::Dif(_) | Child::Other(_) => None,
            })
            .collect();
        assert_eq!(
            paths,
            [
                Some("./face_high_*.model"),
                Some("./oral_glove_l_*.model"),
                Some("./oral_glove_r_*.model"),
                Some("./oral_boots_*.model"),
            ]
        );
        let difs: Vec<&Vec<u8>> = xml
            .children
            .iter()
            .filter_map(|child| match child {
                Child::Dif(dif) => Some(dif),
                Child::Model(_) | Child::Other(_) => None,
            })
            .collect();
        let [dif] = difs.as_slice() else {
            panic!("{difs:?}");
        };
        face_diff::check(dif).unwrap();
        assert_eq!(xml.children.len(), 5);
        // The attributes keep their source order.
        let Child::Model(first) = &xml.children[0] else {
            panic!("{:?}", xml.children[0]);
        };
        let names: Vec<&str> = first
            .attributes
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(names, ["level", "type", "path", "material"]);
    }

    #[test]
    fn the_fumos_xml_s_one_finding_is_its_boots_type_on_pes_16_and_17() {
        let own = files_in(FOLDER, &FUMOS_FILES);
        let common = common();
        let folder = ScopePath::new(FOLDER).unwrap();
        let files = FaceFiles {
            own: &own,
            linked_face: &[],
            common: &common,
            folder: &folder,
        };
        let xml = parse(FUMOS_XML).unwrap();
        for version in [PesVersion::Pes16, PesVersion::Pes17] {
            assert_eq!(
                lines(check(&xml, "face.xml", &files, version)),
                ["xml_type_unknown [Keep] (type=boots)"],
                "{version:?}"
            );
        }
    }

    #[test]
    fn a_reference_s_form_decides_what_it_names() {
        assert_eq!(
            reference("./face_high_*.model"),
            Reference::Local("face_high_win32.model".to_owned())
        );
        assert_eq!(
            reference("model/character/uniform/common/XXX/legs_*.model"),
            Reference::Common {
                segment: "XXX".to_owned(),
                file_name: "legs_win32.model".to_owned(),
            }
        );
        for unchecked in [
            "model/character/face/common/x.model",
            "./a/b.model",
            "x.model",
            "model/character/uniform/common/legs.model",
            "model/character/uniform/common/XXX/a/b.model",
        ] {
            assert_eq!(
                reference(unchecked),
                Reference::Unchecked(unchecked.to_owned()),
                "{unchecked}"
            );
        }
    }

    #[test]
    fn a_reference_resolves_in_the_folder_then_the_linked_face_and_a_material_through_a_link() {
        let own = files_in(
            FOLDER,
            &[
                "Face_High.model",
                "face/hat.model",
                "boots/deep.model",
                "pants_kit2.model",
                "pants_kit1.model",
                "body.mtl.common",
            ],
        );
        let linked_face = files_in("Faces/Round", &["hair_high.model", "face_high.model"]);
        let common = common();
        let folder = ScopePath::new(FOLDER).unwrap();
        let files = FaceFiles {
            own: &own,
            linked_face: &linked_face,
            common: &common,
            folder: &folder,
        };
        let model = FileKind::Model(ModelFormat::PesModel);
        let found = |value: &str, kind| {
            resolve(&reference(value), &files, kind).map(|file| file.path.as_str().to_owned())
        };
        let path = |text: &str| Some(text.to_owned());
        // The folder's own first, case-folded, then the linked face's.
        assert_eq!(
            found("./face_high.model", model),
            path("Players/05 - A/Face_High.model")
        );
        assert_eq!(
            found("./hat.model", model),
            path("Players/05 - A/face/hat.model")
        );
        assert_eq!(
            found("./hair_high.model", model),
            path("Faces/Round/hair_high.model")
        );
        // Only the folder and its `face/` are searched.
        assert_eq!(found("./deep.model", model), None);
        // A kit set's reference finds its lowest variant.
        assert_eq!(
            found("./pants_kitN.model", model),
            path("Players/05 - A/pants_kit1.model")
        );
        // A `.mtl.common` link counts as a `.mtl` of its linked name, standing for Common's.
        assert_eq!(found("./body.mtl", FileKind::Mtl), path("Common/body.mtl"));
        assert_eq!(found("./body.mtl", model), None);
        assert_eq!(
            found("model/character/uniform/common/XXX/legs.model", model),
            path("Common/legs.model")
        );
        assert_eq!(found("model/character/face/common/legs.model", model), None);
        // A Common file of another kind is not the one named.
        assert_eq!(
            found("model/character/uniform/common/XXX/body.mtl", model),
            None
        );
    }

    #[test]
    fn each_error_of_a_model_entry_drops_the_folder() {
        let cases: [(&str, &[&str], &[&str]); 5] = [
            (
                r#"<model type="parts"/>"#,
                &[],
                &["xml_model_path_missing [DropFolder] (entry=2, type=parts)"],
            ),
            (
                r#"<model path="./hat.model"/>"#,
                &["hat.model"],
                &["xml_model_type_missing [DropFolder] (entry=2, path=./hat.model)"],
            ),
            (
                r#"<model/>"#,
                &[],
                &[
                    "xml_model_type_missing [DropFolder] (entry=2)",
                    "xml_model_path_missing [DropFolder] (entry=2)",
                ],
            ),
            (
                r#"<model type="parts" path="model/character/uniform/common/XX/legs.model"/>"#,
                &[],
                &[
                    "xml_common_path_invalid [DropFolder] (attribute=path, value=model/character/uniform/common/XX/legs.model)",
                ],
            ),
            (
                r#"<model type="parts" path="./hat.model" material="model/character/uniform/common/XXX/hat.mtl"/>"#,
                &[],
                &[
                    "xml_model_not_found [DropFolder] (attribute=path, value=./hat.model)",
                    "xml_model_not_found [DropFolder] (attribute=material, value=model/character/uniform/common/XXX/hat.mtl)",
                ],
            ),
        ];
        for (entry, own, expected) in cases {
            assert_eq!(with_face(entry, own), expected, "{entry}");
        }
    }

    #[test]
    fn a_common_or_linked_face_reference_that_exists_and_a_missing_material_are_no_finding() {
        assert_eq!(
            with_face(
                r#"<model type="parts" path="model/character/uniform/common/XXX/legs.model" material="model/character/uniform/common/XXX/body.mtl"/><model type="parts" path="./hair_high.model"/>"#,
                &[]
            ),
            Vec::<String>::new()
        );
    }

    #[test]
    fn on_pes_16_a_model_name_without_its_prefix_drops_the_folder_whatever_its_path_form() {
        let text = r#"<config>
            <model type="face_neck" path="./Face_High_x.model"/>
            <model type="parts" path="./oral_hat.model"/>
            <model type="uniform" path="./body_uniform.model"/>
            <model type="parts" path="model/character/uniform/common/XXX/legs.model"/>
            <model type="parts" path="model/character/face/common/hat.model"/>
        </config>"#;
        let own = ["Face_High_x.model", "oral_hat.model", "body_uniform.model"];
        assert_eq!(
            checked(text, &own, PesVersion::Pes16),
            [
                "xml_oral_prefix_missing [DropFolder] (path=./body_uniform.model)",
                "xml_oral_prefix_missing [DropFolder] (path=model/character/uniform/common/XXX/legs.model)",
                "xml_oral_prefix_missing [DropFolder] (path=model/character/face/common/hat.model)",
                "xml_path_unchecked [Keep] (attribute=path, value=model/character/face/common/hat.model)",
            ]
        );
        assert_eq!(
            checked(text, &own, PesVersion::Pes17),
            [
                "xml_path_unchecked [Keep] (attribute=path, value=model/character/face/common/hat.model)"
            ]
        );
    }

    #[test]
    fn a_dif_beside_a_face_diff_xml_is_a_conflict_and_beside_a_bin_none() {
        let dif = base64::engine::general_purpose::STANDARD
            .encode(crate::templates::Templates::embedded().face_diff());
        let rest = format!("<dif>{dif}</dif>");
        assert_eq!(with_face(&rest, &[]), Vec::<String>::new());
        // The dual-engine layout: the `<dif>` for PES 15-17, the bin for PES 18-21.
        assert_eq!(
            with_face(&rest, &["face/face_diff.bin"]),
            Vec::<String>::new()
        );
        for face_diff_xml in ["Face_Diff.xml", "face/face_diff.xml"] {
            assert_eq!(
                with_face(&rest, &[face_diff_xml]),
                ["xml_dif_conflict [DropFolder] (file=face.xml)"],
                "{face_diff_xml}"
            );
        }
        // One in `boots/` is no face diff of the folder's.
        assert_eq!(
            with_face(&rest, &["boots/face_diff.xml"]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn what_the_compiler_cannot_vouch_for_is_kept_and_warned() {
        let cases: [(&str, &[&str]); 10] = [
            (
                r#"<extra a="1">text</extra>"#,
                &["xml_element_unknown [Keep] (element=extra)"],
            ),
            (
                r#"<model type="parts" path="./face_high.model" glow="1"/>"#,
                &["xml_attribute_unknown [Keep] (entry=2, attribute=glow)"],
            ),
            (
                r#"<model type="cape" path="./face_high.model"/>"#,
                &["xml_type_unknown [Keep] (type=cape)"],
            ),
            (
                r#"<model type="uniform_sub" path="./face_high.model"/><model type="gloveL" path="./face_high.model"/>"#,
                &[],
            ),
            (
                r#"<model level="1" type="parts" path="./face_high.model"/>"#,
                &["xml_level_lod [Keep] (level=1)"],
            ),
            (
                r#"<model type="parts" path="./face_high.model" ratio="abc"/><model type="parts" path="./face_high.model" ratio="inf"/>"#,
                &[
                    "xml_ratio_invalid [Keep] (ratio=abc)",
                    "xml_ratio_invalid [Keep] (ratio=inf)",
                ],
            ),
            (
                r#"<model type="parts" path="./face_high.model" ratio=" 1.5 "/>"#,
                &[],
            ),
            (
                r#"<model type="parts" path="./a/b.model" material="model/character/face/common/x.mtl"/>"#,
                &[
                    "xml_path_unchecked [Keep] (attribute=path, value=./a/b.model)",
                    "xml_path_unchecked [Keep] (attribute=material, value=model/character/face/common/x.mtl)",
                ],
            ),
            (
                r#"<model type="face_neck" path="./face_high.model"/>"#,
                &["xml_face_neck_multiple [Keep] (count=2)"],
            ),
            (
                // A comment is dropped, and is no element.
                r#"<!-- a note --><model type="parts" path="./face_high.model"/>"#,
                &[],
            ),
        ];
        for (rest, expected) in cases {
            assert_eq!(with_face(rest, &[]), expected, "{rest}");
        }
    }

    #[test]
    fn a_model_of_the_folder_no_entry_lists_is_unlisted_in_file_order() {
        assert_eq!(
            with_face(
                r#"<model type="parts" path="./Pants_kitN.model"/>"#,
                &[
                    "pants_kit1.model",
                    "face/hat.model",
                    "pants_kit2.model",
                    "boots/x.model",
                    "torso.model",
                ]
            ),
            [
                "xml_model_unlisted [Keep] (file=face/hat.model)",
                "xml_model_unlisted [Keep] (file=torso.model)",
            ]
        );
    }

    #[test]
    fn an_unknown_element_is_kept_as_written() {
        let xml = parse(br#"<config><extra a="1"> text <!-- c --></extra></config>"#).unwrap();
        assert_eq!(
            xml.children,
            [Child::Other(Element {
                name: "extra".to_owned(),
                attributes: vec![("a".to_owned(), "1".to_owned())],
                text: "text".to_owned(),
                children: Vec::new(),
            })]
        );
    }

    #[test]
    fn a_file_that_is_no_config_document_is_refused_with_its_reason() {
        let error = parse(b"<config><dif>RkFD</dif></config>").unwrap_err();
        assert!(matches!(error, XmlError::Dif(_)), "{error:?}");
        assert_eq!(
            error.to_string(),
            "its <dif>: the face diff is 3 bytes long, shorter than its 80-byte header"
        );
        let error = parse(b"<cfg/>").unwrap_err();
        assert!(
            matches!(&error, XmlError::Root(root) if root == "cfg"),
            "{error:?}"
        );
        let error = parse(b"<config><model").unwrap_err();
        assert!(matches!(error, XmlError::Xml { .. }), "{error:?}");
        assert_eq!(
            error.to_string(),
            "the file is not well-formed XML: the root node was opened but never closed at 1:15"
        );
        let error = parse(b"<config><model></config>").unwrap_err();
        assert_eq!(
            error.to_string(),
            "the file is not well-formed XML: expected 'model' tag, not 'config' at 1:16"
        );
        assert!(matches!(parse(b"\xFF\xFE"), Err(XmlError::Utf8)));
        // A byte order mark is skipped.
        assert!(parse(b"\xEF\xBB\xBF<config/>").is_ok());
    }
}
