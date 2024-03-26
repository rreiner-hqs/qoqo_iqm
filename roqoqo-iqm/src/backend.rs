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

use crate::devices::IqmDevice;
use crate::interface::{call_circuit, IqmCircuit};

use roqoqo::backends::{EvaluatingBackend, RegisterResult};
use roqoqo::devices::Device;
use roqoqo::operations::*;
use roqoqo::registers::{BitOutputRegister, ComplexOutputRegister, FloatOutputRegister};
use roqoqo::{Circuit, RoqoqoBackendError};

use std::collections::HashMap;
use std::env::var;
use std::error::Error;
use std::time::{Duration, Instant};
use std::{fmt, thread};

use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

// Timeout for querying the REST API for results
const TIMEOUT_SECS: f64 = 60.0;
// Time interval between REST API queries
const SECONDS_BETWEEN_CALLS: f64 = 1.0;

type IqmMeasurementResult = HashMap<String, Vec<Vec<u16>>>;

// Helper function to convert the IQM result format into the classical register format used by
// Roqoqo. This involved changing 1 to `true` and 0 to `false`, and replacing the corresponding entry in
// the classical output registers which have been initialized with only `false` entries.
#[inline]
fn _results_to_registers(
    r: IqmMeasurementResult,
    measured_qubits_map: HashMap<String, Vec<usize>>,
    output_registers: &mut HashMap<String, BitOutputRegister>,
) -> Result<(), RoqoqoBackendError> {
    for (reg, reg_result) in r.iter() {
        let measured_qubits =
            measured_qubits_map
                .get(reg)
                .ok_or(RoqoqoBackendError::GenericError {
                    msg: "Backend results contain registers that are not present in the \
                      measured_qubits_map."
                        .to_string(),
                })?;

        let output_values =
            output_registers
                .get_mut(reg)
                .ok_or(RoqoqoBackendError::GenericError {
                msg: "Backend results contain registers that are not present in the BitRegisters \
                      initialized by the Definition operations."
                    .to_string(),
            })?;

        for (i, shot_result) in reg_result.iter().enumerate() {
            for (j, qubit) in measured_qubits.iter().enumerate() {
                output_values[i][*qubit] ^= shot_result[j] != 0
            }
        }
    }
    Ok(())
}

#[inline]
fn _construct_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    // The purpose of this header is to allow the client to check if the server is ready to receive
    // the request before actually sending the request data.
    headers.insert("Expect", HeaderValue::from_str("100-Continue").unwrap());
    headers.insert(
        "User-Agent",
        HeaderValue::from_str("qoqo-iqm client").unwrap(),
    );
    let token_header = &["Bearer", token].join(" ");
    headers.insert(
        "Authorization",
        HeaderValue::from_str(token_header).unwrap(),
    );
    headers
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
struct SingleQubitMapping {
    logical_name: String,
    physical_name: String,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
struct IqmRunRequest {
    circuits: Vec<IqmCircuit>,
    #[serde(default)]
    custom_settings: Option<HashMap<String, String>>, // TODO: CHECK THIS
    #[serde(default)]
    calibration_set_id: Option<String>,
    #[serde(default)]
    qubit_mapping: Option<Vec<SingleQubitMapping>>,
    shots: u16,
    #[serde(default)]
    max_circuit_duration_over_t2: Option<f64>,
    heralding_mode: HeraldingMode,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
struct ResponseBody {
    id: String,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
struct AbortResponse {
    detail: String,
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    #[serde(rename = "pending compilation")]
    PendingCompilation,
    #[serde(rename = "pending execution")]
    PendingExecution,
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "aborted")]
    Aborted,
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum HeraldingMode {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "zeros")]
    Zeros,
}

/// Measurement results from a single circuit. For each measurement operation in the circuit, maps
/// the measurement key to the corresponding results. The outer Vec elements correspond to shots,
/// and the inner Vec elements to the qubits measured in the measurement operation and the
/// respective outcomes.
type CircuitResult = HashMap<String, Vec<Vec<u16>>>;
type BatchResult = Vec<CircuitResult>;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
struct Metadata {
    // #[serde(default)]
    // calibration_set_id: Option<String>,
    request: IqmRunRequest,
    // #[serde(default)]
    // cocos_version: Option<String>,
    // #[serde(default)]
    // timestamps: Option<HashMap<String, String>>,
}

