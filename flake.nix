# SPDX-License-Identifier: MPL-2.0
{
  description = "Julia the Viper - Harvard Architecture language making code injection grammatically impossible";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          just
        ];

        buildInputs = with pkgs; [
          openssl
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

      in
      {
        packages = {
          default = self.packages.${system}.jtv;

          jtv = pkgs.rustPlatform.buildRustPackage {
            pname = "julia-the-viper";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            inherit nativeBuildInputs buildInputs;

            meta = with pkgs.lib; {
              description = "Harvard Architecture language making code injection grammatically impossible";
              homepage = "https://github.com/hyperpolymath/julia-the-viper";
              license = with licenses; [ gpl3Plus ];
              maintainers = [ ];
            };
          };

          # WASM build
          jtv-wasm = pkgs.stdenv.mkDerivation {
            pname = "julia-the-viper-wasm";
            version = "0.1.0";

            src = ./.;

            nativeBuildInputs = with pkgs; [
              rustToolchain
              wasm-pack
              binaryen
            ];

            buildPhase = ''
              wasm-pack build --target web --out-dir $out/pkg crates/jtv-core
            '';

            installPhase = ''
              mkdir -p $out
              cp -r pkg $out/
            '';
          };
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs;

          nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
            # Development tools
            rust-analyzer
            cargo-watch
            cargo-edit
            cargo-audit

            # WASM tooling
            wasm-pack
            wasm-bindgen-cli
            binaryen

            # Documentation
            mdbook

            # Formatting & Linting
            rustfmt
            clippy

            # Testing
            cargo-nextest
          ]);

          shellHook = ''
            echo "🐍 Julia the Viper Development Environment"
            echo ""
            echo "Available commands:"
            echo "  just --list     # Show all build commands"
            echo "  just build      # Build all packages"
            echo "  just test       # Run tests"
            echo "  just build-wasm # Compile to WebAssembly"
            echo "  cargo run --bin jtv -- --help  # Run CLI"
            echo ""
            echo "Rust version: $(rustc --version)"
          '';

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        # CI/CD support
        checks = {
          build = self.packages.${system}.jtv;

          test = pkgs.runCommand "jtv-test" {
            nativeBuildInputs = nativeBuildInputs ++ buildInputs;
          } ''
            cd ${./.}
            cargo test --workspace
            touch $out
          '';

          clippy = pkgs.runCommand "jtv-clippy" {
            nativeBuildInputs = nativeBuildInputs ++ buildInputs;
          } ''
            cd ${./.}
            cargo clippy --workspace -- -D warnings
            touch $out
          '';
        };

        # Formatter
        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
