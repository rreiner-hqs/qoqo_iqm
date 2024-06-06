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

use qoqo_calculator::CalculatorFloat;
use roqoqo::{operations::*, Circuit, RoqoqoBackendError};
use roqoqo_iqm::{call_circuit, call_operation, IqmBackendError, IqmCircuit, IqmInstruction};

use std::collections::HashMap;
use std::f64::consts::PI;
use test_case::test_case;

#[test_case(
    RotateXY::new(1, PI.into(), PI.into()).into(),
    IqmInstruction {
        name : "prx".to_string(),
        qubits: vec!["QB2".to_string()],
        args : HashMap::from([
            ("angle_t".to_string(), CalculatorFloat::Float(0.5)),
            ("phase_t".to_string(), CalculatorFloat::Float(0.5))
        ]),
    };
    "Phased X Rotation")]
#[test_case(
        ControlledPauliZ::new(1, 2).into(),
        IqmInstruction {
            name : "cz".to_string(),
            qubits: vec!["QB2".to_string(), "QB3".to_string()],
            args: HashMap::new(),
        };
        "Controlled Z")]
#[test_case(
    CZQubitResonator::new(1, 0).into(),
    IqmInstruction {
        name : "cz".to_string(),
        qubits: vec!["QB2".to_string(), "COMP_R".to_string()],
        args: HashMap::new(),
    };
    "CZQubitResonator")]
#[test_case(
    SingleExcitationLoad::new(5, 0).into(),
    IqmInstruction {
        name : "move".to_string(),
        qubits: vec!["QB6".to_string(), "COMP_R".to_string()],
        args: HashMap::new(),
    };
    "SingleExcitationLoad")]
#[test_case(
    SingleExcitationStore::new(5, 0).into(),
    IqmInstruction {
        name : "move".to_string(),
        qubits: vec!["QB6".to_string(), "COMP_R".to_string()],
        args: HashMap::new(),
    };
    "SingleExcitationStore")]
fn test_passing_interface(operation: Operation, instruction: IqmInstruction) {
    let called = call_operation(&operation).unwrap().unwrap();
    assert_eq!(instruction, called);
}

#[test_case(CNOT::new(0, 1).into(); "CNOT")]
#[test_case(RotateX::new(0, 1.0.into()).into(); "RotateX")]
#[test_case(Hadamard::new(0).into(); "Hadamard")]
fn test_failure_unsupported_operation(operation: Operation) {
    let called = call_operation(&operation);
    match called {
        Err(RoqoqoBackendError::OperationNotInBackend { .. }) => {}
        _ => panic!("Not the right error"),
    }
}

#[test]
fn test_call_circuit_single_measurement() {
    let mut circuit = Circuit::new();
    let register_length = 3;
    let readout_name = "ro".to_string();
    circuit += ControlledPauliZ::new(0, 1);
    circuit += RotateXY::new(0, PI.into(), PI.into());
    circuit += DefinitionBit::new(readout_name.clone(), register_length, true);
    circuit += MeasureQubit::new(0, readout_name.clone(), 0);
    circuit += MeasureQubit::new(1, readout_name.clone(), 1);
    let res = call_circuit(circuit.iter(), 2, None, 1).unwrap().0;

    let cz_instruction = IqmInstruction {
        name: "cz".to_string(),
        qubits: vec!["QB1".to_string(), "QB2".to_string()],
        args: HashMap::new(),
    };
    let xy_instruction = IqmInstruction {
        name: "prx".to_string(),
        qubits: vec!["QB1".to_string()],
        args: HashMap::from([
            ("angle_t".to_string(), CalculatorFloat::Float(0.5)),
            ("phase_t".to_string(), CalculatorFloat::Float(0.5)),
        ]),
    };
    let meas_instruction = IqmInstruction {
        name: "measure".to_string(),
        qubits: vec!["QB1".to_string(), "QB2".to_string()],
        args: HashMap::from([(
            "key".to_string(),
            CalculatorFloat::Str(readout_name.clone()),
        )]),
    };
    let instruction_vec = vec![cz_instruction, xy_instruction, meas_instruction];

    let mut metadata = HashMap::new();
    metadata.insert(readout_name, (vec![0, 1], register_length));

    let res_expected: IqmCircuit = IqmCircuit {
        name: String::from("qc_1"),
        instructions: instruction_vec,
        metadata: Some(metadata),
    };

    assert_eq!(res, res_expected)
}

