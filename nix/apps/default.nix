{
  inputs,
  perSystem,
  ...
}:
inputs.blueprint.lib.mkApp {
  drv = perSystem.self.profile-pulse;
}
