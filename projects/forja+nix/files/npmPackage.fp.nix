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
      |> lib.fileset.fileFilter ({name, ...}: name == "package.template.json5")
      |> lib.fileset.toList
      |> map (
        path: let
          relativeParentPath = lib.path.removePrefix rootPath (dirOf path);
          path_ = relativeParentPath + "/package.json5";

          manifest = lib.attrsets.recursiveUpdate (builtins.fromJSON (builtins.readFile path)) {
            dependencies = {
              "@zelzip/icebrk" = self'.packages.icebrkWasmNpm;
            };
          };
        in {
          inherit path_;
          drv = pkgs.writeText "package.json5" (builtins.toJSON manifest);
        }
      );
  in {
    files = {
      files = npmPackages;
    };
  };
}
