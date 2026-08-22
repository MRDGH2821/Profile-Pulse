{
  inputs,
  pkgs,
  ...
}: let
  pre-commit-check = import ./checks/pre-commit-check.nix {inherit inputs pkgs;};
  wasmRuntimeLibs = pkgs.lib.makeLibraryPath [
    pkgs.xz
    pkgs.bzip2
  ];
in
  pkgs.mkShell {
    packages = with pkgs; [
      # keep-sorted start
      bun
      bzip2
      cocogitto
      copier
      git
      git-credential-oauth
      glab
      gtk3
      lazygit
      librsvg
      nil
      nixd
      openssl
      pkg-config
      repgrep
      ripgrep
      uv
      webkitgtk_4_1
      xdotool
      xz
      # keep-sorted end
    ];
    shellHook =
      pre-commit-check.shellHook
      + ''
        export LD_LIBRARY_PATH="${wasmRuntimeLibs}:''${LD_LIBRARY_PATH:-}"
      '';
  }
