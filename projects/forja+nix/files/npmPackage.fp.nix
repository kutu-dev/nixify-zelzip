# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{...}: {
  perSystem = {
    pkgs,
    self',
    config,
    lib,
    ...
  }: let
    rootPath = config.forja.rootPath;

    npmPackages =
      config.forja.rootPath
      |> lib.fileset.fileFilter ({name, ...}: name == "package.template.json")
      |> lib.fileset.toList
      |> map (
        path: let
          relativeParentPath = lib.path.removePrefix rootPath (dirOf path);
          path_ = relativeParentPath + "/package.json";

          # TODO(FIX): Integrate JSON5 on Nix
          # - JSON5 parser for Nix
          # - Use JSON5 in templates and final NPM manifests
          # - Add warning message back

          /*
          warningMessage = ''
            // DO NOT EDIT!
            // THIS IS A MACHINE GENERATED FILE
            //
            // Seeded with the data stored at `package.template.json5`
            // to regenerate the file run `forja fix` or `forja gen`.

          '';
          */

          manifest =
            path
            |> builtins.readFile
            |> builtins.fromJSON
            |> (oldManifest:
              lib.attrsets.recursiveUpdate oldManifest {
                dependencies = {
                  "@zelzip/icebrk" = self'.packages.icebrkWasmNpm;
                };
              })
            |> builtins.toJSON
            |> (manifest: manifest);
        in {
          inherit path_;
          drv = pkgs.writeText "package.json" manifest;
        }
      );
  in {
    files = {
      files = npmPackages;
    };
  };
}
