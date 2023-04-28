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

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyByteArray;

use roqoqo::devices::Device;
// use qoqo::QoqoBackendError;
use bincode::{deserialize, serialize};
use roqoqo_iqm::devices::DemoDevice;

/// IQM demo environment device
///
/// Provides endpoint that receives instructions to test the IQM REST API and returns pseudorandom numbers.
#[pyclass(name = "DemoDevice", module = "qoqo_iqm")]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DemoDeviceWrapper {
    /// Internal storage of [roqoqo_iqm::DemoDevice]
    pub internal: DemoDevice,
}

impl DemoDeviceWrapper {
    /// Extracts a DemoDevice from a DemoDeviceWrapper python object.
    ///
    /// When working with qoqo and other rust based python packages compiled separately a downcast
    /// will not detect that two DemoDeviceWrapper objects are compatible. This function tries to
    /// convert a Python object into a DemoDevice instance by first checking if the object is a
    /// DemoDeviceWrapper instance and, if not, by invoking the to_bincode method on the object and
    /// deserializing the returned binary data.
    ///
    ///
    /// # Arguments:
    ///
    /// `input` - The Python object that should be casted to a [roqoqo_iqm::DemoDevice]
    pub fn from_pyany(input: Py<PyAny>) -> PyResult<DemoDevice> {
        Python::with_gil(|py| -> PyResult<DemoDevice> {
            let input = input.as_ref(py);
            if let Ok(try_downcast) = input.extract::<DemoDeviceWrapper>() {
                Ok(try_downcast.internal)
            } else {
                let get_bytes = input.call_method0("to_bincode").map_err(|_| {
                PyTypeError::new_err("Python object cannot be converted to IQM DemoDevice: Cast to binary representation failed".to_string())
            })?;
                let bytes = get_bytes.extract::<Vec<u8>>().map_err(|_| {
                PyTypeError::new_err("Python object cannot be converted to IQM DemoDevice: Cast to binary representation failed".to_string())
            })?;
                deserialize(&bytes[..]).map_err(|err| {
                    PyTypeError::new_err(format!(
                    "Python object cannot be converted to IQM DemoDevice: Deserialization failed: {}",
                    err
                ))
                })
            }
        })
    }
}

#[pymethods]
impl DemoDeviceWrapper {
    /// Create new simulator device.
    #[new]
    pub fn new() -> Self {
        Self {
            internal: DemoDevice::new(),
        }
    }

    /// Change API endpoint URL of the device
    ///
    /// # Arguments
    ///
    /// `new_url` - The new URL to set.
    pub fn set_endpoint_url(&mut self, new_url: String) {
        self.internal.set_endpoint_url(new_url)
    }

    /// Return a copy of the DemoDevice (copy here produces a deepcopy).
    ///
    /// Returns:
    ///     DemoDevice: A deep copy of self.
    pub fn __copy__(&self) -> DemoDeviceWrapper {
        self.clone()
    }

    /// Return a deep copy of the DemoDevice.
    ///
    /// Returns:
    ///     DemoDevice: A deep copy of self.
    pub fn __deepcopy__(&self, _memodict: Py<PyAny>) -> DemoDeviceWrapper {
        self.clone()
    }

    /// Return the bincode representation of the DemoDevice using the [bincode] crate.
    ///
    /// Returns:
    ///     ByteArray: The serialized DemoDevice (in [bincode] form).
    ///
    /// Raises:
    ///     ValueError: Cannot serialize DemoDevice to bytes.
    pub fn to_bincode(&self) -> PyResult<Py<PyByteArray>> {
        let serialized = serialize(&self.internal)
            .map_err(|_| PyValueError::new_err("Cannot serialize DemoDevice to bytes"))?;
        let b: Py<PyByteArray> = Python::with_gil(|py| -> Py<PyByteArray> {
            PyByteArray::new(py, &serialized[..]).into()
        });
        Ok(b)
    }

    /// Convert the bincode representation of the DemoDevice to a DemoDevice using the [bincode] crate.
    ///
    /// Args:
    ///     input (ByteArray): The serialized DemoDevice (in [bincode] form).
    ///
    /// Returns:
    ///     DemoDevice: The deserialized DemoDevice.
    ///
    /// Raises:
    ///     TypeError: Input cannot be converted to byte array.
    ///     ValueError: Input cannot be deserialized to DemoDevice.
    #[staticmethod]
    pub fn from_bincode(input: &PyAny) -> PyResult<DemoDeviceWrapper> {
        let bytes = input
            .extract::<Vec<u8>>()
            .map_err(|_| PyTypeError::new_err("Input cannot be converted to byte array"))?;

        Ok(DemoDeviceWrapper {
            internal: deserialize(&bytes[..])
                .map_err(|_| PyValueError::new_err("Input cannot be deserialized to DemoDevice"))?,
        })
    }

    /// Return number of qubits simulated by DemoDevice.
    ///
    /// Returns:
    ///     int: The number of qubits.
    ///
    pub fn number_qubits(&self) -> usize {
        self.internal.number_qubits()
    }

    /// Return the URL of the API endpoint for the device.
    ///
    /// Returns:
    ///     str: The URL of the remote host executing the Circuits.
    ///
    pub fn remote_host(&self) -> String {
        self.internal.remote_host()
    }

    /// Return the list of pairs of qubits linked by a native two-qubit-gate in the device.
    ///
    /// A pair of qubits is considered linked by a native two-qubit-gate if the device
    /// can implement a two-qubit-gate between the two qubits without decomposing it
    /// into a sequence of gates that involves a third qubit of the device.
    /// The two-qubit-gate also has to form a universal set together with the available
    /// single qubit gates.
    ///
    /// The returned vectors is a simple, graph-library independent, representation of
    /// the undirected connectivity graph of the device.
    /// It can be used to construct the connectivity graph in a graph library of the user's
    /// choice from a list of edges and can be used for applications like routing in quantum algorithms.
    fn two_qubit_edges(&self) -> Vec<(usize, usize)> {
        self.internal.two_qubit_edges()
    }

    /// Returns the gate time of a single qubit operation on this device.
    ///
    /// Returns:
    ///     f64: The gate time.
    ///
    /// Raises:
    ///     ValueError: The gate is not available in the device.
    pub fn single_qubit_gate_time(&self, hqslang: &str, qubit: usize) -> PyResult<f64> {
        self.internal
            .single_qubit_gate_time(hqslang, &qubit)
            .ok_or_else(|| PyValueError::new_err("The gate is not available on the device."))
    }

    /// Returns the gate time of a two qubit operation on this device.
    ///
    /// Returns:
    ///     f64: The gate time.
    ///
    /// Raises:
    ///     ValueError: The gate is not available in the device.
    pub fn two_qubit_gate_time(
        &self,
        hqslang: &str,
        control: usize,
        target: usize,
    ) -> PyResult<f64> {
        self.internal
            .two_qubit_gate_time(hqslang, &control, &target)
            .ok_or_else(|| PyValueError::new_err("The gate is not available on the device."))
    }

    /// Returns the gate time of a multi qubit operation on this device.
    ///
    /// Returns:
    ///     f64: The gate time.
    ///
    /// Raises:
    ///     ValueError: The gate is not available in the device.
    pub fn multi_qubit_gate_time(&self, hqslang: &str, qubits: Vec<usize>) -> PyResult<f64> {
        self.internal
            .multi_qubit_gate_time(hqslang, &qubits)
            .ok_or_else(|| PyValueError::new_err("The gate is not available on the device."))
    }
}
