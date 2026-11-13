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

      # Fonts come from the store, never from the host (DECISIONS D12, T6):
      # only these font folders, and fontconfig's own rules. Its two rules
      # that read the user's and the host's configuration are left out, and
      # so is `makeFontsConf`, whose file includes /etc/fonts and the host's
      # font folders. IBM Plex is the default look; Space Grotesk is the nibli
      # example's display face; DejaVu and Liberation are what pages fall
      # back to for symbols and for Arial, Times New Roman and Courier New;
      # Noto covers other scripts (Tamil among them) and emoji. Chinese,
      # Japanese and Korean aren't covered: those fonts are very large.
      # Space Grotesk (OFL), the nibli example's display face; nixpkgs has no
      # package of it.
      spaceGrotesk = pkgs.stdenvNoCC.mkDerivation {
        pname = "space-grotesk";
        version = "2.0.0";
        src = pkgs.fetchzip {
          url = "https://github.com/floriankarsten/space-grotesk/releases/download/2.0.0/SpaceGrotesk-2.0.0.zip";
          hash = "sha256-niwd5E3rJdGmoyIFdNcK5M9A9P2rCbpsyZCl7CDv7I8=";
          stripRoot = false;
        };
        installPhase = ''
          install -Dm644 -t $out/share/fonts/opentype SpaceGrotesk-2.0.0/otf/*.otf
          install -Dm644 -t $out/share/doc/space-grotesk SpaceGrotesk-2.0.0/OFL.txt
        '';
      };

      fontRules = [
        "10-hinting-slight"
        "10-scale-bitmap-fonts"
        "10-sub-pixel-none"
        "10-yes-antialias"
        "11-lcdfilter-default"
        "20-unhint-small-vera"
        "30-metric-aliases"
        "40-nonlatin"
        "45-generic"
        "45-latin"
        "48-guessfamily"
        "48-spacing"
        "49-sansserif"
        "60-generic"
        "60-latin"
        "65-fonts-persian"
        "65-nonlatin"
        "69-unifont"
        "70-no-bitmaps-except-emoji"
        "80-delicious"
        "90-synthetic"
      ];
      fontsConf = pkgs.writeText "vidni-fonts.conf" ''
        <?xml version="1.0"?>
        <!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
        <fontconfig>
          <dir>${pkgs.ibm-plex}/share/fonts</dir>
          <dir>${spaceGrotesk}/share/fonts</dir>
          <dir>${pkgs.dejavu_fonts}/share/fonts</dir>
          <dir>${pkgs.liberation_ttf}/share/fonts</dir>
          <dir>${pkgs.noto-fonts}/share/fonts</dir>
          <dir>${pkgs.noto-fonts-color-emoji}/share/fonts</dir>
        ${pkgs.lib.concatMapStrings (
          rule: "  <include>${pkgs.fontconfig.out}/etc/fonts/conf.d/${rule}.conf</include>\n"
        ) fontRules}  <cachedir prefix="xdg">fontconfig</cachedir>
        </fontconfig>
      '';

      rustTools = [
        pkgs.rustc
        pkgs.cargo
        pkgs.clippy
        pkgs.rustfmt
        pkgs.just
        # fc-match and fc-list of the fontconfig whose rules fontsConf includes.
        pkgs.fontconfig
      ];

      # Piper 1.8 draws its sampling noise inside the voice's ONNX graph, from
      # generators that onnxruntime seeds when a session starts. This patch
      # adds `--seed` to set that seed, so that a draft take repeats
      # (DECISIONS.md, T5).
      piper = pkgs.piper-tts.overridePythonAttrs (old: {
        patches = (old.patches or [ ]) ++ [ ./nix/piper-seed.patch ];
      });

      # The external tools vidni drives, pinned by this flake (DECISIONS D3).
      mediaTools = [
        pkgs.ffmpeg-full
        pkgs.chromium
        piper
        pkgs.whisper-cpp
        pkgs.espeak-ng
        pkgs.curl
      ];

      vidni = pkgs.rustPlatform.buildRustPackage {
        pname = "vidni";
        version = "0.1.0";
        src = self;
        cargoLock.lockFile = ./Cargo.lock;
        # The parity harness's frame tests drive ffmpeg.
        nativeCheckInputs = [ pkgs.ffmpeg-headless ];
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
        # CI's shell: the Rust toolchain and just, plus a headless ffmpeg for the
        # parity tests and Chromium for the capture tests, without the other
        # media tools.
        ci = pkgs.mkShell {
          packages = rustTools ++ [
            pkgs.ffmpeg-headless
            pkgs.chromium
          ];
          FONTCONFIG_FILE = fontsConf;
        };
      };

      formatter.${system} = pkgs.nixfmt-rfc-style;
    };
}
