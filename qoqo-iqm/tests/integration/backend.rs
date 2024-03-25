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

use pyo3::prelude::*;
use pyo3::Python;
use qoqo_iqm::devices;
use qoqo_iqm::BackendWrapper;
use std::env;

#[test]
fn test_creating_backend_deneb_device() {
    // Initialize python interpreter in a thread-safe way
    pyo3::prepare_freethreaded_python();

    // Test if Backend is created successfully with a dummy access token
    Python::with_gil(|py| {
        // get Python type (i.e. Python class) corresponding to DenebDeviceWrapper Rust type
        let device_type = py.get_type::<devices::DenebDeviceWrapper>();
        let device = device_type
            // Instantiate Python class
            .call0()
            .unwrap()
            .downcast::<PyCell<devices::DenebDeviceWrapper>>()
            .unwrap();
        let backend_type = py.get_type::<BackendWrapper>();
        let _backend = backend_type
            .call1((device, "DUMMY_ACCESS_TOKEN"))
            .unwrap()
            .downcast::<PyCell<BackendWrapper>>()
            .unwrap();
    });

    if env::var("IQM_TOKENS_FILE").is_ok() {
        // Test if Backend correctly retrieves access token from environment variable
        Python::with_gil(|py| {
            let device_type = py.get_type::<devices::DenebDeviceWrapper>();
            let device = device_type
                .call0()
                .unwrap()
                .downcast::<PyCell<devices::DenebDeviceWrapper>>()
                .unwrap();
            let backend_type = py.get_type::<BackendWrapper>();
            let _backend = backend_type
                .call1((device,))
                .unwrap()
                .downcast::<PyCell<BackendWrapper>>()
                .unwrap();
        })
    } else {
        // If the environment variable IQM_TOKENS_FILE is not set and an access token is not provided, creation of the Backend should fail
        Python::with_gil(|py| {
            let device_type = py.get_type::<devices::DenebDeviceWrapper>();
            let device = device_type
                .call0()
                .unwrap()
                .downcast::<PyCell<devices::DenebDeviceWrapper>>()
                .unwrap();
            let backend_type = py.get_type::<BackendWrapper>();
            let backend = backend_type.call1((device,));
            match backend {
                Err(_) => (),
                _ => panic!("Missing Access Token does not return correct error"),
            }
        })
    }
}

#[test]
fn test_creating_backend_resonator_free_device() {
    // Initialize python interpreter in a thread-safe way
    pyo3::prepare_freethreaded_python();

    // Test if Backend is created successfully with a dummy access token
    Python::with_gil(|py| {
        // get Python type (i.e. Python class) corresponding to ResonatorFreeDeviceWrapper Rust type
        let device_type = py.get_type::<devices::ResonatorFreeDeviceWrapper>();
        let device = device_type
            // Instantiate Python class
            .call0()
            .unwrap()
            .downcast::<PyCell<devices::ResonatorFreeDeviceWrapper>>()
            .unwrap();
        let backend_type = py.get_type::<BackendWrapper>();
        let _backend = backend_type
            .call1((device, "DUMMY_ACCESS_TOKEN"))
            .unwrap()
            .downcast::<PyCell<BackendWrapper>>()
            .unwrap();
    });

    if env::var("IQM_TOKENS_FILE").is_ok() {
        // Test if Backend correctly retrieves access token from environment variable
        Python::with_gil(|py| {
            let device_type = py.get_type::<devices::ResonatorFreeDeviceWrapper>();
            let device = device_type
                .call0()
                .unwrap()
                .downcast::<PyCell<devices::ResonatorFreeDeviceWrapper>>()
                .unwrap();
            let backend_type = py.get_type::<BackendWrapper>();
            let _backend = backend_type
                .call1((device,))
                .unwrap()
                .downcast::<PyCell<BackendWrapper>>()
                .unwrap();
        })
    } else {
        // If the environment variable IQM_TOKENS_FILE is not set and an access token is not provided, creation of the Backend should fail
        Python::with_gil(|py| {
            let device_type = py.get_type::<devices::ResonatorFreeDeviceWrapper>();
            let device = device_type
                .call0()
                .unwrap()
                .downcast::<PyCell<devices::ResonatorFreeDeviceWrapper>>()
                .unwrap();
            let backend_type = py.get_type::<BackendWrapper>();
            let backend = backend_type.call1((device,));
            match backend {
                Err(_) => (),
                _ => panic!("Missing Access Token does not return correct error"),
            }
        })
    }
}
