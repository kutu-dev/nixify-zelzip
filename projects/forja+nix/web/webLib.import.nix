# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{
  config,
  pkgs,
  lib,
}: rec {
  rootPath = config.forja.rootPath;
  src = rootPath;

  commonNativeBuildInputs = with pkgs; [
    pnpm.configHook
    nodejs
  ];

  pnpmWorkspaces = [
    "@zelzip/workspace_root"
    "@zelzip/icebrk_web"
    "@zelzip/docs"
  ];

  pnpmDepsSrc = lib.fileset.toSource {
    root = rootPath;

    fileset = lib.fileset.unions [
      (rootPath + "/pnpm-lock.yaml")
      (rootPath + "/pnpm-workspace.yaml")
      (lib.fileset.fileFilter ({name, ...}: name == "package.json") rootPath)
    ];
  };

  pnpmDeps = pkgs.pnpm.fetchDeps {
    pname = "workspaceDeps";
    hash = "sha256-otgcFb2jouDS7gpgFGW5kcWNPv91Fnw+TeZ1DgAgpEs=";

    src = pnpmDepsSrc;
    inherit pnpmWorkspaces;
  };

  makePnpmPackage = packageName: includeProjects: buildPhase:
    pkgs.stdenv.mkDerivation (finalAttrs: let
      mainProject = builtins.elemAt includeProjects 0;

      packageManifest = lib.importJSON (rootPath + "/projects/${mainProject}/package.json");
    in {
      pname = packageManifest.name;
      version = packageManifest.version;

      nativeBuildInputs = commonNativeBuildInputs;

      buildPhase = ''
        runHook preBuild

        ${buildPhase}

        runHook postBuild
      '';

      inherit src;
      inherit pnpmDeps;
      inherit pnpmWorkspaces;
    });

  makePnpmAstroPackage = packageName: includeProjects:
    makePnpmPackage packageName includeProjects ''
      pnpm --filter="${packageName}" run forja:build --outDir "$out"
    '';

  makePnpmVitepressPackage = packageName: includeProjects:
    makePnpmPackage packageName includeProjects ''
      pnpm --filter="${packageName}" run forja:build --outDir "$out"
    '';
}
