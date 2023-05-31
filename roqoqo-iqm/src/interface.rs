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

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use qoqo_calculator::CalculatorFloat;
use roqoqo::operations::*;
use roqoqo::registers::BitOutputRegister;
use roqoqo::RoqoqoBackendError;

// Pragma operations that are ignored by backend and do not throw an error
const ALLOWED_OPERATIONS: &[&str; 8] = &[
    "PragmaBoostNoise",
    "PragmaStopParallelBlock",
    "PragmaGlobalPhase",
    "InputSymbolic",
    "InputBit",
    "PragmaRepeatedMeasurement",
    "PragmaStartDecompositionBlock",
    "PragmaStopDecompositionBlock",
];

/// Convert a qubit number into the format accepted by IQM.
// e.g. "QB2" for qoqo_qubit number 1 (IQM qubits start from 1)
#[inline]
fn _convert_qubit_name_qoqo_to_iqm(qoqo_qubit: usize) -> String {
    format!("QB{}", qoqo_qubit + 1)
}

/// Create a vector will all qubit names, in the format accepted by IQM
#[inline]
fn _convert_all_qubit_names(number_qubits: usize) -> Vec<String> {
    let mut qubit_vec = vec![];
    for i in 1..=number_qubits {
        qubit_vec.push(format!("QB{}", i))
    }
    qubit_vec
}

/// Representation for quantum circuits accepted by the IQM REST API.
///
/// roqoqo does not have a `name` identifier for quantum circuits, but it is needed when
/// submitting to the IQM backend.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct IqmCircuit {
    /// Name of the circuit
    pub name: String,
    /// Vector of instructions accepted by the IQM REST API
    pub instructions: Vec<IqmInstruction>,
    // TODO
    // pub metadata : Option<HashMap< String, String >>,
}

/// Representation for instructions accepted by the IQM REST API
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct IqmInstruction {
    /// Identifies the type of instruction, which can be a gate, a measurement or a barrier
    pub name: String,
    /// The qubits involved in the operation
    pub qubits: Vec<String>,
    /// Arguments of the instruction. They depend on the type of operation, and can hold both gate
    ///parameters or measurement names. The latter are used as register names when converting the
    /// results into roqoqo registers.
    pub args: HashMap<String, CalculatorFloat>,
}

// HashMap that associates to each register name the indices in the register that are being affected
// by measurements. These indices are saved in the order in which the measurement operations appear
// in the circuit, since this is the order in which the backend returns the results.
type RegisterMapping = HashMap<String, Vec<usize>>;

