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
  ];

  pnpmDepsSrc = lib.fileset.toSource {
    root = rootPath;

    fileset = lib.fileset.unions [
      (rootPath + "/pnpm-lock.yaml")
      (rootPath + "/pnpm-workspace.yaml")
      (lib.fileset.fileFilter ({name, ...}: name == "package.json5") rootPath)
    ];
  };

  pnpmDeps = pkgs.pnpm.fetchDeps {
    pname = "workspaceDeps";
    hash = "sha256-ljiUG0wziWRdFAt9fAcVxhMjcg4RF6MTVFfpRyiJpN8=";

    src = pnpmDepsSrc;
    inherit pnpmWorkspaces;
  };

  makePnpmPackage = packageName: includeProjects: buildPhase:
    pkgs.stdenv.mkDerivation (finalAttrs: let
      mainProject = builtins.elemAt includeProjects 0;

      packageManifest = lib.importJSON (rootPath + "/projects/${mainProject}/package.json5");
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
}
