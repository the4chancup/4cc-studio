//! The `pes_models_native` Python extension (core plan, "Phase 2", `python_bindings`):
//! the `fmdl` and `pes_model` codecs for Blender's bundled Python, one wheel per
//! platform via `abi3-py311`. The surface is each codec's `read`/`write`; Blender
//! accessors arrive with the `pes-models` extension's hot-path step.

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

create_exception!(
    pes_models_native,
    FormatError,
    PyException,
    "A model file failed to parse or write; the message is the codec's own error text."
);

/// A codec error as the Python exception, carrying the codec's own message.
fn format_error(error: impl std::fmt::Display) -> PyErr {
    FormatError::new_err(error.to_string())
}

/// `pes_models_native.fmdl`: the Fox Engine FMDL/SKL codecs.
#[pymodule]
mod fmdl {
    use super::*;

    /// A Fox Engine `.fmdl` model file (`fmdl::FmdlFile`).
    #[pyclass]
    struct Fmdl(::fmdl::FmdlFile);

    #[pymethods]
    impl Fmdl {
        /// Parse a `.fmdl` file's bytes.
        #[staticmethod]
        fn read(data: &[u8]) -> PyResult<Self> {
            ::fmdl::FmdlFile::read(data).map(Self).map_err(format_error)
        }

        /// Serialize back to the `.fmdl` bytes.
        fn write<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
            PyBytes::new(py, &self.0.write())
        }
    }

    /// A Fox Engine `.skl` skeleton file (`fmdl::SklFile`).
    #[pyclass]
    struct Skl(::fmdl::SklFile);

    #[pymethods]
    impl Skl {
        /// Parse a `.skl` file's bytes.
        #[staticmethod]
        fn read(data: &[u8]) -> PyResult<Self> {
            ::fmdl::SklFile::read(data).map(Self).map_err(format_error)
        }

        /// Serialize back to the `.skl` bytes.
        fn write<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
            PyBytes::new(py, &self.0.write())
        }
    }
}

/// `pes_models_native.pes_model`: the pre-Fox `.model`/`.mtl` codecs.
#[pymodule]
mod pes_model {
    use super::*;

    /// A pre-Fox `.model` file, unwrapped (`pes_model::format::PreFoxModel`; the WESYS
    /// container is the archive layer's concern, not the codec's).
    #[pyclass]
    struct Model(::pes_model::format::PreFoxModel);

    #[pymethods]
    impl Model {
        /// Parse a `.model` file's bytes.
        #[staticmethod]
        fn read(data: &[u8]) -> PyResult<Self> {
            ::pes_model::format::PreFoxModel::read(data)
                .map(Self)
                .map_err(format_error)
        }

        /// Serialize back to the `.model` bytes.
        fn write<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
            self.0
                .write()
                .map(|bytes| PyBytes::new(py, &bytes))
                .map_err(format_error)
        }
    }

    /// A pre-Fox `.mtl` material set (`pes_model::format::mtl::MaterialSet`).
    #[pyclass]
    struct MaterialSet(::pes_model::format::mtl::MaterialSet);

    #[pymethods]
    impl MaterialSet {
        /// Parse a `.mtl` file's bytes.
        #[staticmethod]
        fn read(data: &[u8]) -> PyResult<Self> {
            ::pes_model::format::mtl::MaterialSet::read(data)
                .map(Self)
                .map_err(format_error)
        }

        /// Serialize back to the `.mtl` bytes.
        fn write<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
            PyBytes::new(py, &self.0.write())
        }
    }
}

/// The extension module itself.
#[pymodule]
mod pes_models_native {
    use super::*;

    // The exception lives on the top module; the submodules raise it.
    #[pymodule_export]
    use FormatError;

    #[pymodule_export]
    use super::fmdl;

    #[pymodule_export]
    use super::pes_model;

    #[pymodule_init]
    fn init(_module: &Bound<'_, PyModule>) -> PyResult<()> {
        // Forwards the libs' `log` output to Python's `logging`. The handle only resets
        // the logger's caches; dropping it leaves the logger installed for the
        // interpreter's lifetime, which is the intent.
        let _reset_handle = pyo3_log::init();
        Ok(())
    }
}