#[test]
fn test_call_circuit_single_measurement_load_store() {
    let mut circuit = Circuit::new();
    let register_length = 3;
    let readout_name = "ro".to_string();
    circuit += ControlledPauliZ::new(0, 1);
    circuit += RotateXY::new(0, PI.into(), PI.into());
    circuit += SingleExcitationStore::new(3, 0);
    circuit += SingleExcitationLoad::new(3, 0);
    circuit += DefinitionBit::new(readout_name.clone(), register_length, true);
    circuit += MeasureQubit::new(0, readout_name.clone(), 0);
    circuit += MeasureQubit::new(1, readout_name.clone(), 1);
    let res = call_circuit(circuit.iter(), 2, None, 1).unwrap().0;

    let cz_instruction = IqmInstruction {
        name: "cz".to_string(),
        qubits: vec!["QB1".to_string(), "QB2".to_string()],
        args: HashMap::new(),
    };
    let xy_instruction = IqmInstruction {
        name: "prx".to_string(),
        qubits: vec!["QB1".to_string()],
        args: HashMap::from([
            ("angle_t".to_string(), CalculatorFloat::Float(0.5)),
            ("phase_t".to_string(), CalculatorFloat::Float(0.5)),
        ]),
    };
    let load_instruction = IqmInstruction {
        name: "move".to_string(),
        qubits: vec!["QB4".to_string(), "COMP_R".to_string()],
        args: HashMap::new(),
    };
    let store_instruction = IqmInstruction {
        name: "move".to_string(),
        qubits: vec!["QB4".to_string(), "COMP_R".to_string()],
        args: HashMap::new(),
    };
    let meas_instruction = IqmInstruction {
        name: "measure".to_string(),
        qubits: vec!["QB1".to_string(), "QB2".to_string()],
        args: HashMap::from([(
            "key".to_string(),
            CalculatorFloat::Str(readout_name.clone()),
        )]),
    };
    let instruction_vec = vec![
        cz_instruction,
        xy_instruction,
        load_instruction,
        store_instruction,
        meas_instruction,
    ];
    let mut metadata = HashMap::new();
    metadata.insert(readout_name, (vec![0, 1], register_length));

    let res_expected: IqmCircuit = IqmCircuit {
        name: String::from("qc_1"),
        instructions: instruction_vec,
        metadata: Some(metadata),
    };

    assert_eq!(res, res_expected)
}

