{inputs, ...}: {
  craneFor = pkgs: import ./crane.nix {inherit inputs pkgs;};
}
