//! The `.mtl` material set: a small XML file, WESYS-wrapped like the model,
//! listing the materials a `.model` binds by name. Read with `roxmltree`,
//! written by hand in one canonical layout under a per-file [`MtlStyle`], so
//! a regularly formatted Konami file rewrites byte for byte and every file
//! rewrites to the same meaning.

/// A `.mtl`: the material definitions a `.model` binds by name.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialSet {
    /// The materials, in file order.
    pub materials: Vec<Material>,
    /// The whitespace convention the file used; detected on read.
    pub style: MtlStyle,
}

/// The whitespace convention of one file: detected on read, Konami's
/// majority for new files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MtlStyle {
    /// The line ending.
    pub newline: Newline,
    /// One indent level (`"    "` or `"\t"`); children of a material get two.
    pub indent: String,
    /// Whether the file ends with a newline.
    pub final_newline: bool,
}

/// The line ending a `.mtl` uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Newline {
    /// `\n`.
    Lf,
    /// `\r\n`.
    CrLf,
}

impl Default for MtlStyle {
    /// Konami's majority: CRLF, four-space indents, a final newline.
    fn default() -> MtlStyle {
        MtlStyle {
            newline: Newline::CrLf,
            indent: "    ".to_owned(),
            final_newline: true,
        }
    }
}

/// One material: its name, its shader, and its entries in file order.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    /// The name a `.model` binds by.
    pub name: String,
    /// The shader name.
    pub shader: String,
    /// Samplers, states and vectors in file order.
    pub entries: Vec<MaterialEntry>,
}

/// One entry of a material.
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialEntry {
    /// A texture binding.
    Sampler(Sampler),
    /// A render state.
    State(State),
    /// A shader parameter.
    Vector(Vector),
}

/// A texture binding. Optional attributes are written in the fixed order
/// every Konami file uses when present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sampler {
    /// The sampler's name (e.g. `DiffuseMap`).
    pub name: String,
    /// The texture path (`.dds`, `./name.dds` or `model/...`).
    pub path: String,
    /// Whether the texture is sRGB.
    pub srgb: Option<bool>,
    /// The minification filter.
    pub minfilter: Option<Filter>,
    /// The magnification filter.
    pub magfilter: Option<Filter>,
    /// The mip filter.
    pub mipfilter: Option<Filter>,
    /// Address mode in u.
    pub uaddr: Option<Address>,
    /// Address mode in v.
    pub vaddr: Option<Address>,
    /// Address mode in w.
    pub waddr: Option<Address>,
    /// The anisotropy level.
    pub maxaniso: Option<u32>,
}

/// A texture filter mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    /// `linear`.
    Linear,
    /// `point`.
    Point,
    /// `anisotropic`.
    Anisotropic,
}

/// A texture address mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Address {
    /// `wrap`.
    Wrap,
    /// `clamp`.
    Clamp,
    /// `repeat`.
    Repeat,
}

/// A render state: the seven the material schema knows (`ztest`, `zwrite`,
/// `twosided`, `alphatest`, `alpharef`, `alphablend`, `blendmode`) plus
/// whatever else a file carries (`shadowcaster`); validation is `check`'s
/// job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    /// The state's name.
    pub name: String,
    /// Its value.
    pub value: u32,
}

/// A shader parameter: one to four components, written as `x`, `y`, `z`, `w`
/// in order.
#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    /// The parameter's name.
    pub name: String,
    /// Its components.
    pub components: Vec<f32>,
}

