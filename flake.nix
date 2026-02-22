{
  description = "Rscanner Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Choose your Rust version:
        # - rust-bin.stable.latest.default (latest stable)
        # - rust-bin.beta.latest.default (beta)
        # - rust-bin.nightly.latest.default (nightly)
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain

            # Additional development tools
            cargo-watch
            cargo-edit
            cargo-audit
            clippy
            rustfmt

            # Build dependencies (common ones)
            pkg-config
            openssl
            #udev  # provides libudev for device management

            # Optional: useful utilities
            bacon  # background code checker
          ];

          shellHook = ''
            echo "Rust development environment"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"

            # Ensure cargo home exists
            mkdir -p $HOME/.cargo
          '';

          # Environment variables
          RUST_BACKTRACE = "1";
          CARGO_HOME = "$HOME/.cargo";
        };
      }
    );
}
