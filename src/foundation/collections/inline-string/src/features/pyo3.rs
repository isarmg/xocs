//! PyO3 integration for CompactString
//!
//! This module provides seamless conversion between `CompactString` and Python strings,
//! allowing `CompactString` to be used in PyO3-based Python extensions.

use pyo3::prelude::*;
use pyo3::types::PyString;

use crate::CompactString;

#[cfg_attr(docsrs, doc(cfg(feature = "pyo3")))]
impl<'a, 'py> FromPyObject<'a, 'py> for CompactString {
    type Error = PyErr;

    fn extract(obj: pyo3::Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let s: &str = obj.extract()?;
        Ok(CompactString::from(s))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "pyo3")))]
impl<'py> IntoPyObject<'py> for CompactString {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = core::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self.as_str()))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "pyo3")))]
impl<'py> IntoPyObject<'py> for &CompactString {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = core::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, self.as_str()))
    }
}

// These tests initialize an embedded Python interpreter and call into libpython via FFI.
// Miri cannot execute foreign functions, so they're excluded under Miri (the impl above is
// still checked by Miri; only these FFI-driven tests are skipped).
#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/pyo3/tests.rs"]
#[cfg(all(test, not(miri)))]
mod tests;
