# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{
  config,
  pkgs,
  self',
  lib,
}: rec {
  # Reexport to make code cleaner
  libs = config.forja.rust.crane.libs;

  workspacePath = config.forja.rootPath;

  commonArgs = {
    src = libs.native.cleanCargoSource workspacePath;
    strictDeps = true;
  };

  # Args for any command that operates on the Cargo workspace level and
  # not individual packages
  makeWorkspaceArgs = pname: extra: (commonArgs
    // {
      inherit pname;

      # Dummy value to silent warning
      version = "1.0.0";
    }
    // extra);

  # NOTE: Crosscompilation to macOS on nixpkgs is only available as `darwin-x86_64` <-> `darwin-aarch64`,
  # so let's do it manually with the LLVM with the native host
  macosCommonExtraArgs = {
    SDKROOT = "${self'.packages.appleSdk}";

    #
    RUSTFLAGS = "-C link-args=-L${pkgs.libiconv}/lib";

    nativeBuildInputs = with pkgs; [
      # `rust-objcopy` is only installed on the Rust toolchain when the host is macOS,
      # it's just a wrapper around `llvm-objcopy`
      (writeShellScriptBin "rust-objcopy" ''
        ${llvmPackages.bintools-unwrapped}/bin/llvm-objcopy "$@"
      '')
    ];
  };

  macosX86_64ExtraArgs =
    {
      CARGO_BUILD_TARGET = "x86_64-apple-darwin";
      CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER = "${pkgs.lld}/bin/lld";
    }
    // macosCommonExtraArgs;

  macosAarch64ExtraArgs =
    {
      CARGO_BUILD_TARGET = "aarch64-apple-darwin";
      CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER = "${pkgs.lld}/bin/lld";
    }
    // macosCommonExtraArgs;

  makeCargoArtifacts = craneLib: extraArgs: craneLib.buildDepsOnly (makeWorkspaceArgs "cargoArtifacts" extraArgs);

  cargoArtifacts = {
    native = makeCargoArtifacts libs.native {};

    linux.x86_64 = makeCargoArtifacts libs.linux.x86_64 {};
    linux.aarch64 = makeCargoArtifacts libs.linux.aarch64 {};

    macos.x86_64 = makeCargoArtifacts libs.macos.x86_64 macosX86_64ExtraArgs;
    macos.aarch64 = makeCargoArtifacts libs.macos.aarch64 macosAarch64ExtraArgs;

    windows.x86_64 = makeCargoArtifacts libs.windows.x86_64 {};

    wasm32 = makeCargoArtifacts libs.wasm32 {
      CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
    };
  };

  # Fileset with only the required files for Rust compilation,
  # where `crateName` is the name of the directory inside `//projects` where the package is stored
  makeFileSetForCrate = cratesNames:
    lib.fileset.toSource {
      root = workspacePath;
      fileset = lib.fileset.unions ([
          (workspacePath + "/Cargo.toml")
          (workspacePath + "/Cargo.lock")
        ]
        ++ (map (crateName: workspacePath + "/projects/${crateName}") cratesNames));
    };

  # `rustPackageName` is the name of the Rust package as defined in the `Cargo.toml` file,
  # `crateName` is the name of the directory inside `//projects` where the package is stored
  makeCrateArgs = cargoPackageName: includeProjects:
    commonArgs
    // {
      inherit (libs.native.crateNameFromCargoToml {src = workspacePath + "/projects/${builtins.elemAt includeProjects 0}";}) pname version;

      cargoExtraArgs = "-p ${cargoPackageName}";
      src = makeFileSetForCrate includeProjects;

      # Disable checks and tests by default as they are merged in a different derivation.
      doCheck = false;
    };

  makeCratePackages = {
    nixPackageName,
    makeNixPackage ? craneLib: crateArgs: {}: craneLib.buildPackage crateArgs,
    cargoPackageName,
    includeProjects,
    hasBin,
    hasLib,
  }: let
    crateArgs = makeCrateArgs cargoPackageName includeProjects;

    modifySuffixOfDerivationBinaries = newSuffix: oldSuffix: drv:
      pkgs.runCommand "modifySuffixOfDerivationBinaries" {} ''
        mkdir -p "$out/bin"

        for file in ${drv}/bin/*${oldSuffix}; do
               old_filename="$(basename "$file")"
               new_filename="''${old_filename%%${oldSuffix}}${newSuffix}"

               cp "$file" "$out/bin/$new_filename"
        done
      '';

    unixifyLinuxDerivationBinaries = drv:
      pkgs.runCommand "unixifyDerivationBinaries" {} ''
        mkdir -p "$out/bin"

        for file in ${drv}/bin/*; do
                filename="$(basename "$file")"
                ${pkgs.patchelf}/bin/patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 --remove-rpath --output "$out/bin/$filename" "$file"
        done
      '';

    mergeMacosDerivationBinaries = drvX86_64: drvAarch64:
      pkgs.runCommand "${nixPackageName}MacosUniversal" {} ''
        mkdir -p "$out/bin/"

        x86_64_binaries_paths=( ${drvX86_64}/bin/* )
        aarch64_binaries_paths=( ${drvAarch64}/bin/* )

        len="''${#x86_64_binaries_paths[@]}"

        for (( i = 0; i < len; i++ )); do
               x86_64_binary_path="''${x86_64_binaries_paths[i]}"
               aarch64_binary_path="''${aarch64_binaries_paths[i]}"

               filename="$(basename --suffix "-macos-x86_64" "$x86_64_binary_path")"

                ${pkgs.llvmPackages.bintools-unwrapped}/bin/llvm-lipo "$x86_64_binary_path" "$aarch64_binary_path" -create -output "$out/bin/$filename-macos-universal"
        done
      '';

    addArtifactsToArgs = cargoArtifacts: crateArgs: crateArgs // {inherit cargoArtifacts;};

    native =
      crateArgs
      |> addArtifactsToArgs cargoArtifacts.native
      |> makeNixPackage libs.native
      |> (package: pkgs.callPackage package {});

    linuxX86_64 =
      crateArgs
      |> addArtifactsToArgs cargoArtifacts.linux.x86_64
      |> makeNixPackage libs.linux.x86_64
      |> (package: pkgs.forja.cross.linux.x86_64.callPackage package {})
      |> modifySuffixOfDerivationBinaries "-linux-x86_64" ""
      |> unixifyLinuxDerivationBinaries;

    linuxAarch64 =
      crateArgs
      |> addArtifactsToArgs cargoArtifacts.linux.aarch64
      |> makeNixPackage libs.linux.aarch64
      |> (package: pkgs.forja.cross.linux.aarch64.callPackage package {})
      |> modifySuffixOfDerivationBinaries "-linux-aarch64" ""
      |> unixifyLinuxDerivationBinaries;

    macosX86_64 =
      crateArgs
      // macosX86_64ExtraArgs
      |> addArtifactsToArgs cargoArtifacts.macos.x86_64
      |> makeNixPackage libs.macos.x86_64
      |> (package: pkgs.callPackage package {})
      |> modifySuffixOfDerivationBinaries "-macos-x86_64" "";

    macosAarch64 =
      crateArgs
      // macosAarch64ExtraArgs
      |> addArtifactsToArgs cargoArtifacts.macos.aarch64
      |> makeNixPackage libs.macos.aarch64
      |> (package: pkgs.callPackage package {})
      |> modifySuffixOfDerivationBinaries "-macos-aarch64" "";

    macosUniversal = mergeMacosDerivationBinaries macosX86_64 macosAarch64;

    windowsX86_64 =
      crateArgs
      |> addArtifactsToArgs cargoArtifacts.windows.x86_64
      |> makeNixPackage libs.windows.x86_64
      |> (package: pkgs.forja.cross.windows.x86_64.callPackage package {})
      |> modifySuffixOfDerivationBinaries "-windows-x86_64.exe" ".exe";

    wasmNpm =
      crateArgs
      // {
        cargoExtraArgs = "";

        # Avoid build failure due to not put build logs to `cargoBuildLog`,
        # not relevant in this case as there is no install phase
        doNotPostBuildInstallCargoBinaries = true;
        installPhaseCommand = '''';

        buildPhaseCargoCommand = ''
          mkdir -p "$out/pkg"

          # TODO(TRACK: https://github.com/ipetkov/crane/issues/362): |>
          #   wasm-pack expects a valid home directory
          HOME=$(mktemp -d fake-home-XXXXXXXXXX) ${pkgs.wasm-pack}/bin/wasm-pack build ./projects/${builtins.elemAt includeProjects 0} --out-dir $out/pkg
        '';

        nativeBuildInputs = with pkgs; [
          wasm-bindgen-cli
        ];
      }
      # TODO(TRACK: https://github.com/ipetkov/crane/issues/873): |>
      #   There is no mechanism on Crane to filter incompatible dependencies
      #|> addArtifactsToArgs cargoArtifacts.wasm32
      |> makeNixPackage libs.wasm32
      |> (package: pkgs.callPackage package {});

    binCrossCompiledPackages =
      if ! pkgs.stdenv.isDarwin
      then {
        "${nixPackageName}LinuxX86_64" = linuxX86_64;
        "${nixPackageName}LinuxAarch64" = linuxAarch64;

        "${nixPackageName}MacosX86_64" = macosX86_64;
        "${nixPackageName}MacosAarch64" = macosAarch64;
        "${nixPackageName}MacosUniversal" = macosUniversal;

        "${nixPackageName}WindowsX86_64" = windowsX86_64;

        "${nixPackageName}All" = pkgs.symlinkJoin {
          name = "${nixPackageName}All";
          paths = [linuxX86_64 linuxAarch64 macosX86_64 macosAarch64 macosUniversal windowsX86_64];
        };
      }
      else {};

    binPackages =
      if hasBin
      then {
        "${nixPackageName}" = native;
      }
      else {};

    libPackages =
      if hasLib
      then {
        "${nixPackageName}WasmNpm" = wasmNpm;
      }
      else {};
  in
    binPackages // libPackages;
}
