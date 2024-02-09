// Copyright © 2020-2023 HQS Quantum Simulations GmbH. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
// or implied. See the License for the specific language governing permissions and limitations under
// the License.

use ndarray::Array2;
use roqoqo::devices::{Device, GenericDevice};

/// IQM Adonis device
///
/// A hardware device composed of six qubits each coupled to a central resonator.
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdonisDevice {
    url: String,
}

impl AdonisDevice {
    /// Create new AdonisDevice with default settings.
    pub fn new() -> Self {
        Self {
            url: "https://cocos.resonance.meetiqm.com/adonis/jobs".to_string(),
        }
    }

    /// Returns API endpoint URL of the device.
    pub fn remote_host(&self) -> String {
        self.url.clone()
    }

    /// Change API endpoint URL of the device
    ///
    /// # Arguments
    ///
    /// `new_url` - The new URL to set.
    pub fn set_endpoint_url(&mut self, new_url: String) {
        self.url = new_url
    }
}

/// Implements the Device trait for AdonisDevice.
///
/// Defines standard functions available for roqoqo-iqm devices.
impl Device for AdonisDevice {
    /// Returns the gate time of a single qubit operation if the single qubit operation is available
    /// on device.
    ///
    /// # Arguments
    ///
    /// * `hqslang` - The hqslang name of a single qubit gate.
    /// * `qubit` - The qubit the gate acts on
    ///
    /// # Returns
    ///
    /// * `Some<f64>` - The gate time.
    /// * `None` - The gate is not available on the device.
    ///
    fn single_qubit_gate_time(&self, hqslang: &str, qubit: &usize) -> Option<f64> {
        if hqslang == "RotateXY" && qubit < &self.number_qubits() {
            Some(1.0)
        } else {
            None
        }
    }

