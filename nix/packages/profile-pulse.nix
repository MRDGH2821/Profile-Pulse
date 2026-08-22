{
  flake,
  pkgs,
  ...
}: let
  crane = flake.lib.craneFor pkgs;
  cargoArtifacts = crane.buildDepsOnly crane.appPackageArgs;
in
  crane.craneLib.buildPackage (
    crane.appPackageArgs
    // {
      inherit cargoArtifacts;
    }
  )
