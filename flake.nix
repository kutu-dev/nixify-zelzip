# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{
  nixConfig = {
    abort-on-warn = true;
    extra-experimental-features = ["pipe-operators"];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    rustOverlay.url = "github:oxalica/rust-overlay";
    rustOverlay.inputs.nixpkgs.follows = "nixpkgs";

    crane.url = "github:ipetkov/crane";

    advisoryDb.url = "github:rustsec/advisory-db";
    advisoryDb.flake = false;
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} ({flake-parts-lib, ...}: {
      imports = [./projects/forja+nix/default.fp.nix];

      perSystem = {
        pkgs,
        inputs',
        self',
        ...
      }: {
        forja.rootPath = ./.;

        forja.inputs = with inputs; {
          inherit nixpkgs;
          inherit rustOverlay;
          inherit crane;
          inherit advisoryDb;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            pkgs.alejandra
            pkgs.taplo
            pkgs.cargo-deny
            pkgs.cargo-hakari
            pkgs.llvmPackages.bintools-unwrapped
            pkgs.patchelf
            pkgs.wasm-pack
            pkgs.wasm-bindgen-cli
          ];
        };
      };
    });
}
