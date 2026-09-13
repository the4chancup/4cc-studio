//! The XML form of a `.fox2` file: `<fox>` root with `<classes>` and `<entities>`,
//! properties as `<property name type container>` holding `<value>` children.

use std::collections::HashSet;

use roxmltree::Node;

use crate::file::{Entity, Fox2Error, Fox2File, Property, TableEntry};
use crate::hash::hash_string;
use crate::text::{double_text, float_text, parse_float};
use crate::values::{Container, EntityLink, FoxString, Values, WideVector3, type_name, type_word};

/// What reading the XML form can fail with.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum XmlError {
    /// `roxmltree` could not parse the text.
    #[error("not well-formed XML: {0}")]
    Malformed(String),
    /// The root element has another tag.
    #[error("root element is <{0}>, not <fox>")]
    NotFox(String),
    /// A `type` attribute the format does not know.
    #[error("unknown data type {0:?}")]
    UnknownType(String),
    /// A `container` attribute the format does not know.
    #[error("unknown container {0:?}")]
    UnknownContainer(String),
    /// An attribute or element text that does not parse as its type.
    #[error("cannot read {what} from {text:?}")]
    BadValue {
        /// The value kind being read.
        what: &'static str,
        /// The text found.
        text: String,
    },
    /// A `<value>` in a `StringMap` property without a `key` attribute.
    #[error("<value> in a StringMap has no key")]
    MissingKey,
    /// The compiled binary layout failed.
    #[error(transparent)]
    Layout(#[from] Fox2Error),
}

fn attribute(out: &mut String, name: &str, value: &str) {
    out.push(' ');
    out.push_str(name);
    out.push_str("=\"");
    for char in value.chars() {
        match char {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '\t' => out.push_str("&#9;"),
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            _ => out.push(char),
        }
    }
    out.push('"');
}

