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
use roqoqo::operations;
use roqoqo::registers::BitOutputRegister;
use roqoqo::{Circuit, RoqoqoBackendError};
use roqoqo_iqm::{call_circuit, call_operation, IqmCircuit, IqmInstruction};

use std::collections::HashMap;
use test_case::test_case;

#[test_case(
    operations::RotateXY::new(1, 1.0.into(), 1.0.into()).into(),
    IqmInstruction {
        name : "phased_rx".to_string(),
        qubits: vec!["QB2".to_string()],
        args : HashMap::from([
            ("angle_t".to_string(), CalculatorFloat::Float(1.0)),
            ("phase_t".to_string(), CalculatorFloat::Float(1.0))
        ]),
    };
    "Phased X Rotation")]
#[test_case(
    operations::ControlledPauliZ::new(1,2).into(),
    IqmInstruction {
        name : "cz".to_string(),
        qubits: vec!["QB2".to_string(), "QB3".to_string()],
        args: HashMap::new(),
    };
    "Controlled Z")]
fn test_passing_interface(operation: operations::Operation, instruction: IqmInstruction) {
    let called = call_operation(&operation).unwrap();
    assert_eq!(instruction, called);
}

#[test_case(operations::CNOT::new(0,1).into(); "CNOT")]
fn test_failure_unsupported_operation(operation: operations::Operation) {
    let called = call_operation(&operation);
    match called {
        Err(RoqoqoBackendError::OperationNotInBackend { .. }) => {}
        _ => panic!("Not the right error"),
    }
}

#[test]
fn test_call_circuit_repeated_measurement() {
    let mut bit_registers: HashMap<String, BitOutputRegister> = HashMap::new();
    let mut inner_circuit = Circuit::new();
    inner_circuit += operations::ControlledPauliZ::new(0, 1);

    let mut circuit = Circuit::new();
    circuit += operations::ControlledPauliZ::new(0, 1);
    circuit += operations::RotateXY::new(0, 1.0.into(), 1.0.into());
    circuit += operations::PragmaLoop::new(CalculatorFloat::Float(3.0), inner_circuit);
    circuit += operations::DefinitionBit::new("ro".to_string(), 2, true);
    circuit += operations::PragmaRepeatedMeasurement::new("ro".to_string(), 10, None);
    let res = call_circuit(circuit.iter(), 2, &mut bit_registers)
        .unwrap()
        .0;

    let cz_instruction = IqmInstruction {
        name: "cz".to_string(),
        qubits: vec!["QB1".to_string(), "QB2".to_string()],
        args: HashMap::new(),
    };
    let xy_instruction = IqmInstruction {
        name: "phased_rx".to_string(),
        qubits: vec!["QB1".to_string()],
        args: HashMap::from([
            ("angle_t".to_string(), CalculatorFloat::Float(1.0)),
            ("phase_t".to_string(), CalculatorFloat::Float(1.0)),
        ]),
    };
    let meas_instruction = IqmInstruction {
        name: "measurement".to_string(),
        qubits: vec!["QB1".to_string(), "QB2".to_string()],
        args: HashMap::from([("key".to_string(), CalculatorFloat::Str("ro".to_string()))]),
    };

    let mut instruction_vec = vec![];
    instruction_vec.push(cz_instruction.clone());
    instruction_vec.push(xy_instruction);
    for _ in 0..3 {
        instruction_vec.push(cz_instruction.clone());
    }
    instruction_vec.push(meas_instruction);

    let res_expected: IqmCircuit = IqmCircuit {
        name: String::from("my_qc"),
        instructions: instruction_vec,
    };

    assert_eq!(res, res_expected)
}

#[test]
fn test_call_circuit_single_measurement() {
    let mut bit_registers: HashMap<String, BitOutputRegister> = HashMap::new();

    let mut circuit = Circuit::new();
    circuit += operations::ControlledPauliZ::new(0, 1);
    circuit += operations::RotateXY::new(0, 1.0.into(), 1.0.into());
    circuit += operations::DefinitionBit::new("ro".to_string(), 2, true);
    circuit += operations::MeasureQubit::new(0, "ro".to_string(), 0);
    circuit += operations::MeasureQubit::new(1, "ro".to_string(), 1);
    let res = call_circuit(circuit.iter(), 2, &mut bit_registers)
        .unwrap()
        .0;

    let cz_instruction = IqmInstruction {
        name: "cz".to_string(),
        qubits: vec!["QB1".to_string(), "QB2".to_string()],
        args: HashMap::new(),
    };
    let xy_instruction = IqmInstruction {
        name: "phased_rx".to_string(),
        qubits: vec!["QB1".to_string()],
        args: HashMap::from([
            ("angle_t".to_string(), CalculatorFloat::Float(1.0)),
            ("phase_t".to_string(), CalculatorFloat::Float(1.0)),
        ]),
    };
    let meas_instruction = IqmInstruction {
        name: "measurement".to_string(),
        qubits: vec!["QB1".to_string(), "QB2".to_string()],
        args: HashMap::from([("key".to_string(), CalculatorFloat::Str("ro".to_string()))]),
    };

    let mut instruction_vec = vec![];
    instruction_vec.push(cz_instruction);
    instruction_vec.push(xy_instruction);
    instruction_vec.push(meas_instruction);

    let res_expected: IqmCircuit = IqmCircuit {
        name: String::from("my_qc"),
        instructions: instruction_vec,
    };

    assert_eq!(res, res_expected)
}
