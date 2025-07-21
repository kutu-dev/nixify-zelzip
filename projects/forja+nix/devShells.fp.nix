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
