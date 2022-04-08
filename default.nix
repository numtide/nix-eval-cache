with import <nixpkgs> {};
mkShell {
  nativeBuildInputs = [
    bashInteractive
    rustc
    cargo
    cargo-watch
    rust-analyzer
  ];
}
