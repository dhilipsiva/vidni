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
        # vidni's one version: the workspace's, in Cargo.toml.
        version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;
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

      # --- The portable bundle (P8-VIDNI-03) ---------------------------------
      # For a Linux without Nix: a small archive with vidni, and a large one
      # with the tools, which `vidni setup` downloads by its SHA-256 and
      # unpacks into the user's cache. Both carry store paths as they are in
      # this store; `vidni setup` moves the tools' to where it unpacks them.

      # The tools the bundle's vidni drives: the flake's own pins, in their
      # smaller builds. Piper is built without what trains voices and speaks
      # Japanese, and ffmpeg without what plays and what reads the network.
      bundlePiper =
        (pkgs.piper-tts.override {
          withAlignment = false;
          withHTTP = false;
          withJapanese = false;
          withTrain = false;
        }).overridePythonAttrs
          (old: {
            patches = (old.patches or [ ]) ++ [ ./nix/piper-seed.patch ];
            pythonImportsCheck = [ "piper" ];
            doCheck = false;
          });
      # whisper.cpp looks for the code that does its sums in the folder it
      # was built to find it in, and then beside its own program. Moved,
      # the first is nowhere, so the bundle's whisper has it beside it.
      bundleWhisper =
        pkgs.runCommand "whisper-cli-with-its-backends"
          {
            closure = pkgs.closureInfo { rootPaths = [ pkgs.whisper-cpp ]; };
          }
          ''
            mkdir -p $out/bin
            cp ${pkgs.whisper-cpp}/bin/whisper-cli $out/bin/
            backends=$(grep -- '-llama-cpp-' $closure/store-paths)
            ln -s $backends/bin/libggml-*.so $out/bin/
          '';
      bundleTools = [
        pkgs.ffmpeg-headless
        pkgs.chromium
        bundlePiper
        bundleWhisper
        pkgs.curl
        pkgs.util-linux
      ];

      # What the bundle's vidni is started with once the tools are set up.
      # `vidni setup` rewrites the store paths in it, as in every text file
      # of the tools.
      bundleEnv = pkgs.writeText "vidni-tools-env.sh" ''
        # Written by vidni's build; read by the bundle's `vidni`.
        PATH="${pkgs.lib.makeBinPath bundleTools}:$PATH"
        FONTCONFIG_FILE="${fontsConf}"
        VIDNI_FONTS="${fontDirs}"
        # The tools' curl checks a server against these, unless you name others.
        : "''${SSL_CERT_FILE:=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt}"
        export PATH FONTCONFIG_FILE VIDNI_FONTS SSL_CERT_FILE
      '';

      # The tools and everything they load, as one archive of store paths
      # under `store/` (the names only: a link's target stays as it is).
      # Sorted, with one time and one owner, read through hard links and
      # packed on one thread, so that it is the same bytes whenever it is
      # built.
      bundleToolsArchive =
        pkgs.runCommand "vidni-tools-x86_64-linux"
          {
            nativeBuildInputs = [ pkgs.zstd ];
            closure = pkgs.closureInfo { rootPaths = bundleTools ++ [ bundleEnv ]; };
          }
          ''
            mkdir -p $out
            sed 's,^/nix/store/,,' $closure/store-paths | LC_ALL=C sort > paths
            tar --create --directory /nix/store --files-from paths \
              --sort=name --mtime=@1 --owner=0 --group=0 --numeric-owner \
              --hard-dereference --mode=u+w --transform 'flags=r;s,^,store/,' \
              | zstd -15 -T1 > $out/vidni-tools-x86_64-linux.tar.zst
          '';

      # vidni itself, with what it loads, the curl it downloads with, the
      # patchelf that `vidni setup` moves the tools with, and the two
      # licences and the notices. Each program is started by a script that
      # hands it to the bundle's own loader, so it runs on any Linux.
      bundle =
        pkgs.runCommand "vidni-${vidni.version}-x86_64-linux"
          {
            closure = pkgs.closureInfo {
              rootPaths = [
                vidni
                pkgs.curl
                pkgs.patchelf
                pkgs.cacert
              ];
            };
            name_ = "vidni-${vidni.version}-x86_64-linux";
            # What the archives were built from: this commit of vidni, when
            # the tree is as committed, and the pinned nixpkgs.
            commit = self.rev or "a tree with changes that aren't committed";
            nixpkgs_ = nixpkgs.rev;
            toolsClosure = pkgs.closureInfo { rootPaths = bundleTools ++ [ bundleEnv ]; };
            url = "https://github.com/dhilipsiva/vidni/releases/download/v${vidni.version}";
            version_ = vidni.version;
          }
          ''
            root=$name_
            mkdir -p $root/bin $root/store $root/share/vidni $root/share/doc/vidni
            for path in $(cat $closure/store-paths); do
              cp -a $path $root/store/
            done
            chmod -R u+w $root/store
            # The desktop app isn't in the bundle: it opens its window with
            # the system's own graphics libraries, which a bundle can't carry.
            rm -f $root/store/$(basename ${vidni})/bin/vidni-app

            # The tools this bundle pins, by their SHA-256: `vidni setup`
            # unpacks them into a folder named by its first 16 digits.
            tools=${bundleToolsArchive}/vidni-tools-x86_64-linux.tar.zst
            sha=$(sha256sum $tools | cut -d' ' -f1)
            id=$(echo $sha | cut -c1-16)

            # Every folder of the bundle that holds a library, for the loader.
            libs=$(cd $root && find store -name '*.so*' -printf '%h\n' | LC_ALL=C sort -u \
              | sed 's,^,$here/,' | paste -sd:)
            loader=store/$(basename ${pkgs.glibc})/lib/ld-linux-x86-64.so.2
            starts() {
              cat > $root/bin/$1 <<EOF
            #!/bin/sh
            # $1, from vidni's portable bundle: started by the bundle's own
            # loader, with the bundle's own libraries, wherever the bundle is.
            self=\$0
            while [ -L "\$self" ]; do
              link=\$(readlink "\$self")
              case \$link in /*) self=\$link ;; *) self=\$(dirname "\$self")/\$link ;; esac
            done
            here=\$(CDPATH= cd -- "\$(dirname -- "\$self")/.." && pwd)
            $2
            exec "\$here/$loader" --library-path "$libs" "\$here/$3" "\$@"
            EOF
              sed -i 's/^            //' $root/bin/$1
              chmod +x $root/bin/$1
            }
            starts curl ': "''${SSL_CERT_FILE:=$here/store/'$(basename ${pkgs.cacert})'/etc/ssl/certs/ca-bundle.crt}"; export SSL_CERT_FILE' \
              store/$(basename ${pkgs.curl.bin})/bin/curl
            starts patchelf : store/$(basename ${pkgs.patchelf})/bin/patchelf
            starts vidni 'VIDNI_BUNDLE=$here; VIDNI_EXE=$here/bin/vidni; PATH=$here/bin:$PATH
            export VIDNI_BUNDLE VIDNI_EXE PATH
            # Sound devices, as the bundle'"'"'s own ALSA library lists them.
            : "''${ALSA_CONFIG_PATH:=$here/store/'$(basename ${pkgs.alsa-lib})'/share/alsa/alsa.conf}"
            export ALSA_CONFIG_PATH
            # The tools this vidni pins, once `vidni setup` has fetched them.
            tools=''${XDG_CACHE_HOME:-$HOME/.cache}/vidni/tools/'$id'/env.sh
            if [ -r "$tools" ]; then . "$tools"; fi' \
              store/$(basename ${vidni})/bin/vidni

            install -m644 -t $root/share/doc/vidni \
              ${./LICENSE-MIT} ${./LICENSE-APACHE} ${./THIRD-PARTY-NOTICES.md}
            for file in $root/share/doc/vidni/*; do
              mv $file $root/share/doc/vidni/$(basename $file | cut -d- -f2-)
            done
            install -m644 ${./packaging/README.txt} $root/README.txt
            # An example to draft: the one-minute video about vidni.
            mkdir -p $root/share/vidni/examples/hello
            install -m644 -t $root/share/vidni/examples/hello \
              ${./examples/hello/README.md} ${./examples/hello/flow.svg} \
              ${./examples/hello/vidni.toml} ${./examples/hello/voice.lock}
            for file in $root/share/vidni/examples/hello/*; do
              mv $file $root/share/vidni/examples/hello/$(basename $file | cut -d- -f2-)
            done

            # What `vidni setup` fetches: the tools, by their SHA-256.
            cat > $root/share/vidni/setup.json <<EOF
            {
              "tools": {
                "file": "vidni-tools-x86_64-linux.tar.zst",
                "url": "$url/vidni-tools-x86_64-linux.tar.zst",
                "sha256": "$sha",
                "bytes": $(stat -c %s $tools),
                "store": "/nix/store",
                "env": "store/$(basename ${bundleEnv})"
              },
              "voice": "piper:en_US-joe-medium",
              "listening_model": {
                "file": "ggml-base.en.bin",
                "url": "${whisperModel.url}",
                "sha256": "${whisperModel.outputHash}"
              }
            }
            EOF
            sed -i 's/^            //' $root/share/vidni/setup.json

            mkdir -p $out
            tar --create --sort=name --mtime=@1 --owner=0 --group=0 --numeric-owner \
              --hard-dereference $root \
              | gzip -9n > $out/$name_.tar.gz
            ln -s $tools $out/

            # What a release says of its archives' sources (the notices,
            # "The portable bundle").
            {
              echo "vidni $version_, the portable bundle for Linux (x86_64)"
              echo
              echo "Built by Nix from:"
              echo "  vidni    https://github.com/dhilipsiva/vidni  commit $commit"
              echo "  nixpkgs  https://github.com/NixOS/nixpkgs     commit $nixpkgs_"
              echo
              echo "Every package below is in the archives as Nix built it from those two."
              echo "THIRD-PARTY-NOTICES.md, in the bundle's share/doc/vidni, says how each"
              echo "one's licence and source are found."
              echo
              echo "$name_.tar.gz holds:"
              sed 's,^/nix/store/[a-z0-9]*-,  ,' $closure/store-paths | LC_ALL=C sort
              echo
              echo "vidni-tools-x86_64-linux.tar.zst holds:"
              sed 's,^/nix/store/[a-z0-9]*-,  ,' $toolsClosure/store-paths | LC_ALL=C sort
            } > $out/SOURCES.txt
          '';
    in
    {
      # `nix run github:dhilipsiva/vidni -- draft my-video` is the command
      # line, and `nix run github:dhilipsiva/vidni#app` the desktop app.
      packages.${system} = {
        default = vidniWithTools;
        inherit vidni bundle;
        bundle-tools = bundleToolsArchive;
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
