# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{inputs, ...}: {
  perSystem = {
    pkgs,
    config,
    lib,
    ...
  }: let
    warningMessage = ''
      # DO NOT EDIT!
      # THIS IS A MACHINE GENERATED FILE
      #
      # Seeded with the data stored at `//projects/forja+nix/files/ignores/`,
      # to regenerate the file run `forja fix` or `forja gen`.

    '';

    ignoreText =
      ./ignores
      |> lib.filesystem.listFilesRecursive
      |> map builtins.readFile
      |> (textBlobs: [warningMessage] ++ textBlobs)
      |> lib.lists.foldl' (a: b: a + b) "";
  in {
    files = {
      gitToplevel = config.forja.rootPath;
      files = [
        {
          path_ = ".gitignore";
          drv = pkgs.writeText ".gitignore" ignoreText;
        }
      ];
    };
  };
}
