let 
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
    rust-analyzer
  ];

  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

}