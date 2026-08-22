{
  flake,
  pkgs,
  ...
}: let
  crane = flake.lib.craneFor pkgs;
  cargoArtifacts = crane.buildDepsOnly crane.workspaceCheckArgs;
in
  crane.craneLib.cargoClippy (
    crane.workspaceCheckArgs
    // {
      inherit cargoArtifacts;
      cargoClippyExtraArgs = "--all-targets -- -D warnings";
    }
  )
