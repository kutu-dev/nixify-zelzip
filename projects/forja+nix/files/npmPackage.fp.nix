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

          manifest = lib.attrsets.recursiveUpdate (builtins.fromJson path) {
            dependencies = {
              "@zelzip/icebrk" = self'.packages.icebrkWasmNpm;
            };
          };
        in {
          inherit path_;
          drv = pkgs.writeText "README.md" (builtins.toJSON manifest);
        }
      );
  in {
    files = {
      files = npmPackages;
    };
  };
}
