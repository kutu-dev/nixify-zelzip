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
    packages.icebrkWeb = webLib.makePnpmAstroPackage "@zelzip/icebrk_web" ["icebrk_web+web"];
  };
}
