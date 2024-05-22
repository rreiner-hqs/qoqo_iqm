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

use roqoqo::devices::Device;
use roqoqo::prelude::*;
use roqoqo::{operations::*, Circuit};
use roqoqo_iqm::devices::DenebDevice;
use roqoqo_iqm::{Backend, GarnetDevice, IqmBackendError};
use std::env;
use std::f64::consts::PI;

#[test]
fn init_backend() {
    let device = DenebDevice::new();
    if env::var("IQM_TOKEN").is_ok() {
        let ok = Backend::new(device.into(), None).is_ok();
        assert!(ok);
    } else {
        let ok = Backend::new(device.into(), None).is_err();
        assert!(ok);
        let device = DenebDevice::new();
        let ok = Backend::new(device.into(), Some("dummy_access_token".to_string())).is_ok();
        assert!(ok);
    }
}

#[test]
fn test_register_initialization() {
    if env::var("IQM_TOKEN").is_ok() {
        let device = DenebDevice::new();
        let backend = Backend::new(device.into(), None).unwrap();

        let mut qc = Circuit::new();
        qc += CZQubitResonator::new(5, 0);
        qc += DefinitionBit::new("my_reg1".to_string(), 5, true);
        qc += DefinitionBit::new("my_reg2".to_string(), 7, true);
        qc += MeasureQubit::new(2, "my_reg1".to_string(), 2);

        let (bit_registers, _float_registers, _complex_registers) =
            backend.run_circuit(&qc).unwrap();

        let reg2_result = bit_registers.get("my_reg2").unwrap();
        let expected_output = vec![vec![false; 7]];

        assert_eq!(*reg2_result, expected_output);
    } else {
        eprintln!("No IQM_TOKEN environment variable found.")
    }
}

#[test]
fn run_circuit_single_measurements_deneb_passes() {
    if env::var("IQM_TOKEN").is_ok() {
        let device = DenebDevice::new();
        let backend = Backend::new(device.into(), None).unwrap();
        let mut qc = Circuit::new();

        qc += RotateXY::new(2, 1.0.into(), 1.0.into());
        qc += CZQubitResonator::new(1, 0);
        qc += SingleExcitationStore::new(5, 0);
        qc += SingleExcitationLoad::new(5, 0);
        qc += DefinitionBit::new("reg1".to_string(), 5, true);
        qc += DefinitionBit::new("reg2".to_string(), 7, true);
        qc += MeasureQubit::new(2, "reg1".to_string(), 2);
        qc += MeasureQubit::new(3, "reg1".to_string(), 3);
        qc += MeasureQubit::new(1, "reg2".to_string(), 1);

        let (bit_registers, _float_registers, _complex_registers) =
            backend.run_circuit(&qc).unwrap();

        assert!(bit_registers.contains_key("reg1"));
        assert!(bit_registers.contains_key("reg2"));

        let res1 = bit_registers.get("reg1").unwrap();
        let res2 = bit_registers.get("reg2").unwrap();

        // check number of measurements
        assert_eq!(res1.len(), 1);
        assert_eq!(res2.len(), 1);

        // check register length
        assert_eq!(res1[0].len(), 5);
        assert_eq!(res2[0].len(), 7);
    } else {
        eprintln!("No IQM_TOKEN environment variable found.")
    }
}