    /// Returns the gate time of a qubit-resonator operation if the operation is available on device.
    /// Note that in this method the control qubit is the actual qubit involved, while the target
    /// qubit corresponds to the central resonator of the Adonis device.
    ///
    /// # Arguments
    ///
    /// * `hqslang` - The hqslang name of a two qubit gate.
    /// * `control` - The control qubit the gate acts on
    /// * `target` - The target qubit the gate acts on
    ///
    /// # Returns
    ///
    /// * `Some<f64>` - The gate time.
    /// * `None` - The gate is not available on the device.
    ///
    fn two_qubit_gate_time(&self, hqslang: &str, control: &usize, target: &usize) -> Option<f64> {
        if target == &0_usize {
            match hqslang {
                "CZQubitResonator" => {
                    if qubit < self.number_qubits() {
                        Some(1.0)
                    } else {
                        None
                    }
                }
                "SingleExcitationLoad" => {
                    if qubit == 5 {
                        Some(1.0)
                    } else {
                        None
                    }
                }
                "SingleExcitationStore" => {
                    if qubit == 5 {
                        Some(1.0)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Returns the gate time of a three qubit operation if the three qubit operation is available
    /// on device.
    ///
    /// # Arguments
    ///
    /// * `hqslang` - The hqslang name of a three qubit gate.
    /// * `control` - The control qubit the gate acts on
    /// * `target` - The target qubit the gate acts on
    ///
    /// # Returns
    ///
    /// * `Some<f64>` - The gate time.
    /// * `None` - The gate is not available on the device.
    ///
    fn three_qubit_gate_time(
        &self,
        _hqslang: &str,
        _control_0: &usize,
        _control_1: &usize,
        _target: &usize,
    ) -> Option<f64> {
        None
    }

    /// Returns the gate time of a multi qubit operation if the multi qubit operation is available
    /// on device.
    ///
    /// # Arguments
    ///
    /// * `hqslang` - The hqslang name of a multi qubit gate.
    /// * `qubits` - The qubits the gate acts on
    ///
    /// # Returns
    ///
    /// * `Some<f64>` - The gate time.
    /// * `None` - The gate is not available on the device.
    ///
    fn multi_qubit_gate_time(&self, _hqslang: &str, _qubits: &[usize]) -> Option<f64> {
        None
    }

    /// Returns the matrix of the decoherence rates of the Lindblad equation.
    ///
    /// $$
    /// \frac{d}{dt}\rho = \sum_{i,j=0}^{2} M_{i,j} L_{i} \rho L_{j}^{\dagger} - \frac{1}{2} \{ L_{j}^{\dagger} L_i, \rho \} \\\\
    ///     L_0 = \sigma^{+} \\\\
    ///     L_1 = \sigma^{-} \\\\
    ///     L_3 = \sigma^{z}
    /// $$
    ///
    /// # Arguments
    ///
    /// * `qubit` - The qubit for which the rate matrix is returned.
    ///
    /// # Returns
    ///
    /// * `Some<Array2<f64>>` - The decoherence rates.
    /// * `None` - The qubit is not part of the device.
    ///
    fn qubit_decoherence_rates(&self, _qubit: &usize) -> Option<Array2<f64>> {
        None
    }

    /// Returns the number of qubits the device supports.
    ///
    /// # Returns
    ///
    /// The number of qubits in the device.
    ///
    fn number_qubits(&self) -> usize {
        6
    }

    /// Returns the list of pairs of qubits linked with a native two-qubit-gate in the device.
    ///
    /// A pair of qubits is considered linked by a native two-qubit-gate if the device
    /// can implement a two-qubit-gate between the two qubits without decomposing it
    /// into a sequence of gates that involves a third qubit of the device.
    /// The two-qubit-gate also has to form a universal set together with the available
    /// single qubit gates.
    ///
    /// The returned vectors is a simple, graph-library independent, representation of the
    /// undirected connectivity graph of the device. It can be used to construct the connectivity
    /// graph in a graph library of the users choice from a list of edges and can be used for
    /// applications like routing in quantum algorithms.
    ///
    /// # Returns
    ///
    /// A list (Vec) of pairs of qubits linked with a native two-qubit-gate in the device.
    ///
    fn two_qubit_edges(&self) -> Vec<(usize, usize)> {
        vec![]
    }

    /// Turns Device into GenericDevice
    ///
    /// Can be used as a generic interface for devices when a boxed dyn trait object cannot be used
    /// (for example when the interface needs to be serialized)
    ///
    /// # Note
    ///
    /// [crate::devices::GenericDevice] uses nested HashMaps to represent the most general device
    /// connectivity. The memory usage will be inefficient for devices with large qubit numbers.
    fn to_generic_device(&self) -> GenericDevice {
        let mut generic_device = GenericDevice::new(self.number_qubits());
        for qubit in 0..self.number_qubits() {
            generic_device
                .set_single_qubit_gate_time(
                    "RotateXY",
                    qubit,
                    self.single_qubit_gate_time("RotateXY", &qubit).unwrap(),
                )
                .expect("Unexpectedly failed to add single qubit gate time to generic device.");
            generic_device
                .set_single_qubit_gate_time(
                    "SingleExcitationLoad",
                    qubit,
                    self.single_qubit_gate_time("SingleExcitationLoad", &qubit)
                        .unwrap(),
                )
                .expect("Unexpectedly failed to add single qubit gate time to generic device.");
            generic_device
                .set_single_qubit_gate_time(
                    "SingleExcitationStore",
                    qubit,
                    self.single_qubit_gate_time("SingleExcitationStore", &qubit)
                        .unwrap(),
                )
                .expect("Unexpectedly failed to add single qubit gate time to generic device.");
            generic_device
                .set_single_qubit_gate_time(
                    "CZQubitResonator",
                    qubit,
                    self.single_qubit_gate_time("CZQubitResonator", &qubit)
                        .unwrap(),
                )
                .expect("Unexpectedly failed to add single qubit gate time to generic device.")
        }
        generic_device
    }
}

impl Default for AdonisDevice {
    fn default() -> Self {
        Self::new()
    }
}
