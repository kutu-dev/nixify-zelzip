# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{...}: {
  perSystem = {
    pkgs,
    config,
    ...
  }: let
    rootPath = config.forja.rootPath;
  in {
    checks = {
      addLicense =
        pkgs.runCommand "checkAddLicense" {
        } ''
          mkdir -p "$out"
          cd ${rootPath} || exit

          ${pkgs.addlicense}/bin/addlicense -s -l mpl -check .
        '';

      alejandra = pkgs.runCommand "checkAlejandra" {} ''
        mkdir -p "$out"
        cd ${rootPath} || exit

        ${pkgs.alejandra}/bin/alejandra  --check .
      '';
    };
  };
}
