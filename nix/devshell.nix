{
  inputs,
  pkgs,
  ...
}: let
  pre-commit-check = import ./checks/pre-commit-check.nix {inherit inputs pkgs;};
in
  pkgs.mkShell {
    inherit (pre-commit-check) shellHook;
    packages = with pkgs; [
      # keep-sorted start
      bun
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
      # keep-sorted end
    ];
  }
