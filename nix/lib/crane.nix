{
  inputs,
  pkgs,
}: let
  craneLib = inputs.crane.mkLib pkgs;
  inherit (pkgs) lib;

  pname = "profile-pulse";
  version = (fromTOML (builtins.readFile ../../crates/app/Cargo.toml)).package.version;

  sourceFilter = path: type: (craneLib.filterCargoSources path type) || lib.hasSuffix ".css" path;

  src = lib.cleanSourceWith {
    filter = sourceFilter;
    src = craneLib.path ../..;
  };

  desktopNativeBuildInputs = with pkgs; [
    pkg-config
    rustPlatform.bindgenHook
  ];

  desktopBuildInputs = with pkgs; [
    bzip2
    gtk3
    librsvg
    openssl
    webkitgtk_4_1
    xdotool
    xz
  ];

  baseArgs = {
    inherit pname src version;
    strictDeps = true;
  };

  appPackageArgs =
    baseArgs
    // {
      buildInputs = desktopBuildInputs;
      cargoExtraArgs = "--package profile-pulse-app";
      meta.mainProgram = "profile-pulse";
      nativeBuildInputs = desktopNativeBuildInputs;
    };

  workspaceCheckArgs =
    baseArgs
    // {
      cargoExtraArgs = "--workspace --exclude profile-pulse-legacy";
    };

  buildDepsOnly = args: craneLib.buildDepsOnly args;
in {
  inherit craneLib buildDepsOnly;
  inherit appPackageArgs workspaceCheckArgs;
}