#[test]
fn test_call_circuit_repeated_measurement_passes() {
    let mut inner_circuit = Circuit::new();
    inner_circuit += ControlledPauliZ::new(0, 1);

    let mut circuit = Circuit::new();
    let register_length = 2;
    let readout_name = "ro".to_string();
    let number_measurements_expected = 100;
    circuit += ControlledPauliZ::new(0, 1);
    circuit += RotateXY::new(0, PI.into(), PI.into());
    circuit += CZQubitResonator::new(1, 0);
    circuit += SingleExcitationStore::new(5, 0);
    circuit += SingleExcitationLoad::new(5, 0);
    circuit += PragmaLoop::new(CalculatorFloat::Float(3.0), inner_circuit);
    circuit += DefinitionBit::new(readout_name.clone(), register_length, true);
    circuit += MeasureQubit::new(0, readout_name.clone(), 0);
    circuit += MeasureQubit::new(1, readout_name.clone(), 1);
    circuit +=
        PragmaSetNumberOfMeasurements::new(number_measurements_expected, readout_name.clone());

    let cz_instruction = IqmInstruction {
        name: "cz".to_string(),
        qubits: vec!["QB1".to_string(), "QB2".to_string()],
        args: HashMap::new(),
    };
    let xy_instruction = IqmInstruction {
        name: "prx".to_string(),
        qubits: vec!["QB1".to_string()],
        args: HashMap::from([
            ("angle_t".to_string(), CalculatorFloat::Float(0.5)),
            ("phase_t".to_string(), CalculatorFloat::Float(0.5)),
        ]),
    };
    let cz_qubit_resonator_instruction = IqmInstruction {
        name: "cz".to_string(),
        qubits: vec!["QB2".to_string(), "COMP_R".to_string()],
        args: HashMap::new(),
    };
    let load_instruction = IqmInstruction {
        name: "move".to_string(),
        qubits: vec!["QB6".to_string(), "COMP_R".to_string()],
        args: HashMap::new(),
    };
    let store_instruction = IqmInstruction {
        name: "move".to_string(),
        qubits: vec!["QB6".to_string(), "COMP_R".to_string()],
        args: HashMap::new(),
    };
    let meas_instruction = IqmInstruction {
        name: "measure".to_string(),
        qubits: vec!["QB1".to_string(), "QB2".to_string()],
        args: HashMap::from([(
            "key".to_string(),
            CalculatorFloat::Str(readout_name.clone()),
        )]),
    };
    let mut instruction_vec = vec![
        cz_instruction.clone(),
        xy_instruction,
        cz_qubit_resonator_instruction,
        load_instruction,
        store_instruction,
    ];
    for _ in 0..3 {
        instruction_vec.push(cz_instruction.clone());
    }
    instruction_vec.push(meas_instruction);

    let mut metadata = HashMap::new();
    metadata.insert(readout_name, (vec![0, 1], register_length));

    let res_expected: IqmCircuit = IqmCircuit {
        name: String::from("qc_1"),
        instructions: instruction_vec,
        metadata: Some(metadata),
    };
    let (res, number_measurements) = call_circuit(circuit.iter(), 2, None, 1).unwrap();

    assert_eq!(res, res_expected);
    assert_eq!(number_measurements, number_measurements_expected);
}

// test that setting multiple measurements with different numbers of measurements throws an error
#[test]
fn test_call_circuit_repeated_measurement_error() {
    let number_measurements_1 = 10;
    let number_measurements_2 = 20;

    let mut circuit = Circuit::new();

    circuit += RotateXY::new(2, 1.0.into(), 1.0.into());
    circuit += CZQubitResonator::new(1, 0);
    circuit += SingleExcitationStore::new(5, 0);
    circuit += SingleExcitationLoad::new(5, 0);
    circuit += DefinitionBit::new("reg1".to_string(), 5, true);
    circuit += DefinitionBit::new("reg2".to_string(), 7, true);
    circuit += MeasureQubit::new(2, "reg1".to_string(), 2);
    circuit += MeasureQubit::new(3, "reg1".to_string(), 3);
    circuit += MeasureQubit::new(1, "reg2".to_string(), 1);
    circuit += PragmaSetNumberOfMeasurements::new(number_measurements_1, "reg1".to_string());
    circuit += PragmaSetNumberOfMeasurements::new(number_measurements_2, "reg2".to_string());

    let err = call_circuit(circuit.iter(), 6, None, 1);
    assert!(matches!(err, Err(IqmBackendError::InvalidCircuit { .. })))
}

// test the an error is returned when a measurement operation tries to write to an undefined register
#[test]
fn test_call_circuit_undefined_register_error() {
    let mut circuit = Circuit::new();

    circuit += DefinitionBit::new("reg1".to_string(), 5, true);
    circuit += RotateXY::new(2, 1.0.into(), 1.0.into());
    circuit += CZQubitResonator::new(1, 0);
    circuit += SingleExcitationStore::new(5, 0);
    circuit += SingleExcitationLoad::new(5, 0);
    circuit += MeasureQubit::new(2, "reg2".to_string(), 2);

    let err = call_circuit(circuit.iter(), 6, None, 1);
    assert!(matches!(err, Err(IqmBackendError::InvalidCircuit { .. })));

    circuit += DefinitionBit::new("reg1".to_string(), 5, true);
    circuit += RotateXY::new(2, 1.0.into(), 1.0.into());
    circuit += CZQubitResonator::new(1, 0);
    circuit += SingleExcitationStore::new(5, 0);
    circuit += SingleExcitationLoad::new(5, 0);
    circuit += MeasureQubit::new(2, "reg1".to_string(), 2);
    circuit += PragmaSetNumberOfMeasurements::new(10, "reg2".to_string());

    let err = call_circuit(circuit.iter(), 6, None, 1);
    assert!(matches!(err, Err(IqmBackendError::InvalidCircuit { .. })));
}

