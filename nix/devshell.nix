{
  inputs,
  pkgs,
  ...
}: let
  pre-commit-check = import ./checks/pre-commit-check.nix {inherit inputs pkgs;};
  inherit (pkgs) gst_all_1;
  gstreamerPlugins = with gst_all_1; [
    gst-plugins-base
    gst-plugins-good
  ];
  desktopRuntimeLibs = pkgs.lib.makeLibraryPath (
    with pkgs; [
      # keep-sorted start
      atk
      bzip2
      cairo
      gdk-pixbuf
      glib
      gst_all_1.gst-plugins-base
      gst_all_1.gstreamer
      gtk3
      libdrm
      libepoxy
      librsvg
      libsoup_3
      libxkbcommon
      mesa
      nspr
      nss
      openssl
      pango
      wayland
      webkitgtk_4_1
      xdotool
      xz
      # keep-sorted end
    ]
  );
  gstreamerPluginPath = pkgs.lib.makeSearchPath "lib/gstreamer-1.0" gstreamerPlugins;
in
  pkgs.mkShell {
    packages = with pkgs; [
      # keep-sorted start
      bun
      bzip2
      cocogitto
      copier
      dioxus-cli
      git
      git-credential-oauth
      glab
      gst_all_1.gst-plugins-base
      gst_all_1.gst-plugins-good
      gst_all_1.gstreamer
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
      wasm-bindgen-cli_0_2_126
      webkitgtk_4_1
      xdotool
      xz
      # keep-sorted end
    ];
    shellHook =
      pre-commit-check.shellHook
      + ''
        export LD_LIBRARY_PATH="${desktopRuntimeLibs}:''${LD_LIBRARY_PATH:-}"
        export GST_PLUGIN_SYSTEM_PATH_1_0="${gstreamerPluginPath}''${GST_PLUGIN_SYSTEM_PATH_1_0:+:}$GST_PLUGIN_SYSTEM_PATH_1_0"

        # KDE/Plasma gtk-3.0/settings.ini lists host-only modules; empty env overrides it.
        export GTK_MODULES=""
      '';
  }
