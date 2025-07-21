_: {
  perSystem = {
    config,
    system,
    pkgs,
    ...
  }: let
    nixpkgs = config.forja.inputs.nixpkgs;
    localSystem = system;

    overlays = [
      (import config.forja.inputs.rustOverlay)
    ];

    makeCrossPkgs = crossSystem:
      import nixpkgs {
        inherit localSystem;
        inherit overlays;
        inherit crossSystem;
      };
  in {
    _module.args.pkgs = import nixpkgs {
      inherit system;

      overlays =
        overlays
        ++ [
          (final: prev: {
            forja.cross.linux.x86_64 = makeCrossPkgs "x86_64-unknown-linux-gnu";
            forja.cross.linux.aarch64 = makeCrossPkgs "aarch64-unknown-linux-gnu";

            forja.cross.windows.x86_64 = makeCrossPkgs {
              config = "x86_64-w64-mingw32";
              libc = "msvcrt";
            };
          })
        ];
    };
  };
}