// test the an error is returned when a qubit is being measured twice
#[test]
fn test_symbolic_pragma_loop_error() {
    let mut inner_circuit = Circuit::new();
    inner_circuit += RotateXY::new(2, 1.0.into(), 1.0.into());

    let mut circuit = Circuit::new();
    circuit += DefinitionBit::new("reg1".to_string(), 5, true);
    circuit += CZQubitResonator::new(1, 0);
    circuit += SingleExcitationStore::new(5, 0);
    circuit += SingleExcitationLoad::new(5, 0);

    circuit += PragmaLoop::new("repetitions".into(), inner_circuit);

    circuit += MeasureQubit::new(2, "reg1".to_string(), 2);
    circuit += MeasureQubit::new(2, "reg1".to_string(), 2);

    let err = call_circuit(circuit.iter(), 6, None, 1);
    assert!(matches!(err, Err(IqmBackendError::InvalidCircuit { .. })))
}

// test the an error is returned when a qubit is being measured twice
#[test]
fn test_qubit_measured_twice_error() {
    let mut circuit = Circuit::new();

    circuit += DefinitionBit::new("reg1".to_string(), 5, true);
    circuit += RotateXY::new(2, 1.0.into(), 1.0.into());
    circuit += CZQubitResonator::new(1, 0);
    circuit += SingleExcitationStore::new(5, 0);
    circuit += SingleExcitationLoad::new(5, 0);
    circuit += MeasureQubit::new(2, "reg1".to_string(), 2);
    circuit += MeasureQubit::new(2, "reg1".to_string(), 2);

    let err = call_circuit(circuit.iter(), 6, None, 1);
    assert!(matches!(err, Err(IqmBackendError::InvalidCircuit { .. })))
}

#[test]
fn test_call_circuit_repeated_measurements_with_mapping() {
    let mut circuit = Circuit::new();
    circuit += ControlledPauliZ::new(0, 1);
    circuit += RotateXY::new(0, 1.0.into(), 1.0.into());
    circuit += DefinitionBit::new("ro".to_string(), 2, true);
    let qubit_mapping = HashMap::from([(0, 1), (1, 0)]);
    circuit += PragmaRepeatedMeasurement::new("ro".to_string(), 3, Some(qubit_mapping));
    let ok = call_circuit(circuit.iter(), 2, None, 1).is_ok();

    assert!(ok);
}

#[test]
fn test_fail_multiple_repeated_measurements() {
    let mut circuit = Circuit::new();
    circuit += ControlledPauliZ::new(0, 1);
    circuit += DefinitionBit::new("ro".to_string(), 2, true);
    circuit += PragmaSetNumberOfMeasurements::new(5, "ro".to_string());
    circuit += PragmaRepeatedMeasurement::new("ro".to_string(), 3, None);
    let res = call_circuit(circuit.iter(), 2, None, 1);

    assert!(res.is_err());
}

#[test]
fn test_fail_overlapping_measurements() {
    let mut circuit = Circuit::new();
    circuit += ControlledPauliZ::new(0, 1);
    circuit += DefinitionBit::new("ro".to_string(), 2, true);
    circuit += MeasureQubit::new(0, "ro".to_string(), 0);
    circuit += PragmaRepeatedMeasurement::new("ro".to_string(), 3, None);
    let res = call_circuit(circuit.iter(), 2, None, 1);

    assert!(res.is_err());
}