/// Representation of the HTML response from the backend.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct IqmRunResult {
    /// Status of the job
    status: Status,
    /// Measurement results, if status is Ready
    #[serde(default)]
    measurements: Option<BatchResult>,
    /// Message if status is Failed
    #[serde(default)]
    message: Option<String>,
    /// Metadata associated with the request
    metadata: Metadata,
    /// Warnings from the IQM device
    #[serde(default)]
    warnings: Option<Vec<String>>,
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
struct IqmRunStatus {
    status: Status,
    message: Option<String>,
    warnings: Option<Vec<String>>,
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
struct Token {
    pid: u64,
    timestamp: String,
    refresh_status: String,
    access_token: String,
    refresh_token: String,
    auth_server_url: String,
}

#[derive(Debug, Clone)]
struct TokenError {
    msg: String,
}
impl Error for TokenError {}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

fn _get_token_from_env_var() -> Result<String, TokenError> {
    let token: String = var("IQM_TOKEN").map_err(|_| TokenError {
        msg: "Could not retrieve token from environment variable IQM_TOKEN.".to_string(),
    })?;
    Ok(token)
}

// Helper function to get number of qubits in a qoqo Circuit
fn _get_number_qubits(qc: &Circuit) -> Option<usize> {
    let mut number_qubits_vec: Vec<usize> = vec![];
    for op in qc.iter() {
        if let InvolvedQubits::Set(s) = op.involved_qubits() {
            if let Some(x) = s.iter().max() {
                number_qubits_vec.push(*x)
            }
        }
    }
    number_qubits_vec.iter().max().map(|x| x + 1)
}

/// IQM backend
///
/// Provides functions to run circuits and measurements on IQM devices.
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Backend {
    /// IQM device used by the backend
    pub device: IqmDevice,
    /// OAuth access token for authentication
    access_token: String,
    /// Number of measurements
    pub number_measurements_internal: Option<usize>,
}

impl Backend {
    /// Creates a new IQM backend.
    ///
    /// # Arguments
    ///
    /// * `device` - The IQM device the Backend uses to execute operations and circuits.
    /// * `access_token` - An access_token is required to access IQM hardware and simulators. The
    ///                    access_token can either be passed as an argument, or if the argument is set to None will be
    ///                    read from the environmental variable `IQM_TOKEN`.
    ///
    /// # Returns
    ///
    /// * `Ok(Backend)` - The newly created IQM backend
    /// * `Err(RoqoqoBackendError)` - If the access token cannot be retrieved from the `IQM_TOKEN` environment variable.
    pub fn new(
        device: IqmDevice,
        access_token: Option<String>,
    ) -> Result<Self, RoqoqoBackendError> {
        let access_token_internal: String = match access_token {
            Some(s) => s,
            None => _get_token_from_env_var().map_err(|_| {
                RoqoqoBackendError::MissingAuthentification {
                    msg: "IQM access token has not been passed as an argument and could \
                         not be retrieved from the IQM_TOKEN environment variable."
                        .to_string(),
                }
            })?,
        };

        Ok(Self {
            device,
            access_token: access_token_internal,
            number_measurements_internal: None,
        })
    }

    /// Overwrite the number of measurements that will be executed on the [roqoqo::Circuit] or the
    /// [roqoqo::QuantumProgram]. The default number of measurements is the one defined in the submitted
    /// circuits.
    ///
    /// WARNING: this function will overwrite the number of measurements set in a Circuit or
    /// QuantumProgram. Changing the number of measurments WILL change the accuracy of the result.
    pub fn _overwrite_number_of_measurements(&mut self, number_measurements: usize) {
        self.number_measurements_internal = Some(number_measurements)
    }

    /// Check that the device's connectivity is respected.
    ///
    /// # Arguments
    ///
    /// * `circuit` - The [roqoqo::Circuit] to be checked
    ///
    /// # Returns
    ///
    /// *`Err(RoqoqoBackendError)` - The circuit is invalid.
    pub fn validate_circuit_connectivity(
        &self,
        circuit: &Circuit,
    ) -> Result<(), RoqoqoBackendError> {
        let allowed_measurement_ops = [
            "PragmaSetNumberOfMeasurements",
            "PragmaRepeatedMeasurement",
            "MeasureQubit",
            "DefinitionBit",
            "InputBit",
        ];

        for op in circuit.iter() {
            if let Ok(inner_op) = SingleQubitOperation::try_from(op) {
                if self
                    .device
                    .single_qubit_gate_time(inner_op.hqslang(), inner_op.qubit())
                    .is_none()
                {
                    return Err(RoqoqoBackendError::OperationNotInBackend {
                        backend: "IQM",
                        hqslang: inner_op.hqslang(),
                    });
                }
            } else if let Ok(inner_op) = TwoQubitOperation::try_from(op) {
                if self
                    .device
                    .two_qubit_gate_time(inner_op.hqslang(), inner_op.control(), inner_op.target())
                    .is_none()
                {
                    return Err(RoqoqoBackendError::OperationNotInBackend {
                        backend: "IQM",
                        hqslang: inner_op.hqslang(),
                    });
                }
            } else if let Ok(inner_op) = MultiQubitOperation::try_from(op) {
                if self
                    .device
                    .multi_qubit_gate_time(inner_op.hqslang(), inner_op.qubits())
                    .is_none()
                {
                    return Err(RoqoqoBackendError::OperationNotInBackend {
                        backend: "IQM",
                        hqslang: inner_op.hqslang(),
                    });
                }
            } else if !allowed_measurement_ops.contains(&op.hqslang()) {
                return Err(RoqoqoBackendError::OperationNotInBackend {
                    backend: "IQM",
                    hqslang: op.hqslang(),
                });
            }
        }
        Ok(())
    }