fn text_escaped(text: &str, out: &mut String) {
    for char in text.chars() {
        match char {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\r' => out.push_str("&#13;"),
            _ => out.push(char),
        }
    }
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

/// `>{escaped}</value>` — an empty `text` gives `></value>` as the format writes it.
fn text_body(text: &str, out: &mut String) {
    out.push('>');
    text_escaped(text, out);
    out.push_str("</value>");
}

fn literal_text(string: &FoxString) -> &str {
    match string {
        FoxString::Literal(text) => text,
        FoxString::Hash(_) => "",
    }
}

fn hash_attr(string: &FoxString) -> String {
    format!("0x{:08X}", string.hash())
}

fn value_xml(out: &mut String, values: &Values, index: usize, key: Option<&FoxString>) {
    indent(out, 5);
    out.push_str("<value");
    if let Some(key) = key {
        match key {
            FoxString::Literal(text) => attribute(out, "key", text),
            FoxString::Hash(_) => attribute(out, "key", &hash_attr(key)),
        }
    }
    match values {
        Values::Int8(list) => text_body(&list[index].to_string(), out),
        Values::Uint8(list) => text_body(&list[index].to_string(), out),
        Values::Int16(list) => text_body(&list[index].to_string(), out),
        Values::Uint16(list) => text_body(&list[index].to_string(), out),
        Values::Int32(list) => text_body(&list[index].to_string(), out),
        Values::Uint32(list) => text_body(&list[index].to_string(), out),
        Values::Int64(list) => text_body(&list[index].to_string(), out),
        Values::Uint64(list) => text_body(&list[index].to_string(), out),
        Values::Float(list) => text_body(&float_text(list[index]), out),
        Values::Double(list) => text_body(&double_text(list[index]), out),
        Values::Bool(list) => {
            text_body(if list[index] { "true" } else { "false" }, out);
        }
        Values::String(list) | Values::Path(list) | Values::FilePtr(list) => match &list[index] {
            FoxString::Literal(text) => text_body(text, out),
            FoxString::Hash(hash) => {
                attribute(out, "hash", &format!("0x{hash:08X}"));
                out.push_str(" />");
            }
        },
        Values::EntityPtr(list) | Values::EntityHandle(list) => {
            text_body(&format!("0x{:08X}", list[index]), out);
        }
        Values::Vector3(list) | Values::Vector4(list) | Values::Quat(list) => {
            for (name, value) in ["x", "y", "z", "w"].iter().zip(list[index]) {
                attribute(out, name, &float_text(value));
            }
            out.push_str(" />");
        }
        Values::Color(list) => {
            for (name, value) in ["r", "g", "b", "a"].iter().zip(list[index]) {
                attribute(out, name, &float_text(value));
            }
            out.push_str(" />");
        }
        Values::Matrix3(list) => matrix_xml(out, &list[index], 3),
        Values::Matrix4(list) => matrix_xml(out, &list[index], 4),
        Values::EntityLink(list) => {
            let link = &list[index];
            link_attr(out, "packagePath", &link.package);
            link_attr(out, "archivePath", &link.archive);
            link_attr(out, "nameInArchive", &link.name);
            text_body(&format!("0x{:08X}", link.handle), out);
        }
        Values::WideVector3(list) => {
            let value = &list[index];
            attribute(out, "x", &float_text(value.x));
            attribute(out, "y", &float_text(value.y));
            attribute(out, "z", &float_text(value.z));
            attribute(out, "a", &value.a.to_string());
            attribute(out, "b", &value.b.to_string());
            out.push_str(" />");
        }
    }
    out.push('\n');
}

fn link_attr(out: &mut String, name: &str, string: &FoxString) {
    match string {
        FoxString::Literal(text) => attribute(out, name, text),
        FoxString::Hash(_) => attribute(out, &format!("{name}Hash"), &hash_attr(string)),
    }
}

fn matrix_xml(out: &mut String, matrix: &[f32], size: usize) {
    out.push_str(">\n");
    for row in 0..size {
        indent(out, 6);
        out.push_str("<Row");
        out.push_str(&(row + 1).to_string());
        for column in 0..size {
            attribute(
                out,
                &format!("Column{}", column + 1),
                &float_text(matrix[row * size + column]),
            );
        }
        out.push_str(" />\n");
    }
    indent(out, 5);
    out.push_str("</value>");
}

fn property_xml(out: &mut String, property: &Property) {
    indent(out, 4);
    out.push_str("<property");
    attribute(out, "name", literal_text(&property.name));
    if let FoxString::Hash(_) = property.name {
        attribute(out, "nameHash", &hash_attr(&property.name));
    }
    attribute(out, "type", type_name(property.values.data_type()));
    attribute(out, "container", property.container.name());
    if property.values.is_empty() {
        out.push_str(" />\n");
        return;
    }
    attribute(out, "arraySize", &property.values.len().to_string());
    out.push_str(">\n");
    for index in 0..property.values.len() {
        let key = if property.container == Container::StringMap {
            property.keys.get(index)
        } else {
            None
        };
        value_xml(out, &property.values, index, key);
    }
    indent(out, 4);
    out.push_str("</property>\n");
}

fn properties_xml(out: &mut String, tag: &str, properties: &[Property]) {
    indent(out, 3);
    out.push('<');
    out.push_str(tag);
    if properties.is_empty() {
        out.push_str(" />\n");
        return;
    }
    out.push_str(">\n");
    for property in properties {
        property_xml(out, property);
    }
    indent(out, 3);
    out.push_str("</");
    out.push_str(tag);
    out.push_str(">\n");
}

impl Fox2File {
    /// The XML form of this file, exactly as the goldens under `tests/fixtures/` show it.
    /// An unresolved (`Hash`) class or property name prints as `class=""`/`name=""` plus a
    /// `classHash`/`nameHash` attribute, which `from_xml` prefers, so unresolved names survive
    /// a decompile/compile round trip. A `StringMap` key literal starting with `0x` is
    /// ambiguous with a hash on read-back; no file on the machine has one.
    pub fn to_xml(&self) -> String {
        let mut out = String::new();
        out.push_str("<fox formatVersion=\"2\" fileVersion=\"0\" originalVersion=\"\">\n");
        out.push_str("  <classes>\n");
        out.push_str("    <class name=\"Entity\" super=\"\" version=\"2\" />\n");
        out.push_str("    <class name=\"Data\" super=\"Entity\" version=\"2\" />\n");
        let mut seen: HashSet<(String, String)> = HashSet::from([
            ("Entity".to_string(), String::new()),
            ("Data".to_string(), "Entity".to_string()),
        ]);
        for entity in &self.entities {
            let name = literal_text(&entity.class_name).to_string();
            if seen.insert((name.clone(), String::new())) {
                out.push_str("    <class");
                attribute(&mut out, "name", &name);
                attribute(&mut out, "super", "");
                attribute(&mut out, "version", &entity.version.to_string());
                out.push_str(" />\n");
            }
        }
        out.push_str("  </classes>\n");
        out.push_str("  <entities>\n");
        for entity in &self.entities {
            indent(&mut out, 2);
            out.push_str("<entity");
            attribute(&mut out, "class", literal_text(&entity.class_name));
            if let FoxString::Hash(_) = entity.class_name {
                attribute(&mut out, "classHash", &hash_attr(&entity.class_name));
            }
            attribute(&mut out, "classVersion", &entity.version.to_string());
            attribute(&mut out, "addr", &format!("0x{:08X}", entity.address));
            attribute(&mut out, "unknown1", &entity.unknown1.to_string());
            attribute(&mut out, "unknown2", &entity.unknown2.to_string());
            out.push_str(">\n");
            properties_xml(&mut out, "staticProperties", &entity.static_properties);
            properties_xml(&mut out, "dynamicProperties", &entity.dynamic_properties);
            indent(&mut out, 2);
            out.push_str("</entity>\n");
        }
        out.push_str("  </entities>\n");
        out.push_str("</fox>");
        out
    }

    /// A file from its XML form; strings become `Literal`s and the string table is rebuilt in
    /// traversal order (class name, then each property's name, keys and string literals,
    /// non-empty and deduplicated).
    pub fn from_xml(text: &str) -> Result<Fox2File, XmlError> {
        let document = roxmltree::Document::parse(text)
            .map_err(|error| XmlError::Malformed(error.to_string()))?;
        let root = document.root_element();
        if root.tag_name().name() != "fox" {
            return Err(XmlError::NotFox(root.tag_name().name().to_string()));
        }
        let mut entities = Vec::new();
        if let Some(list) = element(&root, "entities") {
            for node in children(&list, "entity") {
                entities.push(read_entity(&node)?);
            }
        }
        let mut string_table = Vec::new();
        let mut seen = HashSet::new();
        for entity in &entities {
            collect_literal(&entity.class_name, &mut string_table, &mut seen);
            for property in entity
                .static_properties
                .iter()
                .chain(entity.dynamic_properties.iter())
            {
                collect_literal(&property.name, &mut string_table, &mut seen);
                if property.container == Container::StringMap {
                    for (index, key) in property.keys.iter().enumerate() {
                        collect_literal(key, &mut string_table, &mut seen);
                        collect_value_literals(
                            &property.values,
                            index,
                            &mut string_table,
                            &mut seen,
                        );
                    }
                } else {
                    for index in 0..property.values.len() {
                        collect_value_literals(
                            &property.values,
                            index,
                            &mut string_table,
                            &mut seen,
                        );
                    }
                }
            }
        }
        Ok(Fox2File {
            entities,
            string_table,
        })
    }
}

fn collect_literal(string: &FoxString, table: &mut Vec<TableEntry>, seen: &mut HashSet<u64>) {
    if let FoxString::Literal(text) = string
        && !text.is_empty()
    {
        let hash = hash_string(text);
        if seen.insert(hash) {
            table.push(TableEntry {
                hash,
                text: text.clone(),
            });
        }
    }
}

fn collect_value_literals(
    values: &Values,
    index: usize,
    table: &mut Vec<TableEntry>,
    seen: &mut HashSet<u64>,
) {
    match values {
        Values::String(list) | Values::Path(list) | Values::FilePtr(list) => {
            collect_literal(&list[index], table, seen);
        }
        Values::EntityLink(list) => {
            let link = &list[index];
            collect_literal(&link.package, table, seen);
            collect_literal(&link.archive, table, seen);
            collect_literal(&link.name, table, seen);
        }
        _ => {}
    }
}

/// The first child element named `tag`.
fn element<'a, 'input>(node: &Node<'a, 'input>, tag: &'a str) -> Option<Node<'a, 'input>> {
    children(node, tag).next()
}

/// All child elements named `tag`.
fn children<'a, 'input>(
    node: &Node<'a, 'input>,
    tag: &'a str,
) -> impl Iterator<Item = Node<'a, 'input>> + 'a {
    node.children()
        .filter(move |child| child.is_element() && child.tag_name().name() == tag)
}

