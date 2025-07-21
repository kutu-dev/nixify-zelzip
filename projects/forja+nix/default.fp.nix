{
  pkgs,
  flake-parts-lib,
  ...
}: {
  imports = [
    ./pkgs.fp.nix
    ./systems.fp.nix
    ./inputs.fp.nix
    ./rust.fp.nix
    ./apple.fp.nix
  ];

  options.perSystem = flake-parts-lib.mkPerSystemOption ({
    config,
    lib,
    pkgs,
    ...
  }: {
    options.forja.rootPath = lib.mkOption {
      type = lib.types.path;

      description = ''
        TODO
      '';
    };
  });
}
