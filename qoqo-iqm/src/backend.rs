// Copyright © 2020-2023 HQS Quantum Simulations GmbH. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyByteArray;

use crate::devices::*;
use qoqo::convert_into_circuit;
use roqoqo::prelude::*;
use roqoqo::registers::Registers;
use roqoqo_iqm::{Backend, IqmDevice};

use bincode::{deserialize, serialize};

/// IQM backend
///
/// Provides functions to run circuits and measurements on IQM devices.
#[pyclass(name = "Backend", module = "qoqo_iqm")]
#[derive(Clone, Debug, PartialEq, Eq)]
#[pyo3(text_signature = "(device, access_token, timeout, mock_port)")]
pub struct BackendWrapper {
    /// Internal storage of [roqoqo_iqm::Backend]
    pub internal: Backend,
}

impl BackendWrapper {
    /// Extracts a Backend from a BackendWrapper python object.
    ///
    /// When working with qoqo and other rust based python packages compiled separately a downcast
    /// will not detect that two BackendWrapper objects are compatible. This function tries to
    /// convert a Python object into a Backend instance by first checking if the object is a
    /// BackendWrapper instance and, if not, by invoking the to_bincode method on the object and
    /// deserializing the returned binary data.
    ///
    ///
    /// # Arguments:
    ///
    /// `input` - The Python object that should be casted to a [roqoqo_iqm::Backend]
    pub fn from_pyany(input: Py<PyAny>) -> PyResult<Backend> {
        Python::with_gil(|py| -> PyResult<Backend> {
            let input = input.as_ref(py);
            if let Ok(try_downcast) = input.extract::<BackendWrapper>() {
                Ok(try_downcast.internal)
            } else {
                let get_bytes = input.call_method0("to_bincode").map_err(|_| {
                PyTypeError::new_err("Python object cannot be converted to IQM Backend: Cast to binary representation failed".to_string())
            })?;
                let bytes = get_bytes.extract::<Vec<u8>>().map_err(|_| {
                PyTypeError::new_err("Python object cannot be converted to IQM Backend: Cast to binary representation failed".to_string())
            })?;
                deserialize(&bytes[..]).map_err(|err| {
                    PyTypeError::new_err(format!(
                    "Python object cannot be converted to IQM Backend: Deserialization failed: {}",
                    err
                ))
                })
            }
        })
    }
}

#[pymethods]
impl BackendWrapper {
    /// Create a new IQM Backend.
    ///
    /// Args:
    ///     device (Device): IQM Device providing information about the endpoint running Circuits.
    ///     access_token (Optional[str]): Optional access token to IQM endpoints.
    ///                                   When None access token is read from $IQM_TOKENS_FILE environmental variable
    ///
    /// Raises:
    ///     TypeError: Device Parameter is not IqmDevice
    ///     RuntimeError: No access token found
    #[new]
    pub fn new(device: &PyAny, access_token: Option<String>) -> PyResult<Self> {
        // TODO for the moment only supports conversion into DemoDevice When support for more
        // devices is added, we should perform the conversion into the correct device type
        let device = DemoDeviceWrapper::from_pyany(device.into()).map_err(|err| {
            PyTypeError::new_err(format!(
                "Device parameter cannot be cast to IqmDevice {:?}",
                err
            ))
        })?;
        let iqm_device: IqmDevice = IqmDevice::from(device);
        Ok(Self {
            internal: Backend::new(iqm_device, access_token).map_err(|err| {
                PyRuntimeError::new_err(format!("No access token found {:?}", err))
            })?,
        })
    }

    /// Return a copy of the Backend (copy here produces a deepcopy).
    ///
    /// Returns:
    ///     Backend: A deep copy of self.
    pub fn __copy__(&self) -> BackendWrapper {
        self.clone()
    }

    /// Return a deep copy of the Backend.
    ///
    /// Returns:
    ///     Backend: A deep copy of self.
    pub fn __deepcopy__(&self, _memodict: Py<PyAny>) -> BackendWrapper {
        self.clone()
    }

    /// Return the bincode representation of the Backend using the [bincode] crate.
    ///
    /// Returns:
    ///     ByteArray: The serialized Backend (in [bincode] form).
    ///
    /// Raises:
    ///     ValueError: Cannot serialize Backend to bytes.
    pub fn to_bincode(&self) -> PyResult<Py<PyByteArray>> {
        let serialized = serialize(&self.internal)
            .map_err(|_| PyValueError::new_err("Cannot serialize Backend to bytes"))?;
        let b: Py<PyByteArray> = Python::with_gil(|py| -> Py<PyByteArray> {
            PyByteArray::new(py, &serialized[..]).into()
        });
        Ok(b)
    }

    /// Convert the bincode representation of the Backend to a Backend using the [bincode] crate.
    ///
    /// Args:
    ///     input (ByteArray): The serialized Backend (in [bincode] form).
    ///
    /// Returns:
    ///     Backend: The deserialized Backend.
    ///
    /// Raises:
    ///     TypeError: Input cannot be converted to byte array.
    ///     ValueError: Input cannot be deserialized to Backend.
    #[staticmethod]
    pub fn from_bincode(input: &PyAny) -> PyResult<BackendWrapper> {
        let bytes = input
            .extract::<Vec<u8>>()
            .map_err(|_| PyTypeError::new_err("Input cannot be converted to byte array"))?;

        Ok(BackendWrapper {
            internal: deserialize(&bytes[..])
                .map_err(|_| PyValueError::new_err("Input cannot be deserialized to Backend"))?,
        })
    }

    /// Run a circuit with the IQM backend.
    ///
    /// A circuit is passed to the backend and executed.
    /// During execution values are written to and read from classical registers
    /// (List[bool], List[float], List[complex]).
    /// To produce sufficient statistics for evaluating expectation values,
    /// circuits have to be run multiple times.
    /// The results of each repetition are concatenated in OutputRegisters
    /// (List[List[bool]], List[List[float]], List[List[complex]]).  
    ///
    /// Args:
    ///     circuit (Circuit): The circuit that is run on the backend.
    ///
    /// Returns:
    ///     Tuple[Dict[str, List[List[bool]]], Dict[str, List[List[float]]]], Dict[str, List[List[complex]]]]: The output registers written by the evaluated circuits.
    ///
    /// Raises:
    ///     TypeError: Circuit argument cannot be converted to qoqo Circuit
    ///     RuntimeError: Running Circuit failed
    pub fn run_circuit(&self, circuit: &PyAny) -> PyResult<Registers> {
        let circuit = convert_into_circuit(circuit).map_err(|err| {
            PyTypeError::new_err(format!(
                "Circuit argument cannot be converted to qoqo Circuit {:?}",
                err
            ))
        })?;
        self.internal
            .run_circuit(&circuit)
            .map_err(|err| PyRuntimeError::new_err(format!("Running Circuit failed {:?}", err)))
    }
}
