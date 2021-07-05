{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, utils, ... }@inputs:
    utils.lib.eachDefaultSystem (system:
      let

        pname = "boids";

        overlays = [ inputs.rust-overlay.overlay ];
        nixpkgs = import inputs.nixpkgs { inherit system overlays; };

        # Get the latest rust nightly
        rust = nixpkgs.rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.default.override { extensions = [ "rust-src" ]; });

        build-script = nixpkgs.writeScriptBin "build" ''
          #!${nixpkgs.stdenv.shell}
          ${nixpkgs.wasm-pack}/bin/wasm-pack build --target web \
                                                   --out-name wasm \
                                                   --out-dir ./docs/wasm/
        '';

        run-script = nixpkgs.writeScriptBin "run" ''
          #!${nixpkgs.stdenv.shell}
          ${build-script}/bin/build && miniserve ./docs
        '';

      in rec {
        # `nix develop`
        devShell = nixpkgs.mkShell {
          nativeBuildInputs = [
            build-script
            nixpkgs.miniserve
            nixpkgs.rustup
            nixpkgs.wasm-pack
            run-script
            rust
          ];
          RUST_SRC_PATH = "${rust}";
        };
      });
}