#[test]
fn run_circuit_multiple_measurements_deneb_passes() {
    if env::var("IQM_TOKEN").is_ok() {
        let number_measurements = 10;
        let device = DenebDevice::new();
        let backend = Backend::new(device.into(), None).unwrap();
        let mut qc = Circuit::new();

        qc += RotateXY::new(2, 1.0.into(), 1.0.into());
        qc += CZQubitResonator::new(1, 0);
        qc += SingleExcitationStore::new(5, 0);
        qc += SingleExcitationLoad::new(5, 0);
        qc += DefinitionBit::new("reg1".to_string(), 5, true);
        qc += DefinitionBit::new("reg2".to_string(), 7, true);
        qc += MeasureQubit::new(2, "reg1".to_string(), 2);
        qc += MeasureQubit::new(3, "reg1".to_string(), 3);
        qc += MeasureQubit::new(1, "reg2".to_string(), 1);
        qc += PragmaSetNumberOfMeasurements::new(number_measurements, "reg1".to_string());
        qc += PragmaSetNumberOfMeasurements::new(number_measurements, "reg2".to_string());

        let (bit_registers, _float_registers, _complex_registers) =
            backend.run_circuit(&qc).unwrap();

        assert!(bit_registers.contains_key("reg1"));
        assert!(bit_registers.contains_key("reg2"));

        let res1 = bit_registers.get("reg1").unwrap();
        let res2 = bit_registers.get("reg2").unwrap();

        // check number of measurements
        assert_eq!(res1.len(), number_measurements);
        assert_eq!(res2.len(), number_measurements);

        // check register length
        assert_eq!(res1[0].len(), 5);
        assert_eq!(res2[0].len(), 7);
    } else {
        eprintln!("No IQM_TOKEN environment variable found.")
    }
}

#[test]
fn run_circuit_multiple_measurements_garnet_passes() {
    if env::var("IQM_TOKEN").is_ok() {
        let number_measurements = 10;
        let device = GarnetDevice::new();
        let backend = Backend::new(device.into(), None).unwrap();
        let mut qc = Circuit::new();

        qc += RotateXY::new(2, 1.0.into(), 1.0.into());
        qc += ControlledPauliZ::new(0, 1);
        qc += DefinitionBit::new("reg1".to_string(), 5, true);
        qc += DefinitionBit::new("reg2".to_string(), 7, true);
        qc += MeasureQubit::new(2, "reg1".to_string(), 2);
        qc += MeasureQubit::new(3, "reg1".to_string(), 3);
        qc += MeasureQubit::new(1, "reg2".to_string(), 1);
        qc += PragmaSetNumberOfMeasurements::new(number_measurements, "reg1".to_string());
        qc += PragmaSetNumberOfMeasurements::new(number_measurements, "reg2".to_string());

        let (bit_registers, _float_registers, _complex_registers) =
            backend.run_circuit(&qc).unwrap();

        assert!(bit_registers.contains_key("reg1"));
        assert!(bit_registers.contains_key("reg2"));

        let res1 = bit_registers.get("reg1").unwrap();
        let res2 = bit_registers.get("reg2").unwrap();

        // check number of measurements
        assert_eq!(res1.len(), number_measurements);
        assert_eq!(res2.len(), number_measurements);

        // check register length
        assert_eq!(res1[0].len(), 5);
        assert_eq!(res2[0].len(), 7);
    } else {
        eprintln!("No IQM_TOKEN environment variable found.")
    }
}

#[test]
fn run_circuit_batch_single_measurement_garnet_passes() {
    if env::var("IQM_TOKEN").is_ok() {
        let number_measurements = 1;
        let device = GarnetDevice::new();
        let backend = Backend::new(device.into(), None).unwrap();

        let mut qc1 = Circuit::new();
        qc1 += RotateXY::new(2, 1.0.into(), 1.0.into());
        qc1 += ControlledPauliZ::new(0, 1);
        qc1 += DefinitionBit::new("reg1".to_string(), 5, true);
        qc1 += MeasureQubit::new(2, "reg1".to_string(), 2);
        qc1 += MeasureQubit::new(3, "reg1".to_string(), 3);

        let mut qc2 = Circuit::new();
        qc2 += RotateXY::new(2, 1.0.into(), 1.0.into());
        qc2 += ControlledPauliZ::new(0, 1);
        qc2 += DefinitionBit::new("reg2".to_string(), 5, true);
        qc2 += MeasureQubit::new(2, "reg2".to_string(), 2);
        qc2 += MeasureQubit::new(3, "reg2".to_string(), 3);

        let batch = vec![qc1, qc2];
        let (bit_registers, _, _) = backend.run_circuit_batch(&batch).unwrap();

        assert!(bit_registers.contains_key("reg1"));
        assert!(bit_registers.contains_key("reg2"));

        let res1 = bit_registers.get("reg1").unwrap();
        let res2 = bit_registers.get("reg2").unwrap();

        // check number of measurements
        assert_eq!(res1.len(), number_measurements);
        assert_eq!(res2.len(), number_measurements);

        // check register length
        assert_eq!(res1[0].len(), 5);
        assert_eq!(res2[0].len(), 7);
    } else {
        eprintln!("No IQM_TOKEN environment variable found.")
    }
}

