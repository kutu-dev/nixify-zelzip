# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{...}: {
  title = "ZELZIP NiiEBLA Library";

  links = {
    "Usage guide" = "https://docs.zelzip.dev/niiebla/niiebla.html";
    "Reference" = "https://docs.rs/zelzip_niiebla";
  };

  body =
    # markdown
    ''
      A parsing library for various Nintendo file formats. With accurate support for multiple niche entries and extensions (TikV1, Wii Savegame data, etc).

      Supports:

      - [`WAD`](https://wiibrew.org/wiki/WAD_files)/`TAD` files manipulation (with content adding, editing and removing), both installable (`Is`/`ib`) and backup (`Bk`) kinds.
      - Encryption/Decryption of content data for Nintendo Wii and Nintendo DSi titles.
      - [Ticket](https://wiibrew.org/wiki/Ticket) (pre Nintendo Switch) `TIK` files.
      - [Title metadata](https://wiibrew.org/wiki/Title_metadata) (pre Nintendo Switch) `TMD` files.
      - [Nintendo certificate chain](https://wiibrew.org/wiki/Certificate_chain) format.
      - [U8 archive](https://wiibrew.org/wiki/U8_archive) files.
      - [Trucha bug based fakesigning for the Nintendo Wii](https://wiibrew.org/wiki/Signing_bug).
      - [Nintendo Wii's savegame format](https://wiibrew.org/wiki/Savegame_Files).

      ## Limitations

      Be aware of the following limitations of the library:
      Soft limitations (will not be implemented unless a lot of interest is arised and documentation is improved):

      - Content viewing and editing only available on Wii and DSi titles (TMD group hashes are not properly updated).
      - CRL data on WAD files is not preserved.
      - Arbitrary content types is not supported (understading and documenting the meaning of its bitflags would be required).
      - Modifying contents on titles with TMD version 1 will not edit its content entry groups hashes.
      - "Section total size" and "size of each region" are not checked for correctness when parsing.

      Hard limitations (cannot or are to complex to be fixed):

      - 1:1 byte match on the V1 section of a `PreSwitchTicket` cannot be ensured.
    '';
}
