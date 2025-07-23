# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{inputs, ...}: {
  imports = [
    inputs.files.flakeModules.default
    ./files/readme.fp.nix
    ./files/gitignore.fp.nix
    ./files/npmPackage.fp.nix
  ];

  perSystem = {
    pkgs,
    config,
    ...
  }: {
    packages.generateFiles = config.files.writer.drv;
  };
}
