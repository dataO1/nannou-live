{
  description = "nannou-live — audio-reactive live visuals with scene switching";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        commonArgs = {
          pname = "nannou-live";
          version = "0.1.0";
          src = pkgs.lib.cleanSource ./.;
          buildInputs = with pkgs; [
            openssl
            alsa-lib
            vulkan-loader
            libx11
            libxcursor
            libxrandr
            libxi
            libxkbcommon
            wayland
            mesa
            udev
          ];
          nativeBuildInputs = with pkgs; [
            pkg-config
            makeWrapper
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          cargoVendorDir = null;
        });

        nannou-live = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          cargoVendorDir = null;
          postInstall = ''
            wrapProgram $out/bin/nannou-live \
              --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath [
                pkgs.wayland
                pkgs.libxkbcommon
                pkgs.mesa
                pkgs.vulkan-loader
                pkgs.alsa-lib
              ]}
          '';
        });
      in
      {
        packages = {
          default = nannou-live;
          inherit nannou-live;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ nannou-live ];
          packages = with pkgs; [
            rustToolchain
            cargo
            rust-analyzer
            nil
            nixpkgs-fmt
          ];

          shellHook = ''
            echo "🔮 nannou-live devshell"
            echo "   cargo run --release    build + launch"
          '';
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
