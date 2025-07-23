# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{flake-parts-lib, ...}: {
  perSystem = {
    pkgs,
    config,
    lib,
    ...
  }: let
    rootPath = config.forja.rootPath;
    src = rootPath;

    commonNativeBuildInputs = with pkgs; [
      pnpm.configHook
      nodejs
    ];

    pnpmWorkspaces = [
      "@zelzip/workspace-root"
      "@zelzip/icebrk-web"
    ];

    pnpmDeps = pkgs.pnpm.fetchDeps {
      pname = "workspaceDeps";
      hash = "sha256-5a6naLHN7BBifDPWmued2++Hr0342Vb82kQh0fqLXiw=";

      inherit src;
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
  in {
    checks.checkEslint = pkgs.stdenv.mkDerivation (finalAttrs: {
      pname = "checkEslint";
      version = "1.0.0";

      inherit src;
      inherit pnpmDeps;
      inherit pnpmWorkspaces;

      nativeBuildInputs = commonNativeBuildInputs;

      buildPhase = ''
        runHook preBuild

        mkdir -p "$out"
        pnpm eslint

        runHook postBuild
      '';
    });

    packages.icebrkWeb = makePnpmAstroPackage "@zelzip/icebrk-web" ["icebrk_web+web"];
  };
}
