# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{...}: {
  perSystem = {
    pkgs,
    self',
    ...
  }: {
    devShells.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        self'.packages.rustToolchain
        self'.packages.forjaCli

        alejandra
        taplo
        cargo-deny
        cargo-hakari
        wasm-pack
        wasm-bindgen-cli
        addlicense
        glow
        nix-output-monitor
        nixd
        vscodium
        tokei
      ];
    };
  };
}
