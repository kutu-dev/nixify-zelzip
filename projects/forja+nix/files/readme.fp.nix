# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{...}: {
  perSystem = {
    pkgs,
    config,
    lib,
    ...
  }: let
    rootPath = config.forja.rootPath;

    makeReadmeHeader = title: ''
      <!--
        DO NOT EDIT!
        THIS IS A MACHINE GENERATED FILE

        Seeded with the data stored at `README.md.template.nix`,
        to regenerate the file run `forja fix` or `nix run .#generateFiles`.
      -->

      # ${title}
      [Usage guide](https://docs.zelzip.dev/niiebla/niiebla.html) | [Reference](https://docs.rs/zelzip_niiebla) | [ZELZIP website](https://zelzip.dev)

    '';

    readmeFooter = ''

      ## Credits
      Every person that has contributed to ZELZIP is credited on our [credits page](https://zelzip.dev/credits).

      ## Copyright
      All files store at this repository are under the [Mozilla Public License Version 2.0](https://www.mozilla.org/en-US/MPL/2.0/) otherwise noted.

      ## Legal notice
      This project is a fan-made homebrew creation developed independently and is not affiliated with, endorsed by, or associated with Nintendo Co., Ltd or any of its subsidiaries, affiliates, or partners. All trademarks and copyrights referenced are the property of their respective owners.
    '';

    readmeFiles =
      config.forja.rootPath
      |> lib.fileset.fileFilter ({name, ...}: name == "README.md.template.nix")
      |> lib.fileset.toList
      |> map (
        path: let
          relativeParentPath = lib.path.removePrefix rootPath (dirOf path);
          path_ = relativeParentPath + "/README.md";

          templateData = import path {};
          readmeHeader = makeReadmeHeader templateData.title;
          text = readmeHeader + templateData.body + readmeFooter;
        in {
          inherit path_;
          drv = pkgs.writeText "README.md" text;
        }
      );
  in {
    files = {
      files = readmeFiles;
    };
  };
}
