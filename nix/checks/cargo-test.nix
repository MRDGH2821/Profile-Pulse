{
  flake,
  pkgs,
  ...
}: let
  crane = flake.lib.craneFor pkgs;
  cargoArtifacts = crane.buildDepsOnly crane.workspaceCheckArgs;
in
  crane.craneLib.cargoTest (
    crane.workspaceCheckArgs
    // {
      inherit cargoArtifacts;
    }
  )
