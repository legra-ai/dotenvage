//! Python bindings for dotenvage using PyO3

use std::collections::HashMap;
use std::path::Path;

use ::dotenvage::{
    AutoDetectPatterns,
    EnvLoader,
    SecretManager,
};
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;

/// Wrapper for SecretManager in Python
#[pyclass(name = "SecretManager")]
pub struct PySecretManager {
    inner: SecretManager,
}

#[pymethods]
impl PySecretManager {
    /// Creates a new SecretManager by loading the key from standard locations
    #[staticmethod]
    fn load_default<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        future_into_py(py, async move {
            let inner = SecretManager::new()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
            Python::attach(|py| Py::new(py, PySecretManager { inner }))
        })
    }

    /// Generates a new random identity
    #[staticmethod]
    fn generate() -> PyResult<Self> {
        Ok(Self {
            inner: SecretManager::generate()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?,
        })
    }

    /// Creates a SecretManager from an existing identity string
    #[staticmethod]
    fn from_identity_string(identity: &str) -> PyResult<Self> {
        use age::x25519;

        let parsed_identity = identity.parse::<x25519::Identity>().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid identity: {}", e))
        })?;

        Ok(Self {
            inner: SecretManager::from_identity(parsed_identity),
        })
    }

    /// Gets the public key as a string in age format (starts with `age1`)
    fn public_key_string(&self) -> String {
        self.inner.public_key_string()
    }

    /// Encrypts a plaintext value and wraps it in the format `ENC[AGE:b64:...]`
    fn encrypt_value(&self, plaintext: &str) -> PyResult<String> {
        self.inner
            .encrypt_value(plaintext)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Decrypts a value if it's encrypted; otherwise returns it unchanged
    fn decrypt_value(&self, value: &str) -> PyResult<String> {
        self.inner
            .decrypt_value(value)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    }

    /// Checks if a value is in a recognized encrypted format
    #[staticmethod]
    fn is_encrypted(value: &str) -> bool {
        SecretManager::is_encrypted(value)
    }
}

/// Wrapper for EnvLoader in Python
#[pyclass(name = "EnvLoader")]
pub struct PyEnvLoader {
    inner: EnvLoader,
}

#[pymethods]
impl PyEnvLoader {
    /// Creates a new EnvLoader with a default SecretManager
    #[staticmethod]
    fn load_default<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        future_into_py(py, async move {
            let inner = EnvLoader::new()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
            Python::attach(|py| Py::new(py, PyEnvLoader { inner }))
        })
    }

    /// Creates an EnvLoader with a specific SecretManager
    #[staticmethod]
    fn with_manager(manager: &PySecretManager) -> Self {
        Self {
            inner: EnvLoader::with_manager(manager.inner.clone()),
        }
    }

    /// Loads `.env` files from the current directory in standard order.
    /// Decrypted values are loaded into the process environment.
    /// Returns the list of file paths that were actually loaded, in load order.
    fn load<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            let paths = inner
                .load()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
            Ok(paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<String>>())
        })
    }

    /// Loads `.env` files from a specific directory using the same order.
    /// Returns the list of file paths that were actually loaded, in load order.
    fn load_from_dir<'py>(&self, py: Python<'py>, dir: String) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            let paths = inner
                .load_from_dir(Path::new(&dir))
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
            Ok(paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<String>>())
        })
    }

    /// Gets all variable names from all loaded `.env` files
    fn get_all_variable_names<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            inner
                .get_all_variable_names()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
        })
    }

    /// Gets all variable names from `.env` files in a specific directory
    fn get_all_variable_names_from_dir<'py>(
        &self,
        py: Python<'py>,
        dir: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            inner
                .get_all_variable_names_from_dir(Path::new(&dir))
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
        })
    }

    /// Computes the ordered list of env file paths to load
    fn resolve_env_paths<'py>(&self, py: Python<'py>, dir: String) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            let paths = inner
                .resolve_env_paths(Path::new(&dir))
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
            Ok(paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<String>>())
        })
    }

    /// Gets all environment variables as a dict (decrypted)
    /// Note: This loads variables into the process environment first
    fn get_all_variables<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            // First load into environment (ignore returned paths)
            let _ = inner
                .load()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

            let names = inner
                .get_all_variable_names()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

            // Collect variables into a HashMap
            let mut vars: HashMap<String, String> = HashMap::new();
            for name in names {
                if let Ok(value) = std::env::var(&name) {
                    vars.insert(name, value);
                }
            }

            Ok(vars)
        })
    }

    /// Gets all environment variables from a specific directory as a dict
    /// (decrypted). Note: This loads variables into the process environment
    /// first
    fn get_all_variables_from_dir<'py>(
        &self,
        py: Python<'py>,
        dir: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        future_into_py(py, async move {
            // First load into environment (ignore returned paths)
            let _ = inner
                .load_from_dir(Path::new(&dir))
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

            let names = inner
                .get_all_variable_names_from_dir(Path::new(&dir))
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

            // Collect variables into a HashMap
            let mut vars: HashMap<String, String> = HashMap::new();
            for name in names {
                if let Ok(value) = std::env::var(&name) {
                    vars.insert(name, value);
                }
            }

            Ok(vars)
        })
    }
}

/// Checks if a key name should be encrypted based on auto-detection patterns
#[pyfunction]
fn should_encrypt(key: &str) -> bool {
    AutoDetectPatterns::should_encrypt(key)
}

/// Python module initialization
#[pymodule]
fn dotenvage(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySecretManager>()?;
    m.add_class::<PyEnvLoader>()?;
    m.add_function(wrap_pyfunction!(should_encrypt, m)?)?;
    Ok(())
}
