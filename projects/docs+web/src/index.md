# ZEL.ZIP Official Documentation

[ZEL.ZIP](https://zel.zip) is a group of developers that build the set of utilitary software and libraries for Nintendo systems known as the "ZEL.ZIP tools". With this documentation is also available the [ZEL.ZIP Wiki](./wiki/wiki), storing informating about obscure and tangential systems made by Nintendo.

Currently there are the following tools available:

## Libraries

- [NiiEBLA](./niiebla): Rust parsing library for various Nintendo file formats. With accurate support for multiple niche entries and extensions (TikV1, Wii Savegame data, etc).
- [ReNUS](./renus): Rust web API wrapper that makes easy to make requests and download content from the [Nintendo Update Server (NUS)](https://wiibrew.org/wiki/NUS).
- IceBrk: Library that implements the different algorithms used on the Nintendo consoles to generate the parental control master key. Available both on [Rust](https://docs.rs/zelzip_icebrk) and on [JavaScript and TypeScript](https://wasm.icebrk.docs.zel.zip).

## Applications

- [ViiENTO](https://zel.zip/viiento): CLI wrapper around the [NiiEBLA](./niiebla) and [ReNUS](./renus) libraries. Capable of:
  - Packing, unpacking, editing and fakesigning of files.
  - Download content from the [NUS](https://wiibrew.org/wiki/NUS) with ease.
  - Generate parental control master keys.
- [IceBrk Web](https://icebrk.zel.zip): Website that allows to generate a master key to unlock the parental control of a console.
- [SetoDB](https://setodb.zel.zip): Database with technical information about hundreds of Nintendo titles (both games and internal software).
