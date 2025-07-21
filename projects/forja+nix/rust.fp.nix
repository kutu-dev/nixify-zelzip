{flake-parts-lib, ...}: {
  imports = [
    ./rust/toolchain.fp.nix
    ./rust/packages.fp.nix
    ./rust/crane.fp.nix
  ];
}
