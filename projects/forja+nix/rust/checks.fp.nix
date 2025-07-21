# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{flake-parts-lib, ...}: {
  options.perSystem = flake-parts-lib.mkPerSystemOption ({
    config,
    lib,
    pkgs,
    self',
    ...
  }: let
    crane = import ./crane.import.nix {
      inherit config;
      inherit pkgs;
      inherit self';
      inherit lib;
    };
  in {
    checks = {
      # Ensure that the document compiles
      inherit (self'.packages) rustDocumentation;

      rustClippy = crane.libs.native.cargoClippy (
        crane.makeWorkspaceArgs "cargoClippy"
        {
          cargoArtifacts = crane.cargoArtifacts.native;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        }
      );

      rustFormat = crane.libs.native.cargoFmt (crane.makeWorkspaceArgs "rustFormat" {});

      taploFmt = crane.libs.native.taploFmt (crane.makeWorkspaceArgs "taploFmt" {
        src = pkgs.lib.sources.sourceFilesBySuffices crane.workspacePath [".toml"];
      });

      cargoAudit = crane.libs.native.cargoAudit (crane.makeWorkspaceArgs "cargoAudit" {
        advisory-db = config.forja.inputs.advisoryDb;
      });

      cargoDeny = crane.libs.native.cargoDeny (crane.makeWorkspaceArgs "cargoDeny" {});

      cargoNextest = crane.libs.native.cargoNextest (
        crane.makeWorkspaceArgs "cargoNextest"
        {
          cargoArtifacts = crane.cargoArtifacts.native;

          partitions = 1;
          partitionType = "count";
          cargoNextestPartitionsExtraArgs = "--no-tests=pass";
        }
      );

      cargoHakari = crane.libs.native.mkCargoDerivation (crane.makeWorkspaceArgs "cargoHakari" {
        cargoArtifacts = null;
        doInstallCargoArtifacts = false;

        buildPhaseCargoCommand = ''
                cargo hakari generate --diff  # workspace-hack Cargo.toml is up-to-date
          cargo hakari manage-deps --dry-run  # all workspace crates depend on workspace-hack
                cargo hakari verify
        '';

        nativeBuildInputs = [
          pkgs.cargo-hakari
        ];
      });
    };
  });
}
