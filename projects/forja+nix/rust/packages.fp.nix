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
    packages =
      {
        rustDocumentation = crane.libs.native.cargoDoc (
          crane.makeWorkspaceArgs "cargoDoc"
          {
            cargoArtifacts = crane.cargoArtifacts.native;
          }
        );
      }
      // (crane.makeCratePackages {
        nixPackageName = "forjaCli";
        cargoPackageName = "zelzip_forja_cli";
        includeProjects = ["forja_cli+rust" "util+rust" "workspace_hack+rust"];
        hasBin = true;
        hasLib = false;
      })
      // (crane.makeCratePackages {
        nixPackageName = "icebrk";
        cargoPackageName = "zelzip_icebrk";
        includeProjects = ["icebrk+rust" "workspace_hack+rust"];
        hasBin = false;
        hasLib = true;
      })
      // (crane.makeCratePackages {
        nixPackageName = "niiebla";
        cargoPackageName = "zelzip_niiebl";
        includeProjects = ["niiebla+rust" "util+rust" "workspace_hack+rust"];
        hasBin = false;
        hasLib = true;
      });

    apps.forjaCli = {
      type = "app";
      program = "${self'.packages.forjaCli}/bin/forja";
    };
  });
}