/// Why a `.mtl` could not be read.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum MtlError {
    /// The WESYS wrapper is corrupt.
    #[error("corrupt WESYS wrapper: {0}")]
    Wesys(String),
    /// The file is not UTF-8.
    #[error("mtl is not UTF-8")]
    InvalidUtf8,
    /// `roxmltree` rejected the XML.
    #[error("invalid xml: {0}")]
    Xml(String),
    /// An element the grammar does not name.
    #[error("unexpected element <{element}> under <{parent}>")]
    UnexpectedElement {
        /// The element's tag.
        element: String,
        /// Its parent's tag.
        parent: String,
    },
    /// An attribute the grammar does not name.
    #[error("unexpected attribute {attribute} on <{element}>")]
    UnexpectedAttribute {
        /// The element's tag.
        element: String,
        /// The attribute's name.
        attribute: String,
    },
    /// A required attribute is absent.
    #[error("missing attribute {attribute} on <{element}>")]
    MissingAttribute {
        /// The element's tag.
        element: String,
        /// The attribute's name.
        attribute: &'static str,
    },
    /// An attribute value the grammar does not allow.
    #[error("invalid value {value:?} for {attribute} on <{element}>")]
    InvalidValue {
        /// The element's tag.
        element: String,
        /// The attribute's name.
        attribute: &'static str,
        /// The offending text.
        value: String,
    },
    /// Non-whitespace text where none belongs.
    #[error("text content in <{0}>")]
    UnexpectedText(String),
}

impl Filter {
    /// The attribute text (`linear`, `point`, `anisotropic`).
    pub fn as_str(self) -> &'static str {
        match self {
            Filter::Linear => "linear",
            Filter::Point => "point",
            Filter::Anisotropic => "anisotropic",
        }
    }

    /// The filter a text names, or `None`.
    pub fn from_str_opt(text: &str) -> Option<Filter> {
        match text {
            "linear" => Some(Filter::Linear),
            "point" => Some(Filter::Point),
            "anisotropic" => Some(Filter::Anisotropic),
            _ => None,
        }
    }
}

impl Address {
    /// The attribute text (`wrap`, `clamp`, `repeat`).
    pub fn as_str(self) -> &'static str {
        match self {
            Address::Wrap => "wrap",
            Address::Clamp => "clamp",
            Address::Repeat => "repeat",
        }
    }

    /// The address mode a text names, or `None`.
    pub fn from_str_opt(text: &str) -> Option<Address> {
        match text {
            "wrap" => Some(Address::Wrap),
            "clamp" => Some(Address::Clamp),
            "repeat" => Some(Address::Repeat),
            _ => None,
        }
    }
}

/// `attribute` on `element` must parse; `parse` maps the text or fails
/// `InvalidValue`.
fn parsed<T>(
    element: &str,
    attribute: &'static str,
    value: &str,
    parse: impl Fn(&str) -> Option<T>,
) -> Result<T, MtlError> {
    parse(value).ok_or_else(|| MtlError::InvalidValue {
        element: element.to_owned(),
        attribute,
        value: value.to_owned(),
    })
}

/// `attribute` on `element`, required.
fn required<'a>(
    node: roxmltree::Node<'a, 'a>,
    element: &str,
    attribute: &'static str,
) -> Result<&'a str, MtlError> {
    node.attribute(attribute)
        .ok_or_else(|| MtlError::MissingAttribute {
            element: element.to_owned(),
            attribute,
        })
}

/// A non-element child node is fine only as whitespace text; comments and
/// processing instructions are ignored (no Konami file carries them).
fn check_text(node: roxmltree::Node, parent: &str) -> Result<(), MtlError> {
    if node.is_text() && node.text().is_some_and(|text| !text.trim().is_empty()) {
        return Err(MtlError::UnexpectedText(parent.to_owned()));
    }
    Ok(())
}

