# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{...}: {
  perSystem = {pkgs, ...}: {
    packages.appleSdk = pkgs.stdenv.mkDerivation {
      name = "apple-sdk_15.2";
      src = pkgs.fetchzip {
        url = "https://github.com/joseluisq/macosx-sdks/releases/download/15.2/MacOSX15.2.sdk.tar.xz";
        sha256 = "sha256:0fgj0pvjclq2pfsq3f3wjj39906xyj6bsgx1da933wyc918p4zi3";
      };

      phases = ["installPhase"];

      installPhase = ''
        mkdir -p "$out"

        cp -r "$src"/* "$out"
      '';
    };
  };
}
