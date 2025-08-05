<!--
  DO NOT EDIT!
  THIS IS A MACHINE GENERATED FILE

  Seeded with the data stored at `README.md.template.nix`,
  to regenerate the file run `forja fix` or `forja gen`.
-->

# ZELZIP NiiEBLA Library
[Reference](https://docs.rs/zelzip_niiebla) | [Usage guide](https://docs.zelzip.dev/niiebla/niiebla.html) | [ZELZIP website](https://zelzip.dev) | [Source code](https://github.com/ZELZIP/ZELZIP)

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

## Credits
Every person that has contributed to ZELZIP is credited on our [credits page](https://zelzip.dev/credits).

## Copyright
All files store at this repository are under the [Mozilla Public License Version 2.0](https://www.mozilla.org/en-US/MPL/2.0/) otherwise noted.

## Legal notice
This project is a fan-made homebrew creation developed independently and is not affiliated with, endorsed by, or associated with Nintendo Co., Ltd or any of its subsidiaries, affiliates, or partners. All trademarks and copyrights referenced are the property of their respective owners.
