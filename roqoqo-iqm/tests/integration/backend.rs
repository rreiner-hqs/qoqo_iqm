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
use roqoqo_iqm::devices::DemoDevice;
use roqoqo_iqm::Backend;
use std::env;
use std::f64::consts::PI;

#[test]
fn init_backend() {
    let device = DemoDevice::new();
    if env::var("IQM_TOKENS_FILE").is_ok() {
        let ok = Backend::new(device.into(), None).is_ok();
        assert!(ok);
    } else {
        let ok = Backend::new(device.into(), None).is_err();
        assert!(ok);
        let device = DemoDevice::new();
        let ok = Backend::new(device.into(), Some("dummy_access_token".to_string())).is_ok();
        assert!(ok);
    }
}

#[test]
fn run_circuit_single_measurements() {
    if env::var("IQM_TOKENS_FILE").is_ok() {
        let device = DemoDevice::new();
        let backend = Backend::new(device.into(), None).unwrap();
        let mut qc = Circuit::new();

        qc += ControlledPauliZ::new(0, 2);
        qc += ControlledPauliZ::new(3, 2);
        qc += RotateXY::new(2, 1.0.into(), 1.0.into());
        qc += DefinitionBit::new("my_reg1".to_string(), 4, true);
        qc += DefinitionBit::new("my_reg2".to_string(), 2, true);
        qc += MeasureQubit::new(2, "my_reg1".to_string(), 2);
        qc += MeasureQubit::new(3, "my_reg1".to_string(), 3);
        qc += MeasureQubit::new(1, "my_reg2".to_string(), 1);

        let (bit_registers, _float_registers, _complex_registers) =
            backend.run_circuit(&qc).unwrap();
        assert!(bit_registers.contains_key("my_reg1"));
        assert!(bit_registers.contains_key("my_reg2"));

        // NOTE
        // For now, only the entries of the output registers that are written on are present in the
        // output registers returned by the backend. In this example, the length of my_reg1 should
        // be 2 because there are only two single-qubit measurements that actually write on this
        // register (even though the register is defined by DefinitionBit as having 4 entries).
        // This could change when we introduce pairings between qubit numbers and positions in the
        // output registers in the backend.
        let out_reg = bit_registers.get("my_reg1").unwrap();
        assert_eq!(out_reg[0].len(), 2);
        let out_reg = bit_registers.get("my_reg2").unwrap();
        assert_eq!(out_reg[0].len(), 1);
    }
}

#[test]
#[ignore]
fn run_circuit_repeated_measurements() {
    if env::var("IQM_TOKENS_FILE").is_ok() {
        let device = DemoDevice::new();
        let backend = Backend::new(device.into(), None).unwrap();
        let mut qc = Circuit::new();

        // Pauli X gate
        qc += RotateXY::new(0, PI.into(), 0.0.into());
        qc += DefinitionBit::new("my_reg".to_string(), 2, true);
        qc += PragmaRepeatedMeasurement::new("my_reg".to_string(), 10, None);

        let (bit_registers, _float_registers, _complex_registers) =
            backend.run_circuit(&qc).unwrap();
        let out_reg = bit_registers.get("my_reg").unwrap();
        let expected_output = vec![vec![true, false, false, false, false]; 10];

        assert!(bit_registers.contains_key("my_reg"));
        assert_eq!(*out_reg, expected_output);
    }
}

#[test]
#[should_panic]
fn disconnected_qubits() {
    let device = DemoDevice::new();
    let backend = Backend::new(device.into(), None).unwrap();
    let mut qc = Circuit::new();

    qc += ControlledPauliZ::new(0, 1);
    qc += DefinitionBit::new("my_reg".to_string(), 2, true);
    qc += PragmaRepeatedMeasurement::new("my_reg".to_string(), 10, None);

    let (_bit_registers, _float_registers, _complex_registers) = backend.run_circuit(&qc).unwrap();
}

#[test]
#[should_panic]
fn too_many_qubits() {
    let device = DemoDevice::new();
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
    qc += ControlledPauliZ::new(0, 1);
    qc += DefinitionBit::new("ro".to_string(), 2, true);
    qc += MeasureQubit::new(0, "ro".to_string(), 0);
    qc += PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);

    let device = DemoDevice::new();
    let backend = Backend::new(device.into(), None).unwrap();

    let (_bit_registers, _float_registers, _complex_registers) = backend.run_circuit(&qc).unwrap();
}
