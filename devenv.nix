{
  pkgs,
  lib,
  config,
  ...
}:
{
  # https://devenv.sh/languages/
  languages.javascript = {
    enable = true;
    pnpm.enable = true;
  };

  languages.rust = {
    enable = true;
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  # https://devenv.sh/packages/
  packages = [
    pkgs.cargo-tauri
  ] ++ lib.optionals pkgs.stdenv.isLinux [
    # GTK/WebKit dependencies only needed on Linux
    # macOS uses native WebKit framework
    pkgs.glib
    pkgs.atkmm
    pkgs.pango
    pkgs.gdk-pixbuf
    pkgs.gtk3
    pkgs.webkitgtk_4_1
  ];

  # https://devenv.sh/basics/
  enterShell = ''
    pnpm install
  '';

  # See full reference at https://devenv.sh/reference/options/
}
