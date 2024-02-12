# Copyright © 2019-2023 HQS Quantum Simulations GmbH. All Rights Reserved.
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
"""IQM backend for the qoqo quantum toolkit.

Allows the user to run qoqo Circuits on IQM quantum computer testbed.
Note that a valid IQM credentials are required to use this Backend. 

Note: At the moment this backend can only be used to test the connection
to the testbed. It will NOT return data from a valid simulation of a quantum circuit.

.. autosummary::
    :toctree: generated/

    Backend
    devices

"""

from . import *

print(
    """Note: At the moment this backend can only be used to test the connection
to the testbed. It will NOT return data from a valid simulation of a quantum circuit.
"""
)
__license__ = "Apache-2.0 for linked dependencies see qoqo_iqm/LICENSE_FOR_BINARY_DISTRIBUTION"
