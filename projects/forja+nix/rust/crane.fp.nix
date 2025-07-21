{flake-parts-lib, ...}: {
  options.perSystem = flake-parts-lib.mkPerSystemOption ({
    config,
    lib,
    pkgs,
    self',
    pkgsLinuxAarch64,
    ...
  }: let
    mkLib = config.forja.inputs.crane.mkLib;

    makeCraneLib = pkgs: overrides: (mkLib pkgs).overrideToolchain (pkgs: pkgs.rust-bin.stable.latest.default.override overrides);
  in {
    options.forja.rust.crane.libs = lib.mkOption {
      type = lib.types.attrsOf lib.types.anything;

      description = ''
        TODO
      '';
    };

    config.forja.rust.crane.libs = {
      native = makeCraneLib pkgs {};

      linux.aarch64 = makeCraneLib pkgs.forja.cross.linux.aarch64 {};
      linux.x86_64 = makeCraneLib pkgs.forja.cross.linux.x86_64 {};

      # NOTE: Crosscompilation to macOS on nixpkgs is only available as `darwin-x86_64` <-> `darwin-aarch64`,
      # building with the native toolchain and manually setup the Apple SDK is required.
      macos.x86_64 = makeCraneLib pkgs {
        targets = ["x86_64-apple-darwin"];
      };

      macos.aarch64 = makeCraneLib pkgs {
        targets = ["aarch64-apple-darwin"];
      };

      windows.x86_64 = makeCraneLib pkgs.forja.cross.windows.x86_64 {
        targets = ["x86_64-pc-windows-gnu"];
      };

      wasm32 = makeCraneLib pkgs {
        targets = ["wasm32-unknown-unknown"];
      };
    };
  });
}
