{
  description = "vidni: an open, deterministic studio for narrated tutorial videos";

  # One pinned nixpkgs: rustc 1.99, Chromium 156 and Piper 1.8 (needed for
  # the en_US-joe-medium voice). It matches the maintainer's NixOS master pin.
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/27eb89b729375903ba8e21c79a19e408e432531e";

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};

      # Fonts come from the store, never from the host's fontconfig (DECISIONS D12).
      fontsConf = pkgs.makeFontsConf { fontDirectories = [ pkgs.ibm-plex ]; };

      rustTools = [
        pkgs.rustc
        pkgs.cargo
        pkgs.clippy
        pkgs.rustfmt
        pkgs.just
      ];

      # The external tools vidni drives, pinned by this flake (DECISIONS D3).
      mediaTools = [
        pkgs.ffmpeg-full
        pkgs.chromium
        pkgs.piper-tts
        pkgs.whisper-cpp
        pkgs.espeak-ng
      ];

      vidni = pkgs.rustPlatform.buildRustPackage {
        pname = "vidni";
        version = "0.1.0";
        src = self;
        cargoLock.lockFile = ./Cargo.lock;
      };
    in
    {
      packages.${system}.default = vidni;

      # `nix flake check` builds the workspace and runs its tests.
      checks.${system}.default = vidni;

      devShells.${system} = {
        default = pkgs.mkShell {
          packages = rustTools ++ mediaTools ++ [ pkgs.rust-analyzer ];
          FONTCONFIG_FILE = fontsConf;
        };
        # CI's shell: the Rust toolchain and just, without the large media tools.
        ci = pkgs.mkShell {
          packages = rustTools;
          FONTCONFIG_FILE = fontsConf;
        };
      };

      formatter.${system} = pkgs.nixfmt-rfc-style;
    };
}