/// The `<sampler>` element.
fn read_sampler(node: roxmltree::Node) -> Result<Sampler, MtlError> {
    let mut sampler = Sampler {
        name: required(node, "sampler", "name")?.to_owned(),
        path: required(node, "sampler", "path")?.to_owned(),
        srgb: None,
        minfilter: None,
        magfilter: None,
        mipfilter: None,
        uaddr: None,
        vaddr: None,
        waddr: None,
        maxaniso: None,
    };
    for attribute in node.attributes() {
        let name = attribute.name();
        let value = attribute.value();
        match name {
            "name" | "path" => {}
            "srgb" => {
                sampler.srgb = Some(parsed("sampler", "srgb", value, |text| match text {
                    "0" => Some(false),
                    "1" => Some(true),
                    _ => None,
                })?);
            }
            "minfilter" => {
                sampler.minfilter =
                    Some(parsed("sampler", "minfilter", value, Filter::from_str_opt)?);
            }
            "magfilter" => {
                sampler.magfilter =
                    Some(parsed("sampler", "magfilter", value, Filter::from_str_opt)?);
            }
            "mipfilter" => {
                sampler.mipfilter =
                    Some(parsed("sampler", "mipfilter", value, Filter::from_str_opt)?);
            }
            "uaddr" => {
                sampler.uaddr = Some(parsed("sampler", "uaddr", value, Address::from_str_opt)?);
            }
            "vaddr" => {
                sampler.vaddr = Some(parsed("sampler", "vaddr", value, Address::from_str_opt)?);
            }
            "waddr" => {
                sampler.waddr = Some(parsed("sampler", "waddr", value, Address::from_str_opt)?);
            }
            "maxaniso" => {
                sampler.maxaniso = Some(parsed("sampler", "maxaniso", value, |text| {
                    text.parse().ok()
                })?);
            }
            _ => {
                return Err(MtlError::UnexpectedAttribute {
                    element: "sampler".to_owned(),
                    attribute: name.to_owned(),
                });
            }
        }
    }
    Ok(sampler)
}

/// The `<vector>` element: `name`, then the components present among `x`,
/// `y`, `z`, `w` in that order.
fn read_vector(node: roxmltree::Node) -> Result<Vector, MtlError> {
    const AXES: [&str; 4] = ["x", "y", "z", "w"];
    let name = required(node, "vector", "name")?.to_owned();
    let mut components = Vec::new();
    for attribute in node.attributes() {
        let attribute_name = attribute.name();
        if attribute_name == "name" {
            continue;
        }
        let axis = AXES
            .iter()
            .position(|axis| *axis == attribute_name)
            .ok_or_else(|| MtlError::UnexpectedAttribute {
                element: "vector".to_owned(),
                attribute: attribute_name.to_owned(),
            })?;
        // A missing earlier component with a later one present.
        if axis != components.len() {
            return Err(MtlError::InvalidValue {
                element: "vector".to_owned(),
                attribute: AXES[components.len()],
                value: String::new(),
            });
        }
        components.push(parsed("vector", AXES[axis], attribute.value(), |text| {
            text.parse().ok()
        })?);
    }
    Ok(Vector { name, components })
}

/// The `<material>` element.
fn read_material(node: roxmltree::Node) -> Result<Material, MtlError> {
    for attribute in node.attributes() {
        if attribute.name() != "name" && attribute.name() != "shader" {
            return Err(MtlError::UnexpectedAttribute {
                element: "material".to_owned(),
                attribute: attribute.name().to_owned(),
            });
        }
    }
    let mut material = Material {
        name: required(node, "material", "name")?.to_owned(),
        shader: required(node, "material", "shader")?.to_owned(),
        entries: Vec::new(),
    };
    for child in node.children() {
        if !child.is_element() {
            check_text(child, "material")?;
            continue;
        }
        let entry = match child.tag_name().name() {
            "sampler" => MaterialEntry::Sampler(read_sampler(child)?),
            "state" => {
                for attribute in child.attributes() {
                    if attribute.name() != "name" && attribute.name() != "value" {
                        return Err(MtlError::UnexpectedAttribute {
                            element: "state".to_owned(),
                            attribute: attribute.name().to_owned(),
                        });
                    }
                }
                MaterialEntry::State(State {
                    name: required(child, "state", "name")?.to_owned(),
                    value: parsed(
                        "state",
                        "value",
                        required(child, "state", "value")?,
                        |text| text.parse().ok(),
                    )?,
                })
            }
            "vector" => MaterialEntry::Vector(read_vector(child)?),
            _ => {
                return Err(MtlError::UnexpectedElement {
                    element: child.tag_name().name().to_owned(),
                    parent: "material".to_owned(),
                });
            }
        };
        material.entries.push(entry);
    }
    Ok(material)
}

