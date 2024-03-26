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
use roqoqo_iqm::Backend;
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
fn run_circuit_single_measurements() {
    if env::var("IQM_TOKEN").is_ok() {
        let device = DenebDevice::new();
        let backend = Backend::new(device.into(), None).unwrap();
        let mut qc = Circuit::new();

        qc += RotateXY::new(2, 1.0.into(), 1.0.into());
        qc += CZQubitResonator::new(1, 0);
        qc += SingleExcitationStore::new(5, 0);
        qc += SingleExcitationLoad::new(5, 0);
        qc += DefinitionBit::new("my_reg1".to_string(), 5, true);
        qc += DefinitionBit::new("my_reg2".to_string(), 7, true);
        qc += MeasureQubit::new(2, "my_reg1".to_string(), 2);
        qc += MeasureQubit::new(3, "my_reg1".to_string(), 3);
        qc += MeasureQubit::new(1, "my_reg2".to_string(), 1);

        let (bit_registers, _float_registers, _complex_registers) =
            backend.run_circuit(&qc).unwrap();
        assert!(bit_registers.contains_key("my_reg1"));
        assert!(bit_registers.contains_key("my_reg2"));
    } else {
        eprintln!("No IQM_TOKEN environment variable found.")
    }
}

// Test a deterministic circuit with repeated measurements
#[test]
fn run_circuit_repeated_measurements() {
    if env::var("IQM_TOKEN").is_ok() {
        let device = DenebDevice::new();
        let mut backend = Backend::new(device.into(), None).unwrap();
        let mut qc = Circuit::new();
        let number_measurements = 1000;

        qc += RotateXY::new(0, PI.into(), 0.0.into());
        qc += DefinitionBit::new("my_reg".to_string(), 6, true);
        qc += PragmaRepeatedMeasurement::new("my_reg".to_string(), 5, None);

        backend._overwrite_number_of_measurements(number_measurements);

        let (bit_registers, _float_registers, _complex_registers) =
            backend.run_circuit(&qc).unwrap();

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
#[should_panic]
fn disconnected_qubits() {
    let device = DenebDevice::new();
    let backend = Backend::new(device.into(), None).unwrap();
    let mut qc = Circuit::new();

    qc += CZQubitResonator::new(1, 2);
    qc += DefinitionBit::new("my_reg".to_string(), 2, true);
    qc += PragmaRepeatedMeasurement::new("my_reg".to_string(), 10, None);

    let (_bit_registers, _float_registers, _complex_registers) = backend.run_circuit(&qc).unwrap();
}

#[test]
#[should_panic]
fn too_many_qubits() {
    let device = DenebDevice::new();
    let number_qubits = device.number_qubits();
    let backend = Backend::new(device.into(), None).unwrap();
    let mut qc = Circuit::new();

    qc += RotateXY::new(number_qubits, PI.into(), 0.0.into());
    qc += DefinitionBit::new("my_reg".to_string(), 2, true);
    qc += PragmaRepeatedMeasurement::new("my_reg".to_string(), 10, None);

    let (_bit_registers, _float_registers, _complex_registers) = backend.run_circuit(&qc).unwrap();
}

#[test]
#[should_panic]
fn double_measurements() {
    let mut qc = Circuit::new();
    qc += CZQubitResonator::new(0, 1);
    qc += DefinitionBit::new("ro".to_string(), 2, true);
    qc += MeasureQubit::new(0, "ro".to_string(), 0);
    qc += PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);

    let device = DenebDevice::new();
    let backend = Backend::new(device.into(), None).unwrap();

    let (_bit_registers, _float_registers, _complex_registers) = backend.run_circuit(&qc).unwrap();
}

#[test]
fn test_overwrite_number_measurements() {
    if env::var("IQM_TOKEN").is_ok() {
        let mut qc = Circuit::new();
        qc += CZQubitResonator::new(0, 1);
        qc += DefinitionBit::new("ro".to_string(), 3, true);
        qc += PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);

        let device = DenebDevice::new();
        let mut backend = Backend::new(device.into(), None).unwrap();

        assert!(backend.number_measurements_internal.is_none());

        backend._overwrite_number_of_measurements(20);

        assert_eq!(backend.number_measurements_internal.unwrap(), 20);
    } else {
        eprintln!("No IQM_TOKEN environment variable found.")
    }
}

#[test]
fn test_overwrite_readout_register() {
    if env::var("IQM_TOKEN").is_ok() {
        let mut qc = Circuit::new();
        qc += CZQubitResonator::new(2, 0);
        qc += RotateXY::new(0, 1.0.into(), 1.0.into());
        qc += DefinitionBit::new("ro1".to_string(), 2, true);
        qc += DefinitionBit::new("ro2".to_string(), 5, true);
        qc += MeasureQubit::new(0, "ro2".to_string(), 0);
        qc += PragmaSetNumberOfMeasurements::new(2, "ro1".to_string());

        let device = DenebDevice::new();
        let backend = Backend::new(device.into(), None).unwrap();

        let (bit_registers, _float_registers, _complex_registers) =
            backend.run_circuit(&qc).unwrap();

        let expected = vec![vec![false; 5], vec![false; 5]];
        let result = bit_registers.get("ro2").unwrap();
        assert_eq!(expected, *result)
    } else {
        eprintln!("No IQM_TOKEN environment variable found.")
    }
}
