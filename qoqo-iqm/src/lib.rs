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
use pyo3::types::PyDict;
use pyo3::wrap_pymodule;

pub mod devices;
pub use devices::DemoDeviceWrapper;

mod backend;
pub use backend::BackendWrapper;

/// IQM python interface
///
/// Provides the devices that are used to execute quantum programs with the IQM backend, as well as the IQM backend.
#[pymodule]
fn qoqo_iqm(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<BackendWrapper>()?;
    module.add_class::<DemoDeviceWrapper>()?;

    let wrapper = wrap_pymodule!(devices::iqm_devices);
    module.add_wrapped(wrapper)?;

    // Adding nice imports corresponding to maturin example
    let system = PyModule::import(_py, "sys")?;
    let system_modules: &PyDict = system.getattr("modules")?.downcast()?;
    system_modules.set_item("qoqo_iqm.devices", module.getattr("iqm_devices")?)?;
    Ok(())
}