    /// Check if the circuit is well-defined according to the device specifications.
    ///
    /// # Arguments
    ///
    /// * `circuit` - The [roqoqo::Circuit] to be checked
    pub fn validate_circuit(&self, circuit: &Circuit) -> Result<(), RoqoqoBackendError> {
        // Check that the circuit doesn't contain more qubits than the device supports
        let mut measured_qubits: Vec<usize> = vec![];
        let number_qubits = match _get_number_qubits(circuit) {
            Some(x) => x,
            None => {
                return Err(RoqoqoBackendError::GenericError {
                    msg: "Empty circuit was passed to the backend.".to_string(),
                })
            }
        };

        if let IqmDevice::DenebDevice(device) = &self.device {
            device.validate_circuit(circuit)?
        } else {
            self.validate_circuit_connectivity(circuit)?
        }

        // Check that
        // 1) Every qubit is only measured once
        // 2) Output registers are large enough
        for op in circuit.iter() {
            match op {
                Operation::MeasureQubit(o) => {
                    let qubit = *o.qubit();
                    if measured_qubits.contains(&qubit) {
                        return Err(RoqoqoBackendError::GenericError {
                            msg: format!("Qubit {} is being measured more than once.", &qubit),
                        });
                    } else {
                        measured_qubits.push(qubit)
                    }
                }
                Operation::PragmaRepeatedMeasurement(o) => {
                    if !measured_qubits.is_empty() {
                        return Err(RoqoqoBackendError::GenericError {
                            msg: "Qubits are being measured more than once. When using \
                                PragmaRepeatedMeasurement, there should not be individual qubit \
                                measurements, and the PragmaRepeatedMeasurement operation can \
                                appear only once in the circuit."
                                .to_string(),
                        });
                    } else {
                        measured_qubits.extend(0..self.device.number_qubits())
                    }

                    let mut readout_length: usize = 0;
                    for def in circuit.definitions() {
                        if let Operation::DefinitionBit(reg) = def {
                            readout_length = *reg.length()
                        }
                    }

                    if number_qubits > readout_length {
                        return Err(RoqoqoBackendError::GenericError {
                            msg: format!("Readout register {} is not large enough.", o.readout()),
                        });
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }

    /// Query results of a submitted job.
    ///
    /// # Arguments
    ///
    /// * `id` - The job ID for the query.
    ///
    /// # Returns
    ///
    /// * `Ok(IqmRunResult)` - Result of the job (status can be pending).
    /// * `Err(RoqoqoBackendError)` - If something goes wrong with HTML requests or response is not formatted correctly.
    pub fn get_results(&self, id: &str) -> Result<IqmRunResult, RoqoqoBackendError> {
        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .build()
            .map_err(|x| RoqoqoBackendError::NetworkError {
                msg: format!("could not create https client {:?}", x),
            })?;

        let job_url = self.device.remote_host() + "/" + id;

        let result = client
            .get(job_url)
            .headers(_construct_headers(&self.access_token))
            .send()
            .map_err(|e| RoqoqoBackendError::NetworkError {
                msg: format!("Error during GET request: {:?}", e),
            })?;

        let iqm_result = result.json::<IqmRunResult>();
        let iqm_result = match iqm_result {
            Ok(res) => res,
            Err(e) => {
                return Err(RoqoqoBackendError::NetworkError {
                    msg: format!("Error during deserialisation of GET response: {:?}", e),
                });
            }
        };

        if iqm_result.warnings.is_some() {
            eprintln!("Warnings: {:?}", iqm_result.clone().warnings.unwrap());
        }

        if iqm_result.status == Status::Failed {
            return Err(RoqoqoBackendError::GenericError {
                msg: format!(
                    "Job FAILED with job ID: {}\nMessage: {}",
                    id,
                    iqm_result.message.unwrap()
                ),
            });
        }

        Ok(iqm_result)
    }

    /// Poll results until job is either ready, failed or timed out.
    ///
    /// # Arguments
    ///
    /// * `id` - The job ID for the query
    ///
    /// # Returns
    ///
    /// * `Ok(IqmMeasurementResult)` - Result of the job if ready.
    /// * `Err(RoqoqoBackendError)` - If job failed or timed out, or if there was an error retrieving. the results
    pub fn wait_for_results(&self, id: &str) -> Result<IqmMeasurementResult, RoqoqoBackendError> {
        let start_time = Instant::now();
        while start_time.elapsed().as_secs_f64() < TIMEOUT_SECS {
            let iqm_result: IqmRunResult = self.get_results(id)?;
            if iqm_result.status == Status::Ready {
                match iqm_result.measurements {
                    Some(x) => match x.first() {
                        Some(y) => return Ok(y.clone()),
                        None => {
                            return Err(RoqoqoBackendError::GenericError {
                                msg: "IQM backend returned empty measurement results".to_string(),
                            })
                        }
                    },
                    None => {
                        return Err(RoqoqoBackendError::GenericError {
                            msg: "IQM backend returned empty measurement results".to_string(),
                        })
                    }
                };
            }
            let duration = Duration::from_secs_f64(SECONDS_BETWEEN_CALLS);
            thread::sleep(duration);
        }
        Err(RoqoqoBackendError::Timeout {
            msg: format!("Job did not finish in {} seconds", TIMEOUT_SECS),
        })
    }

    /// Abort a submitted job.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the job to abort.
    ///
    /// # Returns
    ///
    /// * `Err(RoqoqoBackendError)` - If the job abortion failed.
    pub fn abort_job(&self, id: &str) -> Result<(), RoqoqoBackendError> {
        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .build()
            .map_err(|x| RoqoqoBackendError::NetworkError {
                msg: format!("could not create https client {:?}", x),
            })?;

        let abort_url = [&self.device.remote_host(), "jobs", id, "abort"].join("/");

        let resp = client
            .post(abort_url)
            .headers(_construct_headers(&self.access_token))
            .send()
            .map_err(|e| RoqoqoBackendError::NetworkError {
                msg: format!("Error during POST request for abort_job: {:?}", e),
            })?;

        match resp.status() {
            reqwest::StatusCode::OK => Ok(()),
            _ => {
                let abort_failed_msg: &str =
                    &serde_json::from_str::<AbortResponse>(&resp.text().unwrap())
                        .unwrap()
                        .detail;
                Err(RoqoqoBackendError::GenericError {
                    msg: format!("Job abortion failed: {}", abort_failed_msg),
                })
            }
        }
    }

    /// Get information about the quantum architecture of the given device.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Information about the quantum architecture of the device.
    /// * `Err(RoqoqoBackendError)` - Error response from IQM server.
    pub fn get_quantum_architecture(&self) -> Result<String, RoqoqoBackendError> {
        let endpoint_url = self
            .device
            .remote_host()
            .replace("jobs", "quantum-architecture");

        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .build()
            .map_err(|x| RoqoqoBackendError::NetworkError {
                msg: format!("Could not create https client {:?}", x),
            })?;

        let response = client
            .get(endpoint_url)
            .headers(_construct_headers(&self.access_token))
            .send()
            .map_err(|e| RoqoqoBackendError::NetworkError {
                msg: format!("Error during GET request: {:?}", e),
            })?;
        if response.status().is_success() {
            response
                .text()
                .map_err(|e| RoqoqoBackendError::NetworkError {
                    msg: format!("Error during GET request: {:?}", e),
                })
        } else {
            Err(RoqoqoBackendError::NetworkError {
                msg: format!(
                    "GET request failed with status code: {:?}",
                    response.status()
                ),
            })
        }
    }
}

impl EvaluatingBackend for Backend {
    fn run_circuit(&self, circuit: &Circuit) -> RegisterResult {
        self.validate_circuit(circuit)?;
        self.run_circuit_iterator(circuit.iter())
    }
    fn run_circuit_iterator<'a>(
        &self,
        circuit: impl Iterator<Item = &'a Operation>,
    ) -> RegisterResult {
        let mut bit_registers: HashMap<String, BitOutputRegister> = HashMap::new();
        let float_registers: HashMap<String, FloatOutputRegister> = HashMap::new();
        let complex_registers: HashMap<String, ComplexOutputRegister> = HashMap::new();

        let (iqm_circuit, register_mapping, number_measurements) = call_circuit(
            circuit,
            self.device.number_qubits(),
            &mut bit_registers,
            self.number_measurements_internal,
        )?;

        let data = IqmRunRequest {
            circuits: vec![iqm_circuit],
            shots: number_measurements as u16,
            custom_settings: None,
            calibration_set_id: None,
            qubit_mapping: None,
            max_circuit_duration_over_t2: None,
            heralding_mode: HeraldingMode::None,
        };

        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .build()
            .map_err(|x| RoqoqoBackendError::NetworkError {
                msg: format!("Could not create https client {:?}", x),
            })?;

        let resp = client
            .post(self.device.remote_host())
            .headers(_construct_headers(&self.access_token))
            .json(&data)
            .send()
            .map_err(|e| RoqoqoBackendError::NetworkError {
                msg: format!("Error during POST request: {:?}", e),
            })?;

        let status = resp.status();
        match status {
            reqwest::StatusCode::OK => (),
            reqwest::StatusCode::CREATED => (),
            reqwest::StatusCode::ACCEPTED => (),
            _ => {
                return Err(RoqoqoBackendError::GenericError {
                    msg: format!(
                        "Received an error response with HTTP status code: {}",
                        status
                    ),
                });
            }
        }

        let job_id: &str = &serde_json::from_str::<ResponseBody>(&resp.text().unwrap())
            .unwrap()
            .id;

        let result_map: IqmMeasurementResult = self.wait_for_results(job_id)?;

        _results_to_registers(result_map, register_mapping, &mut bit_registers)?;

        Ok((bit_registers, float_registers, complex_registers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[inline]
    fn _convert_qubit_name_iqm_to_qoqo(name: String) -> usize {
        let qubit_number = name
            .chars()
            .last()
            .expect("Passed empty qubit name string to conversion function.")
            .to_digit(10)
            .expect("Last digit of qubit name in the IQM format should be a number.");

        qubit_number as usize - 1
    }

    #[test]
    fn test_qubit_name_conversion_iqm_to_qoqo() {
        let qubit = String::from("QB2");
        let converted_name = _convert_qubit_name_iqm_to_qoqo(qubit);

        assert_eq!(converted_name, 1)
    }

    #[test]
    fn test_get_number_qubits() {
        let mut qc = Circuit::new();

        assert!(_get_number_qubits(&qc).is_none());

        qc += RotateXY::new(0, PI.into(), 0.0.into());
        qc += RotateXY::new(2, PI.into(), 0.0.into());
        qc += RotateXY::new(6, PI.into(), 0.0.into());
        qc += DefinitionBit::new("my_reg".to_string(), 2, true);
        qc += PragmaRepeatedMeasurement::new("my_reg".to_string(), 10, None);

        assert_eq!(_get_number_qubits(&qc), Some(7))
    }

    #[test]
    fn test_results_to_registers() {
        let mut bit_registers: HashMap<String, BitOutputRegister> = HashMap::new();
        bit_registers.insert(
            "reg1".to_string(),
            vec![
                vec![false, false, false, false, false],
                vec![false, false, false, false, false],
            ],
        );
        bit_registers.insert(
            "reg2".to_string(),
            vec![vec![false, false, false], vec![false, false, false]],
        );
        let mut iqm_results = HashMap::new();
        iqm_results.insert("reg1".to_string(), vec![vec![0, 1, 0], vec![1, 1, 0]]);
        iqm_results.insert("reg2".to_string(), vec![vec![1, 1], vec![1, 0]]);
        let mut measured_qubits_map = HashMap::new();
        measured_qubits_map.insert("reg1".to_string(), vec![0, 2, 4]);
        measured_qubits_map.insert("reg2".to_string(), vec![1, 2]);
        let mut output_registers: HashMap<String, BitOutputRegister> = HashMap::new();
        output_registers.insert(
            "reg1".to_string(),
            vec![
                vec![false, false, true, false, false],
                vec![true, false, true, false, false],
            ],
        );
        output_registers.insert(
            "reg2".to_string(),
            vec![vec![false, true, true], vec![false, true, false]],
        );

        _results_to_registers(iqm_results, measured_qubits_map, &mut bit_registers).unwrap();
        assert_eq!(bit_registers, output_registers);
    }
}