fn bad(what: &'static str, text: &str) -> XmlError {
    XmlError::BadValue {
        what,
        text: text.to_string(),
    }
}

/// `0x` hex or decimal, empty or whitespace as 0.
fn parse_int(text: &str, what: &'static str) -> Result<i128, XmlError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }
    if let Some(hex) = trimmed.strip_prefix("0x") {
        return i128::from_str_radix(hex, 16).map_err(|_| bad(what, text));
    }
    trimmed.parse::<i128>().map_err(|_| bad(what, text))
}

/// `text` parsed to the integer type of `what`, `BadValue` when it does not parse or fit.
fn int_as<T>(text: &str, what: &'static str) -> Result<T, XmlError>
where
    T: TryFrom<i128>,
{
    T::try_from(parse_int(text, what)?).map_err(|_| bad(what, text))
}

/// An attribute as `f32`, missing or empty as `0.0`.
fn float_attr(node: &Node, name: &str) -> Result<f32, XmlError> {
    match node.attribute(name) {
        Some(text) if !text.trim().is_empty() => {
            parse_float(text.trim()).ok_or_else(|| bad("float", text))
        }
        _ => Ok(0.0),
    }
}

/// A string slot from a literal attribute or its `...Hash` twin (`Hash(0)` when absent).
fn link_string(node: &Node, name: &str) -> Result<FoxString, XmlError> {
    if let Some(text) = node.attribute(name) {
        return Ok(FoxString::Literal(text.to_string()));
    }
    if let Some(text) = node.attribute(format!("{name}Hash").as_str()) {
        return Ok(FoxString::Hash(int_as(text, "hash")?));
    }
    Ok(FoxString::Hash(0))
}

fn read_entity(node: &Node) -> Result<Entity, XmlError> {
    let mut static_properties = Vec::new();
    let mut dynamic_properties = Vec::new();
    if let Some(list) = element(node, "staticProperties") {
        for child in children(&list, "property") {
            static_properties.push(read_property(&child)?);
        }
    }
    if let Some(list) = element(node, "dynamicProperties") {
        for child in children(&list, "property") {
            dynamic_properties.push(read_property(&child)?);
        }
    }
    let class_name = match node.attribute("classHash") {
        Some(hash) => FoxString::Hash(int_as(hash, "classHash")?),
        None => FoxString::Literal(node.attribute("class").unwrap_or_default().to_string()),
    };
    Ok(Entity {
        class_name,
        unknown1: int_as(node.attribute("unknown1").unwrap_or("0"), "unknown1")?,
        unknown2: int_as(node.attribute("unknown2").unwrap_or("0"), "unknown2")?,
        version: int_as(
            node.attribute("classVersion").unwrap_or("0"),
            "classVersion",
        )?,
        address: int_as(node.attribute("addr").unwrap_or("0"), "addr")?,
        static_properties,
        dynamic_properties,
    })
}

fn read_property(node: &Node) -> Result<Property, XmlError> {
    let word = match node.attribute("type") {
        Some(name) => type_word(name).ok_or_else(|| XmlError::UnknownType(name.to_string()))?,
        None => 0,
    };
    let container = match node.attribute("container") {
        Some(name) => Container::from_name(name)
            .ok_or_else(|| XmlError::UnknownContainer(name.to_string()))?,
        None => Container::StaticArray,
    };
    let mut values = Values::empty(word)?;
    let mut keys = Vec::new();
    for child in children(node, "value") {
        if container == Container::StringMap {
            let key = child.attribute("key").ok_or(XmlError::MissingKey)?;
            // `0x` plus valid hex is a hash; anything else (including `0xnothex`) is a
            // literal — the ambiguity is inherent to the format.
            keys.push(
                match key
                    .strip_prefix("0x")
                    .and_then(|hex| u64::from_str_radix(hex, 16).ok())
                {
                    Some(hash) => FoxString::Hash(hash),
                    None => FoxString::Literal(key.to_string()),
                },
            );
        }
        read_xml_value(&mut values, &child)?;
    }
    let name = match node.attribute("nameHash") {
        Some(hash) => FoxString::Hash(int_as(hash, "nameHash")?),
        None => FoxString::Literal(node.attribute("name").unwrap_or_default().to_string()),
    };
    Ok(Property {
        name,
        container,
        keys,
        values,
    })
}

fn read_xml_value(values: &mut Values, node: &Node) -> Result<(), XmlError> {
    let text = node.text().unwrap_or_default();
    match values {
        Values::Int8(list) => list.push(int_as(text, "int8")?),
        Values::Uint8(list) => list.push(int_as(text, "uint8")?),
        Values::Int16(list) => list.push(int_as(text, "int16")?),
        Values::Uint16(list) => list.push(int_as(text, "uint16")?),
        Values::Int32(list) => list.push(int_as(text, "int32")?),
        Values::Uint32(list) => list.push(int_as(text, "uint32")?),
        Values::Int64(list) => list.push(int_as(text, "int64")?),
        Values::Uint64(list) => list.push(int_as(text, "uint64")?),
        Values::Float(list) => {
            let trimmed = text.trim();
            list.push(if trimmed.is_empty() {
                0.0
            } else {
                parse_float(trimmed).ok_or_else(|| bad("float", text))?
            });
        }
        Values::Double(list) => {
            let trimmed = text.trim();
            list.push(if trimmed.is_empty() {
                0.0
            } else {
                trimmed.parse::<f64>().map_err(|_| bad("double", text))?
            });
        }
        Values::Bool(list) => {
            let trimmed = text.trim();
            list.push(match trimmed {
                "true" => true,
                "false" | "" => false,
                _ => return Err(bad("bool", text)),
            });
        }
        Values::String(list) | Values::Path(list) | Values::FilePtr(list) => {
            list.push(read_string(node)?);
        }
        Values::EntityPtr(list) | Values::EntityHandle(list) => {
            list.push(int_as(text, "handle")?);
        }
        Values::Vector3(list) | Values::Vector4(list) | Values::Quat(list) => {
            list.push([
                float_attr(node, "x")?,
                float_attr(node, "y")?,
                float_attr(node, "z")?,
                float_attr(node, "w")?,
            ]);
        }
        Values::Color(list) => {
            list.push([
                float_attr(node, "r")?,
                float_attr(node, "g")?,
                float_attr(node, "b")?,
                float_attr(node, "a")?,
            ]);
        }
        Values::Matrix3(list) => list.push(read_matrix::<9>(node, 3)?),
        Values::Matrix4(list) => list.push(read_matrix::<16>(node, 4)?),
        Values::EntityLink(list) => {
            list.push(EntityLink {
                package: link_string(node, "packagePath")?,
                archive: link_string(node, "archivePath")?,
                name: link_string(node, "nameInArchive")?,
                handle: int_as(text, "handle")?,
            });
        }
        Values::WideVector3(list) => {
            list.push(WideVector3 {
                x: float_attr(node, "x")?,
                y: float_attr(node, "y")?,
                z: float_attr(node, "z")?,
                a: int_as(node.attribute("a").unwrap_or("0"), "a")?,
                b: int_as(node.attribute("b").unwrap_or("0"), "b")?,
            });
        }
    }
    Ok(())
}

/// Element text → `Literal`, else a `hash` attribute → `Hash`, else `Literal("")`.
fn read_string(node: &Node) -> Result<FoxString, XmlError> {
    if let Some(text) = node.text() {
        return Ok(FoxString::Literal(text.to_string()));
    }
    if let Some(hash) = node.attribute("hash") {
        return Ok(FoxString::Hash(int_as(hash, "hash")?));
    }
    Ok(FoxString::Literal(String::new()))
}

fn read_matrix<const N: usize>(node: &Node, size: usize) -> Result<[f32; N], XmlError> {
    let mut matrix = [0.0f32; N];
    for row in 0..size {
        let row_name = format!("Row{}", row + 1);
        let row_node = element(node, &row_name);
        for column in 0..size {
            let column_name = format!("Column{}", column + 1);
            matrix[row * size + column] = match &row_node {
                Some(row_node) => float_attr(row_node, &column_name)?,
                None => 0.0,
            };
        }
    }
    Ok(matrix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::Dictionary;

    const AUDI_BIN: &[u8] = include_bytes!("../tests/fixtures/audi_low_parts.fox2");
    const AUDI_XML: &str = include_str!("../tests/fixtures/audi_low_parts.fox2.xml");
    const AUDI_DICT_XML: &str = include_str!("../tests/fixtures/audi_low_parts.dict.fox2.xml");
    const AUDI_COMPILED: &[u8] = include_bytes!("../tests/fixtures/audi_low_parts.compiled.fox2");
    const BOOTS_BIN: &[u8] = include_bytes!("../tests/fixtures/boots_edit_k0051.fox2");
    const BOOTS_XML: &str = include_str!("../tests/fixtures/boots_edit_k0051.fox2.xml");
    const BOOTS_COMPILED: &[u8] =
        include_bytes!("../tests/fixtures/boots_edit_k0051.compiled.fox2");
    const SPIKE_BIN: &[u8] = include_bytes!("../tests/fixtures/edit_spike.fox2");
    const SPIKE_XML: &str = include_str!("../tests/fixtures/edit_spike.fox2.xml");
    const SPIKE_COMPILED: &[u8] = include_bytes!("../tests/fixtures/edit_spike.compiled.fox2");
    const STEWARD_BIN: &[u8] = include_bytes!("../tests/fixtures/steward_sit_st074.fox2");
    const STEWARD_XML: &str = include_str!("../tests/fixtures/steward_sit_st074.fox2.xml");
    const STEWARD_COMPILED: &[u8] =
        include_bytes!("../tests/fixtures/steward_sit_st074.compiled.fox2");

    const FIXTURES: [(&str, &[u8], &str, &[u8]); 4] = [
        ("audi", AUDI_BIN, AUDI_XML, AUDI_COMPILED),
        ("boots", BOOTS_BIN, BOOTS_XML, BOOTS_COMPILED),
        ("spike", SPIKE_BIN, SPIKE_XML, SPIKE_COMPILED),
        ("steward", STEWARD_BIN, STEWARD_XML, STEWARD_COMPILED),
    ];

    /// The first line on which `left` and `right` differ, for mismatch reports.
    fn first_diff(left: &str, right: &str) -> String {
        for (line, (a, b)) in left.lines().zip(right.lines()).enumerate() {
            if a != b {
                return format!("line {}: {a:?} vs {b:?}", line + 1);
            }
        }
        format!(
            "line counts differ: {} vs {}",
            left.lines().count(),
            right.lines().count()
        )
    }

    #[test]
    fn decompile_matches_the_goldens() {
        for (name, binary, golden, _) in FIXTURES {
            let mut file = Fox2File::read(binary).expect(name);
            file.resolve(None);
            assert_eq!(
                file.to_xml(),
                golden,
                "{name}: {}",
                first_diff(&file.to_xml(), golden)
            );
        }
    }

    #[test]
    fn dictionary_resolves_the_empty_string() {
        let mut file = Fox2File::read(AUDI_BIN).expect("audi");
        file.resolve(Some(&Dictionary::from_lines("\n")));
        assert_eq!(
            file.to_xml(),
            AUDI_DICT_XML,
            "{}",
            first_diff(&file.to_xml(), AUDI_DICT_XML)
        );
    }

    #[test]
    fn compile_matches_the_reference() {
        for (name, _, golden, compiled) in FIXTURES {
            let file = Fox2File::from_xml(golden).expect(name);
            assert_eq!(file.write().as_deref(), Ok(compiled), "{name}");
        }
        let file = Fox2File::from_xml(AUDI_DICT_XML).expect("dict");
        assert_eq!(file.write().as_deref(), Ok(AUDI_COMPILED));
    }

    #[test]
    fn compile_then_decompile_is_stable() {
        for (name, _, golden, _) in FIXTURES {
            let file = Fox2File::from_xml(golden).expect(name);
            let mut back = Fox2File::read(&file.write().expect(name)).expect(name);
            back.resolve(None);
            assert_eq!(
                back.to_xml(),
                golden,
                "{name}: {}",
                first_diff(&back.to_xml(), golden)
            );
        }
    }

    #[test]
    fn every_value_type_survives_xml() {
        let property = |container: Container, values: Values, keys: &[&str]| Property {
            name: FoxString::Literal("p".to_string()),
            container,
            keys: keys
                .iter()
                .map(|key| FoxString::Literal((*key).to_string()))
                .collect(),
            values,
        };
        let alpha = || FoxString::Literal("alpha".to_string());
        let beta = || FoxString::Literal("beta".to_string());
        let file = Fox2File {
            entities: vec![Entity {
                class_name: FoxString::Literal("DataSet".to_string()),
                unknown1: 0,
                unknown2: 0,
                version: 0,
                address: 0,
                static_properties: vec![
                    property(Container::StaticArray, Values::Int8(vec![-3]), &[]),
                    property(Container::StaticArray, Values::Int16(vec![-300]), &[]),
                    property(
                        Container::StaticArray,
                        Values::Int64(vec![-9_000_000_000_000]),
                        &[],
                    ),
                    property(Container::StaticArray, Values::Uint16(vec![60000]), &[]),
                    property(Container::StaticArray, Values::Uint64(vec![u64::MAX]), &[]),
                    property(Container::StaticArray, Values::Double(vec![0.1, 1e-5]), &[]),
                    property(Container::StaticArray, Values::Matrix3(vec![[1.0; 9]]), &[]),
                    property(
                        Container::StaticArray,
                        Values::Matrix4(vec![[2.0; 16]]),
                        &[],
                    ),
                    property(
                        Container::StaticArray,
                        Values::WideVector3(vec![WideVector3 {
                            x: 1.5,
                            y: -2.25,
                            z: 3.0,
                            a: 7,
                            b: 8,
                        }]),
                        &[],
                    ),
                    property(
                        Container::StaticArray,
                        Values::String(vec![alpha(), beta()]),
                        &[],
                    ),
                    property(
                        Container::StringMap,
                        Values::String(vec![alpha(), beta()]),
                        &["k1", "k2"],
                    ),
                ],
                dynamic_properties: Vec::new(),
            }],
            // Traversal order, non-empty literals, first occurrence kept.
            string_table: ["DataSet", "p", "alpha", "beta", "k1", "k2"]
                .iter()
                .map(|text| TableEntry {
                    hash: hash_string(text),
                    text: (*text).to_string(),
                })
                .collect(),
        };
        let xml = file.to_xml();
        assert_eq!(Fox2File::from_xml(&xml).as_ref(), Ok(&file), "{}", xml);
    }

    #[test]
    fn malformed_xml_errors() {
        assert_eq!(
            Fox2File::from_xml("<foo />"),
            Err(XmlError::NotFox("foo".to_string()))
        );
        assert!(matches!(
            Fox2File::from_xml("<fox"),
            Err(XmlError::Malformed(_))
        ));
        let bad_type = "<fox><entities><entity><staticProperties>\
            <property type=\"Vector9\"><value>1</value></property>\
            </staticProperties></entity></entities></fox>";
        assert_eq!(
            Fox2File::from_xml(bad_type),
            Err(XmlError::UnknownType("Vector9".to_string()))
        );
        let bad_container = "<fox><entities><entity><staticProperties>\
            <property type=\"int32\" container=\"Bag\"><value>1</value></property>\
            </staticProperties></entity></entities></fox>";
        assert_eq!(
            Fox2File::from_xml(bad_container),
            Err(XmlError::UnknownContainer("Bag".to_string()))
        );
        let missing_key = "<fox><entities><entity><staticProperties>\
            <property type=\"int32\" container=\"StringMap\"><value>1</value></property>\
            </staticProperties></entity></entities></fox>";
        assert_eq!(Fox2File::from_xml(missing_key), Err(XmlError::MissingKey));
        let bad_int = "<fox><entities><entity><staticProperties>\
            <property type=\"int32\"><value>abc</value></property>\
            </staticProperties></entity></entities></fox>";
        assert!(matches!(
            Fox2File::from_xml(bad_int),
            Err(XmlError::BadValue { what: "int32", .. })
        ));
    }

    #[test]
    fn unresolved_names_survive_the_round_trip() {
        let file = Fox2File::read(AUDI_BIN).expect("audi");
        let xml = file.to_xml();
        let class_hash = format!("classHash=\"0x{:08X}\"", hash_string("DataSet"));
        let name_hash = format!("nameHash=\"0x{:08X}\"", hash_string("name"));
        assert!(xml.contains(&class_hash), "{class_hash}");
        assert!(xml.contains(&name_hash), "{name_hash}");
        let rebuilt = Fox2File::from_xml(&xml).expect("xml");
        let bytes = rebuilt.write().expect("write");
        // The entity region (everything before the string table) must match the original;
        // the rebuilt table holds only the literals, so offsets may differ.
        let original_end = i32::from_le_bytes(AUDI_BIN[12..16].try_into().unwrap()) as usize;
        let rebuilt_end = i32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        assert_eq!(&bytes[32..rebuilt_end], &AUDI_BIN[32..original_end]);
    }

    #[test]
    fn whitespace_is_escaped_as_references() {
        let file = Fox2File {
            entities: vec![Entity {
                class_name: FoxString::Literal("DataSet".to_string()),
                unknown1: 0,
                unknown2: 0,
                version: 0,
                address: 0,
                static_properties: vec![Property {
                    name: FoxString::Literal("p".to_string()),
                    container: Container::StringMap,
                    keys: vec![FoxString::Literal("a\tb".to_string())],
                    values: Values::String(vec![FoxString::Literal("x\r\ny".to_string())]),
                }],
                dynamic_properties: Vec::new(),
            }],
            string_table: vec![
                TableEntry {
                    hash: hash_string("DataSet"),
                    text: "DataSet".to_string(),
                },
                TableEntry {
                    hash: hash_string("p"),
                    text: "p".to_string(),
                },
                TableEntry {
                    hash: hash_string("a\tb"),
                    text: "a\tb".to_string(),
                },
                TableEntry {
                    hash: hash_string("x\r\ny"),
                    text: "x\r\ny".to_string(),
                },
            ],
        };
        let xml = file.to_xml();
        assert!(xml.contains("&#9;"), "{xml}");
        assert_eq!(Fox2File::from_xml(&xml).as_ref(), Ok(&file), "{xml}");
    }

    #[test]
    fn strict_bools() {
        let xml = |value: &str| {
            format!(
                "<fox><entities><entity><staticProperties>\
                 <property type=\"bool\"><value>{value}</value></property>\
                 </staticProperties></entity></entities></fox>"
            )
        };
        let read_bool = |value: &str| {
            Fox2File::from_xml(&xml(value)).map(|file| {
                match file.entities[0].static_properties[0].values {
                    Values::Bool(ref list) => list[0],
                    _ => panic!("not a bool"),
                }
            })
        };
        assert_eq!(read_bool("true"), Ok(true));
        assert_eq!(read_bool("false"), Ok(false));
        assert_eq!(read_bool(""), Ok(false));
        assert_eq!(
            read_bool("1"),
            Err(XmlError::BadValue {
                what: "bool",
                text: "1".to_string()
            })
        );
        assert!(matches!(
            read_bool("tru"),
            Err(XmlError::BadValue { what: "bool", .. })
        ));
    }

    #[test]
    fn hex_prefixed_keys() {
        let xml = |key: &str| {
            format!(
                "<fox><entities><entity><staticProperties>\
                 <property type=\"int32\" container=\"StringMap\">\
                 <value key=\"{key}\">1</value></property>\
                 </staticProperties></entity></entities></fox>"
            )
        };
        let file = Fox2File::from_xml(&xml("0xB8A0BF169F98")).expect("hash key");
        assert_eq!(
            file.entities[0].static_properties[0].keys,
            vec![FoxString::Hash(0xB8A0BF169F98)]
        );
        let file = Fox2File::from_xml(&xml("0xnothex")).expect("literal key");
        assert_eq!(
            file.entities[0].static_properties[0].keys,
            vec![FoxString::Literal("0xnothex".to_string())]
        );
    }
}