/// Converts all operations in a [roqoqo::Circuit] into instructions for IQM Hardware or IQM Simulators
///
/// # Arguments
///
/// `circuit` - The [roqoqo::Circuit] that is converted
///
/// `device_number_qubits` - The number of qubits of the backend device. It is used to know how many
///  qubits to measure with [roqoqo::operations::PragmaRepeatedMeasurement]
///
/// `output_registers` - A mutable reference to the classical registers that need to be initialized
///
/// # Returns
///
/// `Ok(IqmCircuit, RegisterMapping, usize)` - Converted circuit, mapping of measured qubits to register indices, and number of measurements
/// `Err(RoqoqoBackendError::OperationNotInBackend)` - Error when [roqoqo::operations::Operation] can not be converted
pub fn call_circuit<'a>(
    circuit: impl Iterator<Item = &'a Operation>,
    device_number_qubits: usize,
    output_registers: &mut HashMap<String, BitOutputRegister>,
) -> Result<(IqmCircuit, RegisterMapping, usize), RoqoqoBackendError> {
    let mut circuit_vec: Vec<IqmInstruction> = Vec::new();
    let mut number_measurements: usize = 1;
    let mut measured_qubits: Vec<usize> = vec![];
    let mut register_mapping: RegisterMapping = HashMap::new();

    for op in circuit {
        match op {
            Operation::DefinitionBit(o) => {
                // initialize output registers with default `false` values
                if *o.is_output() {
                    output_registers
                        .insert((*o).name().to_string(), vec![vec![false; *o.length()]]);
                    register_mapping.insert((*o).name().to_string(), vec![]);
                }
            }
            Operation::MeasureQubit(o) => {
                let readout = o.readout().clone();
                measured_qubits.push(*o.qubit());

                match register_mapping.get_mut(&readout) {
                    Some(x) => x.push(*o.readout_index()),
                    None => {
                        return Err(RoqoqoBackendError::GenericError {
                            msg: "A MeasureQubit operation is writing to an undefined register."
                                .to_string(),
                        })
                    }
                }
                let mut found: bool = false;
                // Check if we already have a measurement to the same register
                // if yes, add the qubit being measured to that measurement
                for instr in &mut circuit_vec {
                    if instr.name == "measurement" {
                        let meas_readout =
                            instr
                                .args
                                .get("key")
                                .ok_or(RoqoqoBackendError::GenericError {
                                msg: "A measurement must contain a `key` entry in the `args` field"
                                    .to_string(),
                            })?;
                        if let CalculatorFloat::Str(s) = meas_readout {
                            if s == &readout {
                                found = true;
                                let iqm_qubit = _convert_qubit_name_qoqo_to_iqm(*o.qubit());
                                if !instr.qubits.contains(&iqm_qubit) {
                                    instr.qubits.push(iqm_qubit);
                                } else {
                                    return Err(RoqoqoBackendError::GenericError {
                                        msg: format!(
                                            "Qubit {} is being measured twice.",
                                            *o.qubit()
                                        ),
                                    });
                                }
                                break;
                            }
                        }
                    }
                }
                if !found {
                    // If no measurement to the same register was found, create a new IqmInstruction
                    let meas = IqmInstruction {
                        name: "measurement".to_string(),
                        qubits: vec![_convert_qubit_name_qoqo_to_iqm(*o.qubit())],
                        args: HashMap::from([("key".to_string(), CalculatorFloat::Str(readout))]),
                    };
                    circuit_vec.push(meas)
                }
            }
            Operation::PragmaSetNumberOfMeasurements(o) => {
                if number_measurements > 1 {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: "Only one repeated measurement is allowed in the circuit.".to_string(),
                    });
                }

                number_measurements = *o.number_measurements();
                let readout = o.readout().clone();

                if !output_registers.contains_key(&readout) {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: format!(
                            "PragmaSetNumberOfMeasurements writes to an undefined register {}",
                            &readout
                        ),
                    });
                } else {
                    let readout_length = match output_registers
                        .get(&readout)
                        .expect("PragmaSetNumberOfMeasurements writes to an undefined register.")
                        .first()
                    {
                        Some(v) => v.len(),
                        None => {
                            return Err(RoqoqoBackendError::GenericError {
                                msg: format!(
                                    "Output register {} has not been initialized correctly.",
                                    &readout
                                ),
                            })
                        }
                    };

                    if measured_qubits.len() > readout_length {
                        return Err(RoqoqoBackendError::GenericError {
                            msg: format!("PragmaSetNumberOfMeasurements writes to register {}, which is too small.", &readout) });
                    }

                    // remove MeasureQubit operations
                    let mut old_measurement_indices = vec![];
                    for (i, meas) in circuit_vec.iter().enumerate() {
                        if meas.name == "measurement" {
                            old_measurement_indices.push(i);
                        }
                    }
                    for i in old_measurement_indices.into_iter().rev() {
                        circuit_vec.remove(i);
                    }

                    // update register mapping with the only register specified by PragmaSetNumberOfMeasurements
                    register_mapping = HashMap::new();
                    register_mapping.insert(readout.clone(), measured_qubits.clone());

                    // add single measurement instruction for all the qubits that were measured with MeasureQubit
                    let meas = IqmInstruction {
                        name: "measurement".to_string(),
                        qubits: measured_qubits
                            .iter()
                            .map(|x| _convert_qubit_name_qoqo_to_iqm(*x))
                            .collect(),
                        args: HashMap::from([("key".to_string(), CalculatorFloat::Str(readout))]),
                    };
                    circuit_vec.push(meas)
                }
            }
            Operation::PragmaRepeatedMeasurement(o) => {
                if number_measurements > 1 {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: "Only one repeated measurement is allowed in the circuit.".to_string(),
                    });
                }
                if !measured_qubits.is_empty() {
                    return Err(RoqoqoBackendError::GenericError {
                        msg: "Some qubits are being measured twice.".to_string(),
                    });
                }

                number_measurements = *o.number_measurements();
                let readout = o.readout().clone();

                match o.qubit_mapping() {
                    None => {
                        if output_registers.contains_key(&readout) {
                            let readout_length = match output_registers
                                .get(&readout)
                                .expect("Tried to access a register that is not a key of output_registers.")
                                .first() {
                                    Some(v) => v.len(),
                                    None => return Err(RoqoqoBackendError::GenericError {
                                        msg: format!("Output register {} has not been initialized correctly.", &readout) })
                                };

                            register_mapping.insert(
                                o.readout().to_string(),
                                (0..readout_length).collect(),
                            );
                        } else {
                            return Err(RoqoqoBackendError::GenericError {
                                msg: "A PragmaRepeatedMeasurement operation is writing to an undefined register.".to_string() })
                        }
                    }
                    Some(map) => {
                        match register_mapping.get_mut(o.readout()) {
                            Some(x) => {
                                for qubit in map.keys().sorted() {
                                    x.push(map[qubit])
                                }},
                            None => return Err(RoqoqoBackendError::GenericError {
                                msg: "A PragmaRepeatedMeasurement operation is writing to an undefined register.".to_string() })
                        }
                    }
                }

                let measure_all = IqmInstruction {
                    name: "measurement".to_string(),
                    qubits: _convert_all_qubit_names(device_number_qubits),
                    args: HashMap::from([("key".to_string(), CalculatorFloat::Str(readout))]),
                };
                circuit_vec.push(measure_all);
            }
            Operation::PragmaLoop(o) => {
                let reps_ref =
                    o.repetitions()
                        .float()
                        .map_err(|_| {
                            RoqoqoBackendError::GenericError {
                        msg:
                            "Only Loops with non-symbolic repetitions are supported by the backend."
                                .to_string(),
                    }
                        })?;
                let reps = (*reps_ref) as i32;
                for _ in 0..reps {
                    for i in o.circuit().iter() {
                        if let Some(instruction) = call_operation(i)? {
                            circuit_vec.push(instruction);
                        }
                    }
                }
            }
            _ => {
                if let Some(instruction) = call_operation(op)? {
                    circuit_vec.push(instruction)
                }
            }
        };
    }

    if number_measurements > 1 {
        for (_, value) in output_registers.iter_mut() {
            *value = vec![(*value)[0].to_vec(); number_measurements];
        }
    }

    let iqm_circuit = IqmCircuit {
        // NOTE
        // circuits have to be given different names when support for circuit batches is added
        // Since for the moment we only support submission of a single circuit, the name is
        // irrelevant and is hardcoded
        name: String::from("my_qc"),
        instructions: circuit_vec,
    };

    Ok((iqm_circuit, register_mapping, number_measurements))
}

