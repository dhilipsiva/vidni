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
        ${
          pkgs.lib.concatMapStrings (
            rule: "  <include>${pkgs.fontconfig.out}/etc/fonts/conf.d/${rule}.conf</include>\n"
          ) fontRules
        }  <cachedir prefix="xdg">fontconfig</cachedir>
        </fontconfig>
      '';

      # The folders that hold the fonts vidni draws with, each pinned by
      # SHA-256 in crates/vidni-core/src/fonts.toml. vidni looks for them in
      # $VIDNI_FONTS and nowhere else (P3-VIDNI-06).
      fontDirs = "${pkgs.ibm-plex}/share/fonts:${spaceGrotesk}/share/fonts";

      rustTools = [
        pkgs.rustc
        pkgs.cargo
        pkgs.clippy
        pkgs.rustfmt
        pkgs.just
        # fc-match and fc-list of the fontconfig whose rules fontsConf includes.
        pkgs.fontconfig
        # unshare, for --offline, and taskset, for `vidni repro`.
        pkgs.util-linux
      ];

      # Piper 1.8 draws its sampling noise inside the voice's ONNX graph, from
      # generators that onnxruntime seeds when a session starts. This patch
      # adds `--seed` to set that seed, so that a draft take repeats
      # (DECISIONS.md, T5).
      piper = pkgs.piper-tts.overridePythonAttrs (old: {
        patches = (old.patches or [ ]) ++ [ ./nix/piper-seed.patch ];
      });

      # The default draft voice, laid out as a voice store, with its files
      # pinned as crates/vidni-core/src/voices.toml pins them. The build's
      # tests draft with it (`just test-build`). A creator's own store is
      # theirs to fill: `vidni voices fetch`.
      voiceFile =
        name: sha256:
        pkgs.fetchurl {
          url = "https://huggingface.co/rhasspy/piper-voices/resolve/6b9cfaacb4fb2e02a9621227e3c1a48bec9710c3/en/en_US/joe/medium/${name}";
          inherit sha256;
        };
      pinnedVoices = pkgs.linkFarm "vidni-pinned-voices" [
        {
          name = "piper/en_US-joe-medium/en_US-joe-medium.onnx";
          path = voiceFile "en_US-joe-medium.onnx" "abf4b998e34045cbd00eaf050de2de620f81ded6eb8b85a24b022a1c2b3f08d2";
        }
        {
          name = "piper/en_US-joe-medium/en_US-joe-medium.onnx.json";
          path = voiceFile "en_US-joe-medium.onnx.json" "7b5f0293c0bf5bda8e377be1696da84af798befeef31038e43beefc01fe858da";
        }
      ];

      # The model of the listening check, pinned by revision and hash
      # (P3-VIDNI-05). vidni reads it from $VIDNI_WHISPER_MODEL.
      whisperModel = pkgs.fetchurl {
        url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-base.en.bin";
        sha256 = "a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002";
      };

      # The recorder reads the microphone through cpal, which on Linux talks
      # to ALSA; PipeWire and PulseAudio stand behind it (P4-VIDNI-01).
      soundLibs = [ pkgs.alsa-lib ];

      # The desktop app opens its window through winit and draws with
      # OpenGL. Both find these libraries when the app starts, not when
      # it is built, so they are named to the loader (P7-VIDNI-01).
      windowLibs = pkgs.lib.makeLibraryPath [
        pkgs.wayland
        pkgs.libxkbcommon
        pkgs.libGL
        pkgs.libx11
        pkgs.libxcursor
        pkgs.libxi
        pkgs.libxrandr
      ];

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
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = soundLibs;
        # The parity harness's frame tests and the build's tests drive ffmpeg.
        nativeCheckInputs = [ pkgs.ffmpeg-headless ];
        VIDNI_FONTS = fontDirs;
        # The licences and the third-party notices go wherever vidni goes
        # (P8-VIDNI-04).
        postInstall = ''
          install -Dm644 -t $out/share/doc/vidni LICENSE-MIT LICENSE-APACHE THIRD-PARTY-NOTICES.md
        '';
      };

      # vidni with the tools it drives: what `nix run` starts (P8-VIDNI-03).
      # The commands find the pinned tools first on their PATH, and the
      # pinned fonts and listening model where they look for them; the
      # desktop app finds the libraries it opens its window with. The build
      # and its tests stay the plain package above, which needs none of this.
      vidniWithTools = pkgs.symlinkJoin {
        name = "vidni-with-tools-${vidni.version}";
        paths = [ vidni ];
        nativeBuildInputs = [ pkgs.makeWrapper ];
        postBuild = ''
          for program in vidni vidni-app; do
            wrapProgram $out/bin/$program \
              --prefix PATH : ${
                pkgs.lib.makeBinPath (
                  mediaTools
                  ++ [
                    pkgs.util-linux
                    pkgs.xdg-utils
                  ]
                )
              } \
              --set-default FONTCONFIG_FILE ${fontsConf} \
              --set-default VIDNI_FONTS ${fontDirs} \
              --set-default VIDNI_WHISPER_MODEL ${whisperModel}
          done
          wrapProgram $out/bin/vidni-app --prefix LD_LIBRARY_PATH : ${windowLibs}
        '';
        meta.mainProgram = "vidni";
      };
    in
    {
      # `nix run github:dhilipsiva/vidni -- draft my-video` is the command
      # line, and `nix run github:dhilipsiva/vidni#app` the desktop app.
      packages.${system} = {
        default = vidniWithTools;
        inherit vidni;
      };
      apps.${system} = {
        default = {
          type = "app";
          program = "${vidniWithTools}/bin/vidni";
          meta.description = "vidni's command line, with its pinned tools";
        };
        app = {
          type = "app";
          program = "${vidniWithTools}/bin/vidni-app";
          meta.description = "vidni's desktop app, with its pinned tools";
        };
      };

      # `nix flake check` builds the workspace and runs its tests.
      checks.${system}.default = vidni;

      devShells.${system} = {
        default = pkgs.mkShell {
          packages = rustTools ++ mediaTools ++ [ pkgs.rust-analyzer ];
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = soundLibs;
          VIDNI_WINDOW_LIBS = windowLibs;
          FONTCONFIG_FILE = fontsConf;
          VIDNI_FONTS = fontDirs;
          VIDNI_PINNED_VOICES = pinnedVoices;
          VIDNI_WHISPER_MODEL = whisperModel;
        };
        # CI's shell: the Rust toolchain and just, plus a headless ffmpeg for the
        # parity and build tests, Chromium for the capture tests, whisper for
        # the listening tests and Piper for the build's, without the other
        # media tools.
        ci = pkgs.mkShell {
          packages = rustTools ++ [
            pkgs.ffmpeg-headless
            pkgs.chromium
            pkgs.whisper-cpp
            piper
          ];
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = soundLibs;
          FONTCONFIG_FILE = fontsConf;
          VIDNI_FONTS = fontDirs;
          VIDNI_PINNED_VOICES = pinnedVoices;
          VIDNI_WHISPER_MODEL = whisperModel;
        };
      };

      formatter.${system} = pkgs.nixfmt-rfc-style;
    };
}
