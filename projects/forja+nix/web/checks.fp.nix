{...}: {
  perSystem = {
    config,
    pkgs,
    lib,
    ...
  }: let
    webLib = import ./webLib.import.nix {
      inherit config pkgs lib;
    };
  in {
    checks.checkEslint = pkgs.stdenv.mkDerivation (finalAttrs: {
      pname = "checkEslint";
      version = "1.0.0";

      src = webLib.src;
      pnpmDeps = webLib.pnpmDeps;
      pnpmWorkspaces = webLib.pnpmWorkspaces;

      nativeBuildInputs = webLib.commonNativeBuildInputs;

      buildPhase = ''
        runHook preBuild

        mkdir -p "$out"
        pnpm eslint

        runHook postBuild
      '';
    });
  };
}