/// Converts a [roqoqo::operations::Operation] into a native instruction for IQM Hardware
///
/// # Arguments
///
/// `operation` - The [roqoqo::operations::Operation] that is converted
///
/// # Returns
///
/// `Ok(IqmInstruction)` - Converted instruction  
/// `Err(RoqoqoBackendError::OperationNotInBackend)` - Error when [roqoqo::operations::Operation] can not be converted
pub fn call_operation(operation: &Operation) -> Result<Option<IqmInstruction>, RoqoqoBackendError> {
    let mut op_parameters = HashMap::new();

    match operation {
        Operation::RotateXY(op) => {
            op_parameters.insert(
                "angle_t".to_string(),
                CalculatorFloat::Float(*op.theta().float()?),
            );
            op_parameters.insert(
                "phase_t".to_string(),
                CalculatorFloat::Float(*op.phi().float()?),
            );

            Ok(Some(IqmInstruction {
                name: "phased_rx".to_string(),
                qubits: vec![_convert_qubit_name_qoqo_to_iqm(*op.qubit())],
                args: op_parameters,
            }))
        }
        Operation::ControlledPauliZ(op) => {
            let control = _convert_qubit_name_qoqo_to_iqm(*op.control());
            let target = _convert_qubit_name_qoqo_to_iqm(*op.target());

            Ok(Some(IqmInstruction {
                name: "cz".to_string(),
                qubits: vec![control, target],
                args: op_parameters,
            }))
        }
        _ => {
            if ALLOWED_OPERATIONS.contains(&operation.hqslang()) {
                Ok(None)
            } else {
                Err(RoqoqoBackendError::OperationNotInBackend {
                    backend: "IQM",
                    hqslang: operation.hqslang(),
                })
            }
        }
    }
}