/// The whitespace convention the file used.
fn detect_style(text: &str) -> MtlStyle {
    let newline = if text.contains("\r\n") {
        Newline::CrLf
    } else {
        Newline::Lf
    };
    // The run of spaces/tabs before the first `<material` that follows a
    // newline.
    let mut indent = "    ".to_owned();
    if let Some(at) = text.find("<material ") {
        let bytes = text.as_bytes();
        let mut start = at;
        while start > 0 && (bytes[start - 1] == b' ' || bytes[start - 1] == b'\t') {
            start -= 1;
        }
        if start > 0 && bytes[start - 1] == b'\n' {
            indent = text[start..at].to_owned();
        }
    }
    MtlStyle {
        newline,
        indent,
        final_newline: text.ends_with('\n'),
    }
}

impl MaterialSet {
    /// Parses a `.mtl`, WESYS-wrapped or not. An `<?xml ...?>` declaration
    /// or a BOM, if ever present, are accepted by the parser and not
    /// carried.
    pub fn read(bytes: &[u8]) -> Result<Self, MtlError> {
        let unwrapped = wezlib::decompress_if_wrapped(bytes)
            .map_err(|error| MtlError::Wesys(error.to_string()))?;
        let text = std::str::from_utf8(&unwrapped).map_err(|_| MtlError::InvalidUtf8)?;
        let document =
            roxmltree::Document::parse(text).map_err(|error| MtlError::Xml(error.to_string()))?;
        let root = document.root_element();
        if root.tag_name().name() != "materialset" {
            return Err(MtlError::UnexpectedElement {
                element: root.tag_name().name().to_owned(),
                parent: String::new(),
            });
        }
        if let Some(attribute) = root.attributes().next() {
            return Err(MtlError::UnexpectedAttribute {
                element: "materialset".to_owned(),
                attribute: attribute.name().to_owned(),
            });
        }
        let mut materials = Vec::new();
        for child in root.children() {
            if !child.is_element() {
                check_text(child, "materialset")?;
                continue;
            }
            if child.tag_name().name() != "material" {
                return Err(MtlError::UnexpectedElement {
                    element: child.tag_name().name().to_owned(),
                    parent: "materialset".to_owned(),
                });
            }
            materials.push(read_material(child)?);
        }
        Ok(MaterialSet {
            materials,
            style: detect_style(text),
        })
    }

    /// Serializes under `style`, unwrapped.
    pub fn write(&self) -> Vec<u8> {
        let newline = match self.style.newline {
            Newline::Lf => "\n",
            Newline::CrLf => "\r\n",
        };
        let indent = self.style.indent.as_str();
        let mut out = String::new();
        out.push_str("<materialset>");
        out.push_str(newline);
        for material in &self.materials {
            out.push_str(indent);
            out.push_str("<material");
            attribute(&mut out, "name", &material.name);
            attribute(&mut out, "shader", &material.shader);
            out.push('>');
            out.push_str(newline);
            for entry in &material.entries {
                out.push_str(indent);
                out.push_str(indent);
                match entry {
                    MaterialEntry::Sampler(sampler) => {
                        out.push_str("<sampler");
                        attribute(&mut out, "name", &sampler.name);
                        attribute(&mut out, "path", &sampler.path);
                        if let Some(srgb) = sampler.srgb {
                            attribute(&mut out, "srgb", if srgb { "1" } else { "0" });
                        }
                        for (name, filter) in [
                            ("minfilter", sampler.minfilter),
                            ("magfilter", sampler.magfilter),
                            ("mipfilter", sampler.mipfilter),
                        ] {
                            if let Some(filter) = filter {
                                attribute(&mut out, name, filter.as_str());
                            }
                        }
                        for (name, address) in [
                            ("uaddr", sampler.uaddr),
                            ("vaddr", sampler.vaddr),
                            ("waddr", sampler.waddr),
                        ] {
                            if let Some(address) = address {
                                attribute(&mut out, name, address.as_str());
                            }
                        }
                        if let Some(maxaniso) = sampler.maxaniso {
                            attribute(&mut out, "maxaniso", &maxaniso.to_string());
                        }
                        out.push_str(" />");
                    }
                    MaterialEntry::State(state) => {
                        out.push_str("<state");
                        attribute(&mut out, "name", &state.name);
                        attribute(&mut out, "value", &state.value.to_string());
                        out.push_str(" />");
                    }
                    MaterialEntry::Vector(vector) => {
                        out.push_str("<vector");
                        attribute(&mut out, "name", &vector.name);
                        for (axis, component) in ["x", "y", "z", "w"].iter().zip(&vector.components)
                        {
                            attribute(&mut out, axis, &component.to_string());
                        }
                        out.push_str(" />");
                    }
                }
                out.push_str(newline);
            }
            out.push_str(indent);
            out.push_str("</material>");
            out.push_str(newline);
        }
        out.push_str("</materialset>");
        if self.style.final_newline {
            out.push_str(newline);
        }
        out.into_bytes()
    }
}

