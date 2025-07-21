# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{flake-parts-lib, ...}: {
  imports = [
    ./rust/toolchain.fp.nix
    ./rust/packages.fp.nix
    ./rust/checks.fp.nix
    ./rust/craneLibs.fp.nix
  ];
}