// Test that an error is returned when different circuits in the batch write to the same output register
#[test]
fn run_circuit_batch_same_reg_error() {
    let number_measurements = 10;
    let device = GarnetDevice::new();
    let backend = Backend::new(device.into(), Some("dummy_token".to_string())).unwrap();

    let mut qc1 = Circuit::new();
    qc1 += RotateXY::new(2, 1.0.into(), 1.0.into());
    qc1 += ControlledPauliZ::new(0, 1);
    qc1 += DefinitionBit::new("reg1".to_string(), 5, true);
    qc1 += MeasureQubit::new(2, "reg1".to_string(), 2);
    qc1 += MeasureQubit::new(3, "reg1".to_string(), 3);
    qc1 += PragmaSetNumberOfMeasurements::new(number_measurements, "reg1".to_string());

    let mut qc2 = Circuit::new();
    qc2 += RotateXY::new(2, 1.0.into(), 1.0.into());
    qc2 += ControlledPauliZ::new(0, 1);
    qc2 += DefinitionBit::new("reg1".to_string(), 5, true);
    qc2 += MeasureQubit::new(2, "reg1".to_string(), 2);
    qc2 += MeasureQubit::new(3, "reg1".to_string(), 3);
    qc2 += PragmaSetNumberOfMeasurements::new(number_measurements, "reg1".to_string());

    let batch = vec![qc1, qc2];
    let err = backend.run_circuit_batch(&batch);

    assert!(matches!(err, Err(IqmBackendError::InvalidCircuit { .. })));
}

// Test that an error is returned when different circuits in the batch have different numbers of measurements
#[test]
fn run_circuit_batch_different_number_measurements_error() {
    let device = GarnetDevice::new();
    let backend = Backend::new(device.into(), Some("dummy_token".to_string())).unwrap();
    let number_measurements_1 = 10;
    let number_measurements_2 = 20;

    let mut qc1 = Circuit::new();
    qc1 += RotateXY::new(2, 1.0.into(), 1.0.into());
    qc1 += ControlledPauliZ::new(0, 1);
    qc1 += DefinitionBit::new("reg1".to_string(), 5, true);
    qc1 += MeasureQubit::new(2, "reg1".to_string(), 2);
    qc1 += MeasureQubit::new(3, "reg1".to_string(), 3);
    qc1 += PragmaSetNumberOfMeasurements::new(number_measurements_1, "reg1".to_string());

    let mut qc2 = Circuit::new();
    qc2 += RotateXY::new(2, 1.0.into(), 1.0.into());
    qc2 += ControlledPauliZ::new(0, 1);
    qc2 += DefinitionBit::new("reg2".to_string(), 5, true);
    qc2 += MeasureQubit::new(2, "reg2".to_string(), 2);
    qc2 += MeasureQubit::new(3, "reg2".to_string(), 3);
    qc2 += PragmaSetNumberOfMeasurements::new(number_measurements_2, "reg2".to_string());

    let batch = vec![qc1, qc2];
    let err = backend.run_circuit_batch(&batch);

    assert!(matches!(err, Err(IqmBackendError::InvalidCircuit { .. })));
}

// Test a deterministic circuit with repeated measurements
#[test]
fn run_circuit_repeated_measurements_deterministic() {
    if env::var("IQM_TOKEN").is_ok() {
        let device = DenebDevice::new();
        let mut backend = Backend::new(device.into(), None).unwrap();
        let mut circuit = Circuit::new();
        let number_measurements = 1000;

        circuit += RotateXY::new(0, PI.into(), 0.0.into());
        circuit += DefinitionBit::new("my_reg".to_string(), 6, true);
        circuit += PragmaRepeatedMeasurement::new("my_reg".to_string(), 5, None);

        backend._overwrite_number_of_measurements(number_measurements);

        let (bit_registers, _, _) = backend.run_circuit(&circuit).unwrap();

        assert!(bit_registers.contains_key("my_reg"));

        let shots_in_results = bit_registers.get("my_reg").unwrap().len();
        assert_eq!(shots_in_results, number_measurements);

        let result = bit_registers.get("my_reg").unwrap().clone();

        let number_of_true: usize = result.iter().map(|x| x[0]).filter(|&x| x).count();

        let threshold = (0.9 * (number_measurements as f64)).round() as usize;
        assert!(number_of_true > threshold);
    } else {
        eprintln!("No IQM_TOKEN environment variable found.")
    }
}

