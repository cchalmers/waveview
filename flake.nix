{
  inputs = {
    # nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.05";
    nixpkgs.url = "github:NixOS/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";

    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    cargo2nix.inputs.nixpkgs.follows = "nixpkgs";
    cargo2nix.inputs.flake-utils.follows = "flake-utils";
    cargo2nix.inputs.rust-overlay.follows = "rust-overlay";

    gitignore.url = "github:hercules-ci/gitignore.nix";
    gitignore.inputs.nixpkgs.follows = "nixpkgs";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
  };

  outputs = { self, nixpkgs, gitignore ,cargo2nix, rust-overlay, flake-utils }:
   flake-utils.lib.eachDefaultSystem (system:
    let pkgs = import nixpkgs {
          inherit system;
          overlays = [ cargo2nix.overlays.default ];
        };
        inherit (gitignore.lib) gitignoreSource;
    in with pkgs; {
      devShell = mkShell {
        buildInputs = [
          trunk
          wasm-bindgen-cli
          libiconv
          darwin.apple_sdk.frameworks.AppKit
          cargo2nix.packages.${system}.cargo2nix
        ];
        shellHook = ''
          echo "welcome to the waveview shell"
        '';
      };

      packages = {

        waveview = let
          rustPkgs = rustBuilder.makePackageSet {
            rustVersion = "1.63.0";
            packageFun = import ./Cargo.nix;
            packageOverrides = pkgs: pkgs.rustBuilder.overrides.all ++ [

                (pkgs.rustBuilder.rustLib.makeOverride {
                    name = "waveview";
                    overrideAttrs = drv: {
                      buildInputs = drv.buildInputs or [ ] ++ [
                        darwin.apple_sdk.frameworks.AppKit
                      ];
                    };
                })
            ];
          };
        in (rustPkgs.workspace.waveview {}).bin;

      };
  });
}
