# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{
  pkgs,
  flake-parts-lib,
  config,
  inputs,
  ...
}: {
  imports = [
    ./pkgs.fp.nix
    ./systems.fp.nix
    ./inputs.fp.nix
    ./rust.fp.nix
    ./apple.fp.nix
    ./checks.fp.nix
    ./devShells.fp.nix
    ./files.fp.nix
    ./web.fp.nix
  ];

  options.perSystem = flake-parts-lib.mkPerSystemOption ({
    config,
    lib,
    pkgs,
    ...
  }: {
    options.forja.rootPath = lib.mkOption {
      type = lib.types.path;

      description = ''
        TODO
      '';
    };
  });
}
