"""Test qoqo mocked backend"""
# Copyright © 2020-2023 HQS Quantum Simulations GmbH. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
# in compliance with the License. You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software distributed under the License
# is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
# or implied. See the License for the specific language governing permissions and limitations under
# the License.
import pytest
import sys
from qoqo import operations as ops
from qoqo import Circuit
from qoqo_iqm.devices import DemoDevice
from qoqo_iqm import Backend


def test_mocked_backend():
    """Test mocked backend"""
    circuit = Circuit()
    circuit += ops.DefinitionFloat(name='ro', length=1, is_output=True)
    circuit += ops.DefinitionComplex(name='ro', length=1, is_output=True)
    circuit += ops.DefinitionBit(name='ro', length=1, is_output=True)
    circuit += ops.ControlledPauliZ(0, 1)
    circuit += ops.MeasureQubit(0, 'ro', True)

    device = DemoDevice()
    _backend = Backend(device, "")

if __name__ == '__main__':
    pytest.main(sys.argv)
