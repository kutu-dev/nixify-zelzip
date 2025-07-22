# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{
  flake-parts-lib,
  lib,
  ...
}: {
  options.perSystem = flake-parts-lib.mkPerSystemOption ({config, ...}: {
    options.forja.inputs = {
      nixpkgs = lib.mkOption {
        type = lib.types.package;

        description = ''
          The resolved input of Nixpkgs.
          Project homepage: "https://github.com/NixOS/nixpkgs/".
        '';
      };

      rustOverlay = lib.mkOption {
        type = lib.types.package;

        description = ''
          The resolved input of the "rust-overlay" Rust toolchain manager.
          Project homepage: "https://github.com/oxalica/rust-overlay".
        '';
      };

      crane = lib.mkOption {
        type = lib.types.package;

        description = ''
          The resolved input of the Crane Nix library.
          Project homepage: "https://github.com/ipetkov/crane".
        '';
      };

      files = lib.mkOption {
        type = lib.types.package;

        description = ''
          The resolved input of the Files Nix flake-parts module.
          Project homepage: "https://github.com/mightyiam/files".
        '';
      };

      advisoryDb = lib.mkOption {
        type = lib.types.package;

        description = ''
          The resolved input (non-Flake) of the RustSec's AdvisoryDB repository.
          Project homepage: "https://github.com/rustsec/advisory-db"
        '';
      };
    };
  });
}
