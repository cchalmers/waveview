{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      # inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        # When filtering sources, we want to allow assets other than .rs files
        src = lib.cleanSourceWith {
          src = ./.; # The original, unfiltered source
          filter = path: type:
            (lib.hasSuffix "\.html" path) ||
            (lib.hasSuffix "\.scss" path) ||
            # Example of a folder for images, icons, etc
            (lib.hasInfix "/assets/" path) ||
            # Default filter from crane (allow .rs files)
            (craneLibWasm.filterCargoSources path type)
          ;
        };

        rustToolchain = pkgs.rust-bin.stable."1.97.0".minimal.override {
          extensions = [ "clippy" "rustfmt" ];
          targets = [ "wasm32-unknown-unknown" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common arguments can be set here to avoid repeating them later
        # Note: changes here will rebuild all dependency crates
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          buildInputs = [
            # Add additional build inputs here
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];
        };

        my-crate = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          # Additional environment variables or build phases/hooks can be set
          # here *without* rebuilding all dependency crates
          # MY_CUSTOM_VAR = "some value";
        });

        # wasm

        craneLibWasm = craneLib;

        # Common arguments can be set here to avoid repeating them later
        commonArgsWasm = {
          inherit src;
          strictDeps = true;
          # We must force the target, otherwise cargo will attempt to use your native target
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

          buildInputs = [
            # Add additional build inputs here
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifactsWasm = craneLibWasm.buildDepsOnly (commonArgsWasm // {
          # You cannot run cargo test on a wasm build
          doCheck = false;
        });

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        # This derivation is a directory you can put on a webserver.
        my-app = craneLibWasm.buildTrunkPackage (commonArgsWasm // {
          inherit cargoArtifactsWasm;

          # Keep Cargo.toml's exact wasm-bindgen version aligned with nixpkgs.
          wasm-bindgen-cli = pkgs.wasm-bindgen-cli;
        });

        # Quick example on how to serve the app,
        # This is just an example, not useful for production environments
        serve-app = pkgs.writeShellScriptBin "serve-app" ''
          ${pkgs.python3Minimal}/bin/python3 -m http.server --directory ${my-app} 8000
        '';

        reload-waveview = pkgs.writeShellApplication {
          name = "reload-waveview";
          runtimeInputs = [ rustToolchain pkgs.entr pkgs.findutils ];
          text = ''
            cargo build -p waveview-ui-reload
            while true; do
              find waveview-ui waveview-ui-reload waveview-model -type f \
                \( -name '*.rs' -o -name Cargo.toml \) \
                | entr -dnp cargo build -p waveview-ui-reload \
                || true
            done &
            watcher_pid=$!
            trap 'kill "$watcher_pid" 2>/dev/null || true' EXIT INT TERM
            cargo run --features reload -- "$@"
          '';
        };
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          native = my-crate;
          wasm = my-app;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          wasm-clippy = craneLibWasm.cargoClippy (commonArgsWasm // {
            cargoArtifacts = cargoArtifactsWasm;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          # Check formatting
          fmt = craneLibWasm.cargoFmt {
            inherit src;
          };

          native-clippy = craneLib.cargoClippy (commonArgs // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          reload-clippy = craneLib.cargoClippy (commonArgs // {
            cargoArtifacts = craneLib.buildDepsOnly commonArgs;
            cargoClippyExtraArgs = "--features reload --all-targets -- --deny warnings";
          });
        };

        packages.default = my-app;

        apps.default = flake-utils.lib.mkApp {
          drv = serve-app;
        };

        apps.native = flake-utils.lib.mkApp {
          drv = my-crate;
        };

        apps.reload = flake-utils.lib.mkApp {
          drv = reload-waveview;
        };

        devShells = {
          default = craneLib.devShell {
            # Trunk parses NO_COLOR as a boolean rather than following the usual presence-only convention.
            NO_COLOR = "true";
            # Inherit inputs from checks.
            # checks = self.checks.${system};

            # Additional dev-shell environment variables can be set directly
            # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

            # Extra inputs can be added here; cargo and rustc are provided by default.
            packages = with pkgs; [
              bacon
              reload-waveview
            ] ++ lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
          };

          wasm = craneLibWasm.devShell {
            NO_COLOR = "true";
            # Additional dev-shell environment variables can be set directly
            # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

            # Extra inputs can be added here; cargo and rustc are provided by default.
            packages = with pkgs; [
              trunk
              bacon
            ];
          };
        };

      });
}
