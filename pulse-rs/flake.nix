{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        is-darwin = pkgs.stdenv.isDarwin;
        darwin-packages = if is-darwin then [
          pkgs.libiconv
        ] else [ ];
        darwin-env = if is-darwin then ''
          export LIBRARY_PATH=$LIBRARY_PATH:${pkgs.libiconv}/lib
        '' else "";

      in {
        devShell = with pkgs; mkShell {
          packages = 
            let
              bazel-bin = writeShellScriptBin "bazel" "exec ${bazelisk}/bin/bazelisk $@";
              rust-toolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
            in [
              bazel-bin
              bazel-buildtools
              rust-toolchain
            ] ++ darwin-packages;
          shellHook = with pkgs; ''
            ${darwin-env}
          '';
        };
      }
    );
}