/// One attribute, ` name="value"` with the value escaped.
fn attribute(out: &mut String, name: &str, value: &str) {
    out.push(' ');
    out.push_str(name);
    out.push_str("=\"");
    escaped(value, out);
    out.push('"');
}

/// `text` with XML's four attribute escapes applied.
fn escaped(text: &str, out: &mut String) {
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(character),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::fixtures::*;

    fn read(bytes: &[u8]) -> MaterialSet {
        MaterialSet::read(bytes).expect("fixture parses")
    }

    #[test]
    fn regular_fixtures_write_byte_for_byte() {
        for bytes in [CARDHEAD_MTL, HEAD_HI_MTL, HAIR_MTL, SHADOW_MTL] {
            let written = read(bytes).write();
            assert_eq!(
                String::from_utf8_lossy(&written),
                String::from_utf8_lossy(bytes)
            );
        }
    }

    #[test]
    fn every_fixture_round_trips_semantically() {
        for bytes in ALL_MTL {
            let set = read(bytes);
            assert_eq!(read(&set.write()), set);
        }
    }

    #[test]
    fn card_red_is_semantically_identical_but_reformatted() {
        // The file carries trailing spaces on some lines.
        let set = read(CARD_RED_MTL);
        assert_ne!(set.write(), CARD_RED_MTL);
        assert_eq!(read(&set.write()), set);
    }

    #[test]
    fn style_is_detected_per_file() {
        assert_eq!(
            read(CARDHEAD_MTL).style,
            MtlStyle {
                newline: Newline::Lf,
                indent: "  ".to_owned(),
                final_newline: false,
            }
        );
        assert_eq!(
            read(HEAD_HI_MTL).style,
            MtlStyle {
                newline: Newline::Lf,
                indent: "\t".to_owned(),
                final_newline: true,
            }
        );
        assert_eq!(
            read(HAIR_MTL).style,
            MtlStyle {
                newline: Newline::CrLf,
                indent: "    ".to_owned(),
                final_newline: true,
            }
        );
        assert_eq!(
            read(SHADOW_MTL).style,
            MtlStyle {
                newline: Newline::CrLf,
                indent: "\t".to_owned(),
                final_newline: true,
            }
        );
    }

    #[test]
    fn accessory_parses() {
        let set = read(ACCESSORY_MTL);
        assert_eq!(set.materials.len(), 16);
        let material = &set.materials[2];
        assert_eq!(material.name, "glasses_01T");
        assert_eq!(material.shader, "Basic_C");
        assert_eq!(material.entries.len(), 8);
        let expected_states = [
            ("ztest", 1),
            ("zwrite", 1),
            ("twosided", 1),
            ("alphatest", 1),
            ("alpharef", 1),
            ("alphablend", 1),
            ("blendmode", 0),
        ];
        for (entry, (name, value)) in material.entries[..7].iter().zip(expected_states) {
            assert_eq!(
                entry,
                &MaterialEntry::State(State {
                    name: name.to_owned(),
                    value,
                })
            );
        }
        assert_eq!(
            material.entries[7],
            MaterialEntry::Sampler(Sampler {
                name: "DiffuseMap".to_owned(),
                path: "./Glasses01.dds".to_owned(),
                srgb: Some(true),
                minfilter: Some(Filter::Linear),
                magfilter: Some(Filter::Linear),
                mipfilter: None,
                uaddr: None,
                vaddr: None,
                waddr: None,
                maxaniso: None,
            })
        );
    }

    #[test]
    fn hair_parses() {
        let set = read(HAIR_MTL);
        let material = &set.materials[1];
        assert_eq!(material.name, "head_phong");
        assert_eq!(material.shader, "Wrinkle");
        match &material.entries[0] {
            MaterialEntry::Sampler(sampler) => {
                assert_eq!(sampler.mipfilter, Some(Filter::Linear));
                assert_eq!(sampler.uaddr, Some(Address::Clamp));
                assert_eq!(sampler.vaddr, Some(Address::Clamp));
                assert_eq!(sampler.waddr, Some(Address::Wrap));
                assert_eq!(sampler.maxaniso, Some(2));
                assert_eq!(
                    sampler.path,
                    "model/character/face/common/head_normal_default.dds"
                );
            }
            entry => panic!("expected a sampler, got {entry:?}"),
        }
    }

    #[test]
    fn cap_parses() {
        let set = read(CAP_MTL);
        assert_eq!(set.materials.len(), 1);
        let vectors: Vec<&Vector> = set.materials[0]
            .entries
            .iter()
            .filter_map(|entry| match entry {
                MaterialEntry::Vector(vector) => Some(vector),
                _ => None,
            })
            .collect();
        assert_eq!(
            vectors
                .iter()
                .map(|vector| vector.name.as_str())
                .collect::<Vec<_>>(),
            ["SpecularColor", "Shininess"]
        );
        for vector in vectors {
            assert_eq!(vector.components, [0.0, 0.0, 0.0, 0.0]);
        }
    }

    #[test]
    fn card_red_sampler_path() {
        let set = read(CARD_RED_MTL);
        match &set.materials[0].entries[0] {
            MaterialEntry::Sampler(sampler) => {
                assert_eq!(
                    sampler.path,
                    "model/character/parts/referee/card_red_bsm.dds"
                );
            }
            entry => panic!("expected a sampler, got {entry:?}"),
        }
    }

    #[test]
    fn shadow_parses() {
        let set = read(SHADOW_MTL);
        assert_eq!(set.materials.len(), 1);
        let material = &set.materials[0];
        assert_eq!(material.name, "lambert2");
        assert_eq!(material.shader, "Default");
        assert!(material.entries.is_empty());
    }

    #[test]
    fn attributes_escape_on_write() {
        let set = MaterialSet {
            materials: vec![Material {
                name: "a&b \"c\" <d>".to_owned(),
                shader: "s".to_owned(),
                entries: vec![MaterialEntry::Sampler(Sampler {
                    name: "n".to_owned(),
                    path: "a&b.dds".to_owned(),
                    srgb: None,
                    minfilter: None,
                    magfilter: None,
                    mipfilter: None,
                    uaddr: None,
                    vaddr: None,
                    waddr: None,
                    maxaniso: None,
                })],
            }],
            style: MtlStyle::default(),
        };
        let written = set.write();
        let text = String::from_utf8_lossy(&written);
        assert!(text.contains("&amp;"));
        assert!(text.contains("&quot;"));
        let reread = read(&written);
        assert_eq!(reread.materials[0].name, "a&b \"c\" <d>");
        match &reread.materials[0].entries[0] {
            MaterialEntry::Sampler(sampler) => assert_eq!(sampler.path, "a&b.dds"),
            entry => panic!("expected a sampler, got {entry:?}"),
        }
    }

    #[test]
    fn rejects_an_unknown_root_child() {
        assert_eq!(
            MaterialSet::read(b"<materialset><foo/></materialset>"),
            Err(MtlError::UnexpectedElement {
                element: "foo".to_owned(),
                parent: "materialset".to_owned(),
            })
        );
    }

    #[test]
    fn rejects_an_unknown_sampler_attribute() {
        assert_eq!(
            MaterialSet::read(
                b"<materialset><material name=\"m\" shader=\"s\"><sampler name=\"n\" path=\"p\" foo=\"1\" /></material></materialset>"
            ),
            Err(MtlError::UnexpectedAttribute {
                element: "sampler".to_owned(),
                attribute: "foo".to_owned(),
            })
        );
    }

    #[test]
    fn rejects_an_invalid_state_value() {
        assert!(matches!(
            MaterialSet::read(
                b"<materialset><material name=\"m\" shader=\"s\"><state name=\"n\" value=\"x\" /></material></materialset>"
            ),
            Err(MtlError::InvalidValue {
                element,
                attribute: "value",
                ..
            }) if element == "state"
        ));
    }

    #[test]
    fn rejects_an_invalid_filter() {
        assert!(matches!(
            MaterialSet::read(
                b"<materialset><material name=\"m\" shader=\"s\"><sampler name=\"n\" path=\"p\" minfilter=\"cubic\" /></material></materialset>"
            ),
            Err(MtlError::InvalidValue {
                element,
                attribute: "minfilter",
                ..
            }) if element == "sampler"
        ));
    }

    #[test]
    fn rejects_text_content() {
        assert_eq!(
            MaterialSet::read(b"<materialset>hello</materialset>"),
            Err(MtlError::UnexpectedText("materialset".to_owned()))
        );
    }

    #[test]
    fn rejects_a_missing_material_name() {
        assert!(matches!(
            MaterialSet::read(b"<materialset><material shader=\"s\"></material></materialset>"),
            Err(MtlError::MissingAttribute {
                element,
                attribute: "name",
            }) if element == "material"
        ));
    }

    #[test]
    fn rejects_an_incomplete_vector() {
        assert!(matches!(
            MaterialSet::read(
                b"<materialset><material name=\"m\" shader=\"s\"><vector name=\"n\" x=\"1.0\" z=\"3.0\" /></material></materialset>"
            ),
            Err(MtlError::InvalidValue {
                element,
                attribute: "y",
                ..
            }) if element == "vector"
        ));
    }

    #[test]
    fn rejects_malformed_xml() {
        assert!(matches!(
            MaterialSet::read(b"<materialset>"),
            Err(MtlError::Xml(_))
        ));
    }

    #[test]
    fn rejects_a_corrupt_wesys_wrapper() {
        let mut bytes = vec![0u8; 16];
        bytes[3..8].copy_from_slice(b"WESYS");
        assert!(matches!(MaterialSet::read(&bytes), Err(MtlError::Wesys(_))));
    }

    #[test]
    fn default_style_writes_konamis_layout() {
        let set = MaterialSet {
            materials: vec![Material {
                name: "m".to_owned(),
                shader: "s".to_owned(),
                entries: vec![MaterialEntry::Sampler(Sampler {
                    name: "n".to_owned(),
                    path: "p".to_owned(),
                    srgb: None,
                    minfilter: None,
                    magfilter: None,
                    mipfilter: None,
                    uaddr: None,
                    vaddr: None,
                    waddr: None,
                    maxaniso: None,
                })],
            }],
            style: MtlStyle::default(),
        };
        assert_eq!(
            String::from_utf8_lossy(&set.write()),
            "<materialset>\r\n    <material name=\"m\" shader=\"s\">\r\n        <sampler name=\"n\" path=\"p\" />\r\n    </material>\r\n</materialset>\r\n"
        );
    }
}
