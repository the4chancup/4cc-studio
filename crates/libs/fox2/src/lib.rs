//! Fox Engine entity files (`.fox2`): the binary layout, the FoxTool XML form, and the
//! CityHash64-based string hash the format keys every name by.

/// The binary layout: `Fox2File`, `Entity`, `Property`, `Fox2Error`.
pub mod file;
/// The 48-bit hash the format keys names by.
pub mod hash;
/// The round-trip float text the XML form uses.
pub mod text;
/// The property value model: `Values`, `FoxString`, `Container`.
pub mod values;
/// The XML form: `Fox2File::to_xml` / `from_xml`, `XmlError`.
pub mod xml;