#[test]
fn test_submit_batch() {}

#[test]
fn disconnected_qubits_deneb() {
    let device = DenebDevice::new();
    let backend = Backend::new(device.into(), Some("dummy_token".to_string())).unwrap();
    let mut circuit = Circuit::new();

    circuit += CZQubitResonator::new(1, 2);
    circuit += DefinitionBit::new("my_reg".to_string(), 2, true);
    circuit += PragmaRepeatedMeasurement::new("my_reg".to_string(), 10, None);

    let err = backend.validate_circuit(&circuit);
    assert!(matches!(err, Err(IqmBackendError::InvalidCircuit { .. })));
}

#[test]
fn disconnected_qubits_garnet() {
    let device = GarnetDevice::new();
    let backend = Backend::new(device.into(), Some("dummy_token".to_string())).unwrap();
    let mut circuit = Circuit::new();

    circuit += ControlledPauliZ::new(1, 7);
    circuit += DefinitionBit::new("my_reg".to_string(), 2, true);
    circuit += PragmaRepeatedMeasurement::new("my_reg".to_string(), 10, None);

    let err = backend.validate_circuit(&circuit);
    assert!(matches!(err, Err(IqmBackendError::InvalidCircuit { .. })));
}

#[test]
fn too_many_qubits_deneb() {
    let device = DenebDevice::new();
    let number_qubits = device.number_qubits();
    let backend = Backend::new(device.into(), Some("dummy_token".to_string())).unwrap();
    let mut circuit = Circuit::new();

    circuit += RotateXY::new(number_qubits, PI.into(), 0.0.into());
    circuit += DefinitionBit::new("my_reg".to_string(), 10, true);
    circuit += PragmaRepeatedMeasurement::new("my_reg".to_string(), 10, None);

    let err = backend.validate_circuit(&circuit);
    assert!(matches!(err, Err(IqmBackendError::InvalidCircuit { .. })));
}

#[test]
fn too_many_qubits_garnet() {
    let device = GarnetDevice::new();
    let number_qubits = device.number_qubits();
    let backend = Backend::new(device.into(), Some("dummy_token".to_string())).unwrap();
    let mut circuit = Circuit::new();

    circuit += RotateXY::new(number_qubits, PI.into(), 0.0.into());
    circuit += DefinitionBit::new("my_reg".to_string(), 2, true);
    circuit += PragmaRepeatedMeasurement::new("my_reg".to_string(), 10, None);

    let err = backend.validate_circuit(&circuit);
    assert!(matches!(err, Err(IqmBackendError::InvalidCircuit { .. })));
}

#[test]
fn double_measurements() {
    let mut circuit = Circuit::new();
    circuit += CZQubitResonator::new(0, 1);
    circuit += DefinitionBit::new("ro".to_string(), 2, true);
    circuit += MeasureQubit::new(0, "ro".to_string(), 0);
    circuit += PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);

    let device = DenebDevice::new();
    let backend = Backend::new(device.into(), Some("dummy_token".to_string())).unwrap();

    let err = backend.validate_circuit(&circuit);
    assert!(matches!(err, Err(IqmBackendError::InvalidCircuit { .. })));
}

#[test]
fn test_overwrite_number_measurements() {
    let mut circuit = Circuit::new();
    circuit += CZQubitResonator::new(0, 1);
    circuit += DefinitionBit::new("ro".to_string(), 3, true);
    circuit += PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);

    let device = DenebDevice::new();
    let mut backend = Backend::new(device.into(), Some("dummy_token".to_string())).unwrap();

    assert!(backend.number_measurements_internal.is_none());

    backend._overwrite_number_of_measurements(20);
    assert_eq!(backend.number_measurements_internal.unwrap(), 20);
}
