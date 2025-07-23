# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{...}: {
  title = "ZELZIP Icebrk Library";

  body =
    # markdown
    ''
      Implementation of the different algorithms used on Nintendo consoles to generate the parental control master key.

      ## Getting Started

      This library is available on:
      - Rust, check the [crate documentation](https://docs.rs/zelzip_icebrk).
      - JavaScript or TypeScript via WASM, check the [typed NPM library documentation](https://wasm.icebrk.docs.zelzip.dev).

      ## Limitations
      - No support for the Nintendo Switch v4 algorithm as it requires a Device ID value only obtainable using homebrew tools, [these same tools also allows for disabling any sort of parental control](https://gbatemp.net/threads/reset-parental-control-nx-an-easy-to-reset-the-pin-for-controls.556891/) making the support of this version redundant.
    '';
}
