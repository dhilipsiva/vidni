# Third-party notices

vidni itself is licensed under MIT OR Apache-2.0 ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)).

This file lists what vidni is built from, what it runs, and what it reads, each with its licence. It is generated: `just notices` writes it again from `Cargo.lock`, `fonts.toml`, `voices.toml`, `flake.nix` and `flake.lock`, and `just notices check` fails when it is out of date. Don't edit it by hand.

A video you make with vidni is yours. Two things in it come from others: the fonts its text is drawn in, and, until you record your own takes, the draft voice. Their licences are below.

## Programs vidni runs

vidni runs these as separate programs. None of them is part of the `vidni` binary, and vidni's own licence doesn't change theirs. The versions are the ones vidni's flake pins (nixpkgs `27eb89b729375903ba8e21c79a19e408e432531e`).

| Program | Version | Licence | What vidni uses it for | Home |
|---|---|---|---|---|
| ffmpeg | 9.0.2 | LGPL-2.1-or-later, GPL-2.0-or-later, LGPL-3.0-or-later, GPL-3.0-or-later | encodes each cut, measures it, and normalizes loudness | https://www.ffmpeg.org/ |
| x264 | 0-unstable-2025-09-10 | GPL-2.0-or-later | the H.264 encoder inside that ffmpeg | http://www.videolan.org/developers/x264.html |
| Chromium | 156.0.8078.79 | BSD-3-Clause | captures web and page scenes | https://www.chromium.org/ |
| Piper | 1.8.0 | GPL-3.0-only | speaks the draft voice | https://github.com/OHF-Voice/piper1-gpl |
| eSpeak NG | 1.52.0.1-unstable-2025-09-09 | GPL-3.0-or-later | turns text into phonemes for Piper | https://github.com/espeak-ng/espeak-ng |
| ONNX Runtime | 1.27.1 | MIT | runs Piper's voice models | https://github.com/microsoft/onnxruntime |
| whisper.cpp | 1.9.5 | MIT | listens to takes for `vidni hear` | https://github.com/ggerganov/whisper.cpp |
| curl | 8.22.0 | curl | downloads a voice for `vidni voices fetch` | https://curl.se/ |

### The GPL programs, and their source

ffmpeg, x264, Piper, eSpeak NG are under the GNU General Public License. The ffmpeg that vidni pins is built with `--enable-gpl` and libx264, which makes the whole of that ffmpeg GPL-3.0-or-later.

- **Running them changes nothing for you.** vidni starts each as a program of its own, so their licence covers neither vidni nor your videos.
- **Passing them on does.** Anyone who gives another person a copy of one of these programs, alone or inside a bundle with vidni, must give that person its complete corresponding source as well, or a written offer of it, under the same licence (GPL version 3, section 6).
- **Where the source is.** Each program is built from the recipe in nixpkgs at revision `27eb89b729375903ba8e21c79a19e408e432531e`. `nix build --inputs-from . nixpkgs#<name>.src` fetches the exact source, with <name> one of: `ffmpeg-full`, `x264`, `piper-tts`, `espeak-ng`. The recipes and their patches are at <https://github.com/NixOS/nixpkgs/tree/27eb89b729375903ba8e21c79a19e408e432531e>, and vidni's one patch to Piper is [`nix/piper-seed.patch`](nix/piper-seed.patch).

Chromium is under a BSD-style licence and includes many other projects; its own list is at `chrome://credits` in the browser.

## The portable bundle

The portable bundle for Linux ([docs/install.md](docs/install.md)) is two archives, and both pass programs on:

- **`vidni-<version>-x86_64-linux.tar.gz`** holds vidni, and what starts it on a Linux with no Nix: the GNU C Library and GCC's runtime libraries, ALSA's library, curl with the libraries it loads (OpenSSL and MIT Kerberos among them) and Mozilla's list of certificate authorities, patchelf, Bash and GNU coreutils, and the libraries those load in turn.
- **`vidni-tools-x86_64-linux.tar.zst`**, which `vidni setup` fetches, holds the programs of the table above in the builds the bundle uses (ffmpeg without what plays and what reads the network, Piper without what trains voices), the fonts, and everything those programs load: some 400 packages, from the C library to Python.

Every package in either archive is in it exactly as Nix built it from nixpkgs at revision `27eb89b729375903ba8e21c79a19e408e432531e`, by vidni's `flake.nix`. Their folders keep the names Nix gave them, `<hash>-<name>-<version>`, so each can be traced:

- **What is in an archive:** its packages are the folders under `store/`. `tar tf vidni-tools-x86_64-linux.tar.zst | cut -d/ -f2 | sort -u` lists the tools' archive's, and `tar tzf vidni-<version>-x86_64-linux.tar.gz | grep /store/ | cut -d/ -f3 | sort -u` the other's.
- **A package's licence** is in its recipe in nixpkgs, under `meta.license`: <https://search.nixos.org/packages> finds the recipe by name.
- **A package's source:** `nix derivation show /nix/store/<its folder>` names the source its build fetched, pinned by hash, and its patches; `nix build --inputs-from . nixpkgs#<name>.src` fetches it. The recipes are at <https://github.com/NixOS/nixpkgs/tree/27eb89b729375903ba8e21c79a19e408e432531e>.

Many of these packages are under the GPL or the LGPL (the C library, GCC's runtime, Bash, coreutils, patchelf, ffmpeg, x264, Piper and eSpeak NG among them). **Whoever gives either archive to someone else must offer that person the source of each of them,** as the section above says. The lines above are how the source is found; a release of vidni that carries the archives names the vidni commit and the nixpkgs revision they were built from, which fixes every source.

## Fonts

vidni draws text in these fonts, and their outlines end up in your video's frames. All are under the SIL Open Font License 1.1 (<https://openfontlicense.org>), which allows that, for any purpose, and asks that the font files themselves are passed on only with their licence and under their own names.

| Font | Version | Licence | Copyright | Source |
|---|---|---|---|---|
| IBM Plex Sans Bold | 1.1.0 | OFL-1.1 | Copyright 2018 IBM Corp. All rights reserved. | https://github.com/IBM/plex |
| IBM Plex Mono Bold | 2.5.0 | OFL-1.1 | Copyright 2017 IBM Corp. All rights reserved. | https://github.com/IBM/plex |
| Space Grotesk Bold | 2.0.0 | OFL-1.1 | Copyright 2020 The Space Grotesk Project Authors (https://github.com/floriankarsten/space-grotesk) | https://github.com/floriankarsten/space-grotesk |

## Draft voices

A draft voice is a model trained on recordings of a person, and its licence is its training data's. It decides what you may do with audio the voice speaks. `vidni voices` prints this list, and `vidni validate` warns when a project's draft voice doesn't allow commercial use.

| Voice | Licence | Commercial use | Notes | Model card |
|---|---|---|---|---|
| piper:en_US-joe-medium | CC0-1.0 | yes | Trained on the OHF-Voice voice datasets (CC0), fine-tuned from the rdh base model (CC0). | https://huggingface.co/rhasspy/piper-voices/blob/6b9cfaacb4fb2e02a9621227e3c1a48bec9710c3/en/en_US/joe/medium/MODEL_CARD |
| piper:en_US-ryan-medium | CC-BY-NC-SA-4.0 | no | Trained on the RyanSpeech dataset (CC BY-NC-SA 4.0), fine-tuned from the rdh base model (CC0). | https://huggingface.co/rhasspy/piper-voices/blob/6b9cfaacb4fb2e02a9621227e3c1a48bec9710c3/en/en_US/ryan/medium/MODEL_CARD |

## Models

| Model | Licence | Notes | Source | SHA-256 |
|---|---|---|---|---|
| ggml-base.en.bin | MIT | OpenAI's Whisper base.en model, converted for whisper.cpp. | https://github.com/openai/whisper | a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002 |

`vidni hear` listens with this model. Nothing it hears changes an output.

## Rust crates

The `vidni` binaries are built from these 324 crates, each under the licence its authors chose (22 licence expressions in all). Each crate's own licence files are in its source, at the address given.

| Crate | Version | Licence | Source |
|---|---|---|---|
| accesskit | 0.25.2 | MIT OR Apache-2.0 | https://github.com/AccessKit/accesskit |
| accesskit_atspi_common | 0.21.1 | MIT OR Apache-2.0 | https://github.com/AccessKit/accesskit |
| accesskit_consumer | 0.39.2 | MIT OR Apache-2.0 | https://github.com/AccessKit/accesskit |
| accesskit_unix | 0.24.1 | MIT OR Apache-2.0 | https://github.com/AccessKit/accesskit |
| accesskit_winit | 0.34.2 | Apache-2.0 | https://github.com/AccessKit/accesskit |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | https://github.com/oyvindln/adler2 |
| ahash | 0.8.12 | MIT OR Apache-2.0 | https://github.com/tkaitchuck/ahash |
| aho-corasick | 1.1.5 | Unlicense OR MIT | https://github.com/BurntSushi/aho-corasick |
| alsa | 0.11.0 | Apache-2.0/MIT | https://github.com/diwic/alsa-rs |
| alsa-sys | 0.4.0 | MIT | https://github.com/diwic/alsa-sys |
| anstream | 1.0.0 | MIT OR Apache-2.0 | https://github.com/rust-cli/anstyle.git |
| anstyle | 1.0.14 | MIT OR Apache-2.0 | https://github.com/rust-cli/anstyle.git |
| anstyle-parse | 1.0.0 | MIT OR Apache-2.0 | https://github.com/rust-cli/anstyle.git |
| anstyle-query | 1.1.5 | MIT OR Apache-2.0 | https://github.com/rust-cli/anstyle.git |
| arboard | 3.7.0 | MIT OR Apache-2.0 | https://github.com/1Password/arboard |
| arrayref | 0.3.9 | BSD-2-Clause | https://github.com/droundy/arrayref |
| arrayvec | 0.7.8 | MIT OR Apache-2.0 | https://github.com/bluss/arrayvec |
| as-raw-xcb-connection | 1.0.1 | MIT OR Apache-2.0 | https://github.com/psychon/as-raw-xcb-connection |
| async-broadcast | 0.7.2 | MIT OR Apache-2.0 | https://github.com/smol-rs/async-broadcast |
| async-channel | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-channel |
| async-executor | 1.14.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-executor |
| async-io | 2.6.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-io |
| async-lock | 3.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-lock |
| async-process | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-process |
| async-recursion | 1.2.0 | MIT OR Apache-2.0 | https://github.com/dcchut/async-recursion |
| async-signal | 0.2.14 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-signal |
| async-task | 4.7.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-task |
| async-trait | 0.1.92 | MIT OR Apache-2.0 | https://github.com/dtolnay/async-trait |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT | https://github.com/smol-rs/atomic-waker |
| atspi | 0.29.0 | Apache-2.0 OR MIT | https://github.com/odilia-app/atspi |
| atspi-common | 0.13.0 | Apache-2.0 OR MIT | https://github.com/odilia-app/atspi |
| atspi-proxies | 0.13.0 | Apache-2.0 OR MIT | https://github.com/odilia-app/atspi |
| base64 | 0.23.1 | MIT OR Apache-2.0 | https://github.com/marshallpierce/rust-base64 |
| bit-set | 0.10.0 | Apache-2.0 OR MIT | https://github.com/contain-rs/bit-set |
| bit-vec | 0.9.1 | Apache-2.0 OR MIT | https://github.com/contain-rs/bit-vec |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | https://github.com/bitflags/bitflags |
| block-buffer | 0.12.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| blocking | 1.7.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/blocking |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| bytemuck_derive | 1.12.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| byteorder-lite | 0.1.0 | Unlicense OR MIT | https://github.com/image-rs/byteorder-lite |
| calloop | 0.13.0 | MIT | https://github.com/Smithay/calloop |
| calloop-wayland-source | 0.3.0 | MIT | https://github.com/smithay/calloop-wayland-source |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/cfg-if |
| clap | 4.6.7 | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| clap_builder | 4.6.7 | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| clap_derive | 4.6.7 | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| clap_lex | 1.1.1 | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| codespan-reporting | 0.13.1 | Apache-2.0 | https://github.com/brendanzab/codespan |
| color | 0.3.3 | Apache-2.0 OR MIT | https://github.com/linebender/color |
| colorchoice | 1.0.5 | MIT OR Apache-2.0 | https://github.com/rust-cli/anstyle.git |
| command-fds | 0.3.3 | Apache-2.0 | https://github.com/google/command-fds/ |
| concurrent-queue | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/concurrent-queue |
| const-oid | 0.10.3 | Apache-2.0 OR MIT | https://github.com/RustCrypto/formats |
| convert_case | 0.10.0 | MIT | https://github.com/rutrum/convert-case |
| core_maths | 0.1.1 | MIT | https://github.com/robertbastian/core_maths |
| cosmic-text | 0.19.0 | MIT OR Apache-2.0 | https://github.com/pop-os/cosmic-text |
| cpal | 0.17.3 | Apache-2.0 | https://github.com/RustAudio/cpal |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| crc32fast | 1.5.5 | MIT OR Apache-2.0 | https://github.com/srijs/rust-crc32fast |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |
| crossterm | 0.29.0 | MIT | https://github.com/crossterm-rs/crossterm |
| crypto-common | 0.2.2 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| ctrlc | 3.5.2 | MIT/Apache-2.0 | https://github.com/Detegr/rust-ctrlc.git |
| cursor-icon | 1.2.0 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/cursor-icon |
| dasp_sample | 0.11.0 | MIT OR Apache-2.0 | https://github.com/rustaudio/sample.git |
| data-url | 0.3.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url |
| derive_more | 2.1.1 | MIT | https://github.com/JelteF/derive_more |
| derive_more-impl | 2.1.1 | MIT | https://github.com/JelteF/derive_more |
| digest | 0.11.3 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| dlib | 0.5.3 | MIT | https://github.com/elinorbgr/dlib |
| document-features | 0.2.12 | MIT OR Apache-2.0 | https://github.com/slint-ui/document-features |
| downcast-rs | 1.2.1 | MIT/Apache-2.0 | https://github.com/marcianx/downcast-rs |
| dpi | 0.1.2 | Apache-2.0 AND MIT | https://github.com/rust-windowing/winit |
| dyn-clone | 1.0.20 | MIT OR Apache-2.0 | https://github.com/dtolnay/dyn-clone |
| ecolor | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| eframe | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/eframe |
| egui | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| egui-wgpu | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/egui-wgpu |
| egui-winit | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/egui-winit |
| egui_glow | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/egui_glow |
| either | 1.19.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/either |
| emath | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/emath |
| endi | 1.1.1 | MIT | https://github.com/zeenix/endi |
| enumflags2 | 0.7.12 | MIT OR Apache-2.0 | https://github.com/meithecatte/enumflags2 |
| enumflags2_derive | 0.7.12 | MIT OR Apache-2.0 | https://github.com/meithecatte/enumflags2 |
| epaint | 0.37.0 | MIT OR Apache-2.0 | https://github.com/emilk/egui/tree/main/crates/epaint |
| epaint_default_fonts | 0.37.0 | (MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0 | https://github.com/emilk/egui/tree/main/crates/epaint_default_fonts |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/equivalent |
| errno | 0.3.15 | MIT OR Apache-2.0 | https://github.com/lambda-fairy/rust-errno |
| euclid | 0.22.14 | MIT OR Apache-2.0 | https://github.com/servo/euclid |
| event-listener | 5.4.2 | Apache-2.0 OR MIT | https://github.com/smol-rs/event-listener |
| event-listener-strategy | 0.5.4 | Apache-2.0 OR MIT | https://github.com/smol-rs/event-listener-strategy |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/fastrand |
| fax | 0.2.7 | MIT | https://github.com/pdf-rs/fax |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | https://github.com/image-rs/fdeflate |
| fearless_simd | 0.4.1 | Apache-2.0 OR MIT | https://github.com/linebender/fearless_simd |
| filetime | 0.2.29 | MIT/Apache-2.0 | https://github.com/alexcrichton/filetime |
| fixedbitset | 0.5.9 | MIT OR Apache-2.0 | https://github.com/petgraph/fixedbitset |
| flate2 | 1.1.11 | MIT OR Apache-2.0 | https://github.com/rust-lang/flate2-rs |
| float-cmp | 0.9.0 | MIT | https://github.com/mikedilger/float-cmp |
| foldhash | 0.1.5 | Zlib | https://github.com/orlp/foldhash |
| foldhash | 0.2.0 | Zlib | https://github.com/orlp/foldhash |
| font-types | 0.11.3 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| font-types | 0.12.7 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| fontdb | 0.23.0 | MIT | https://github.com/RazrFalcon/fontdb |
| fontdb | 0.24.0 | MIT | https://github.com/RazrFalcon/fontdb |
| futures-core | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| futures-io | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| futures-lite | 2.6.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/futures-lite |
| futures-macro | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| futures-task | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| futures-util | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| gethostname | 1.1.0 | Apache-2.0 | https://codeberg.org/swsnr/gethostname.rs.git |
| getrandom | 0.3.4 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| glifo | 0.3.0 | Apache-2.0 OR MIT | https://github.com/linebender/vello |
| glow | 0.17.0 | MIT OR Apache-2.0 OR Zlib | https://github.com/grovesNL/glow |
| glutin | 0.32.3 | Apache-2.0 | https://github.com/rust-windowing/glutin |
| glutin-winit | 0.5.0 | MIT | https://github.com/rust-windowing/glutin |
| glutin_egl_sys | 0.7.1 | Apache-2.0 | https://github.com/rust-windowing/glutin |
| glutin_glx_sys | 0.6.1 | Apache-2.0 | https://github.com/rust-windowing/glutin |
| guillotiere | 0.7.0 | MIT/Apache-2.0 | https://github.com/nical/guillotiere |
| half | 2.7.1 | MIT OR Apache-2.0 | https://github.com/VoidStarKat/half-rs |
| harfrust | 0.12.0 | MIT | https://github.com/harfbuzz/harfrust |
| harfrust | 0.5.2 | MIT | https://github.com/harfbuzz/harfrust |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| heck | 0.5.0 | MIT OR Apache-2.0 | https://github.com/withoutboats/heck |
| hex | 0.4.3 | MIT OR Apache-2.0 | https://github.com/KokaKiwi/rust-hex |
| hound | 3.5.1 | Apache-2.0 | https://github.com/ruuda/hound |
| hybrid-array | 0.4.15 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hybrid-array |
| image | 0.25.10 | MIT OR Apache-2.0 | https://github.com/image-rs/image |
| imagesize | 0.15.0 | MIT | https://github.com/Roughsketch/imagesize |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/indexmap |
| is_terminal_polyfill | 1.70.2 | MIT OR Apache-2.0 | https://github.com/polyfill-rs/is_terminal_polyfill |
| itertools | 0.15.0 | MIT OR Apache-2.0 | https://github.com/rust-itertools/itertools |
| itoa | 1.0.19 | MIT OR Apache-2.0 | https://github.com/dtolnay/itoa |
| kurbo | 0.13.1 | Apache-2.0 OR MIT | https://github.com/linebender/kurbo |
| libc | 0.2.191 | MIT OR Apache-2.0 | https://github.com/rust-lang/libc |
| libloading | 0.8.9 | ISC | https://github.com/nagisa/rust_libloading/ |
| libm | 0.2.16 | MIT | https://github.com/rust-lang/compiler-builtins |
| linebender_resource_handle | 0.1.1 | Apache-2.0 OR MIT | https://github.com/linebender/raw_resource_handle |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/sunfishcode/linux-raw-sys |
| linux-raw-sys | 0.4.15 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/sunfishcode/linux-raw-sys |
| litrs | 1.0.1 | MIT OR Apache-2.0 | https://github.com/LukasKalbertodt/litrs |
| lock_api | 0.4.14 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| log | 0.4.35 | MIT OR Apache-2.0 | https://github.com/rust-lang/log |
| memchr | 2.8.3 | Unlicense OR MIT | https://github.com/BurntSushi/memchr |
| memmap2 | 0.9.12 | MIT OR Apache-2.0 | https://github.com/RazrFalcon/memmap2-rs |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| mio | 1.2.5 | MIT | https://github.com/tokio-rs/mio |
| moxcms | 0.8.1 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/moxcms.git |
| naga | 30.0.2 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| naga-types | 30.0.2 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| nix | 0.31.3 | MIT | https://github.com/nix-rust/nix |
| nohash-hasher | 0.2.0 | Apache-2.0 OR MIT | https://github.com/paritytech/nohash-hasher |
| nom | 8.0.0 | MIT | https://github.com/rust-bakery/nom |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | https://github.com/rust-num/num-traits |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | https://github.com/matklad/once_cell |
| ordered-stream | 0.2.0 | MIT OR Apache-2.0 | https://github.com/danieldg/ordered-stream |
| os_pipe | 1.2.3 | MIT | https://github.com/oconnor663/os_pipe.rs |
| parking | 2.2.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/parking |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| peniko | 0.6.1 | Apache-2.0 OR MIT | https://github.com/linebender/peniko |
| percent-encoding | 2.3.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url/ |
| petgraph | 0.8.3 | MIT OR Apache-2.0 | https://github.com/petgraph/petgraph |
| phf | 0.14.0 | MIT | https://github.com/rust-phf/rust-phf |
| phf_generator | 0.14.0 | MIT | https://github.com/rust-phf/rust-phf |
| phf_macros | 0.14.0 | MIT | https://github.com/rust-phf/rust-phf |
| phf_shared | 0.14.0 | MIT | https://github.com/rust-phf/rust-phf |
| pico-args | 0.5.0 | MIT | https://github.com/RazrFalcon/pico-args |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | https://github.com/taiki-e/pin-project-lite |
| piper | 0.2.5 | MIT OR Apache-2.0 | https://github.com/smol-rs/piper |
| png | 0.18.1 | MIT OR Apache-2.0 | https://github.com/image-rs/image-png |
| polling | 3.11.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/polling |
| polycool | 0.4.0 | MIT OR Apache-2.0 | https://github.com/linebender/kurbo |
| proc-macro-crate | 3.5.0 | MIT OR Apache-2.0 | https://github.com/bkchr/proc-macro-crate |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | https://github.com/dtolnay/proc-macro2 |
| profiling | 1.0.18 | MIT OR Apache-2.0 | https://github.com/aclysma/profiling |
| pxfm | 0.1.30 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/pxfm |
| quick-error | 2.0.1 | MIT/Apache-2.0 | http://github.com/tailhook/quick-error |
| quick-xml | 0.41.0 | MIT | https://github.com/tafia/quick-xml |
| quote | 1.0.47 | MIT OR Apache-2.0 | https://github.com/dtolnay/quote |
| rand_core | 0.10.1 | MIT OR Apache-2.0 | https://github.com/rust-random/rand_core |
| rangemap | 1.8.0 | MIT/Apache-2.0 | https://github.com/jeffparsons/rangemap |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/raw-window-handle |
| read-fonts | 0.37.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| read-fonts | 0.41.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| ref-cast | 1.0.27 | MIT OR Apache-2.0 | https://github.com/dtolnay/ref-cast |
| ref-cast-impl | 1.0.27 | MIT OR Apache-2.0 | https://github.com/dtolnay/ref-cast |
| regex | 1.13.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| regress | 0.12.0 | MIT OR Apache-2.0 | https://github.com/ridiculousfish/regress |
| renderdoc-sys | 1.1.0 | MIT OR Apache-2.0 | https://github.com/ebkalderon/renderdoc-rs |
| resvg | 0.49.0 | Apache-2.0 OR MIT | https://github.com/linebender/resvg |
| rgb | 0.8.53 | MIT | https://github.com/kornelski/rust-rgb |
| roxmltree | 0.21.2 | MIT OR Apache-2.0 | https://github.com/RazrFalcon/roxmltree |
| rustc-hash | 1.1.0 | Apache-2.0/MIT | https://github.com/rust-lang-nursery/rustc-hash |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT | https://github.com/rust-lang/rustc-hash |
| rustix | 0.38.44 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/rustix |
| rustix | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/rustix |
| ruzstd | 0.8.3 | MIT | https://github.com/KillingSpark/zstd-rs |
| schemars | 1.2.3 | MIT | https://github.com/GREsau/schemars |
| schemars_derive | 1.2.3 | MIT | https://github.com/GREsau/schemars |
| scoped-tls | 1.0.1 | MIT/Apache-2.0 | https://github.com/alexcrichton/scoped-tls |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 | https://github.com/bluss/scopeguard |
| self_cell | 1.3.0 | Apache-2.0 OR GPL-2.0-only | https://github.com/Voultapher/self_cell |
| serde | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_derive_internals | 0.30.0 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | https://github.com/serde-rs/json |
| serde_repr | 0.1.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/serde-repr |
| serde_spanned | 1.1.3 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| sha2 | 0.11.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |
| signal-hook | 0.3.18 | Apache-2.0/MIT | https://github.com/vorner/signal-hook |
| signal-hook-mio | 0.2.5 | MIT OR Apache-2.0 | https://github.com/vorner/signal-hook |
| signal-hook-registry | 1.4.8 | MIT OR Apache-2.0 | https://github.com/vorner/signal-hook |
| simd-adler32 | 0.3.10 | MIT | https://github.com/mcountryman/simd-adler32 |
| simplecss | 0.2.2 | Apache-2.0 OR MIT | https://github.com/linebender/simplecss |
| siphasher | 1.0.4 | MIT OR Apache-2.0 | https://github.com/jedisct1/rust-siphash |
| skrifa | 0.40.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| skrifa | 0.44.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| slab | 0.4.12 | MIT | https://github.com/tokio-rs/slab |
| slotmap | 1.1.1 | Zlib | https://github.com/orlp/slotmap |
| smallvec | 1.16.5 | MIT OR Apache-2.0 | https://github.com/servo/rust-smallvec |
| smithay-client-toolkit | 0.19.2 | MIT | https://github.com/smithay/client-toolkit |
| smol_str | 0.2.2 | MIT OR Apache-2.0 | https://github.com/rust-analyzer/smol_str |
| smol_str | 0.3.7 | MIT OR Apache-2.0 | https://github.com/rust-lang/rust-analyzer/tree/master/lib/smol_str |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 | https://github.com/nvzqz/static-assertions-rs |
| strict-num | 0.1.1 | MIT | https://github.com/RazrFalcon/strict-num |
| strsim | 0.11.1 | MIT | https://github.com/rapidfuzz/strsim-rs |
| svgtypes | 0.16.2 | Apache-2.0 OR MIT | https://github.com/linebender/svgtypes |
| swash | 0.2.10 | Apache-2.0 OR MIT | https://github.com/dfrg/swash |
| syn | 2.0.119 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| syn | 3.0.10 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| sys-locale | 0.3.2 | MIT OR Apache-2.0 | https://github.com/1Password/sys-locale |
| tar | 0.4.46 | MIT OR Apache-2.0 | https://github.com/composefs/tar-rs |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | https://github.com/Stebalien/tempfile |
| thiserror | 1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| thiserror-impl | 1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| tiff | 0.11.3 | MIT | https://github.com/image-rs/image-tiff |
| tiny-skia | 0.12.0 | BSD-3-Clause | https://github.com/linebender/tiny-skia |
| tiny-skia-path | 0.12.0 | BSD-3-Clause | https://github.com/linebender/tiny-skia/tree/master/path |
| tinyvec | 1.13.3 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/tinyvec |
| toml | 1.2.1+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| toml_datetime | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| toml_edit | 0.25.19+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| toml_writer | 1.2.0+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| tracing | 0.1.44 | MIT | https://github.com/tokio-rs/tracing |
| tracing-attributes | 0.1.31 | MIT | https://github.com/tokio-rs/tracing |
| tracing-core | 0.1.36 | MIT | https://github.com/tokio-rs/tracing |
| tree_magic_mini | 3.2.2 | MIT | https://github.com/mbrubeck/tree_magic/ |
| ttf-parser | 0.25.1 | MIT OR Apache-2.0 | https://github.com/harfbuzz/ttf-parser |
| twox-hash | 2.1.5 | MIT | https://github.com/shepmaster/twox-hash |
| type-map | 0.5.1 | MIT/Apache-2.0 | https://github.com/kardeiz/type-map |
| typenum | 1.20.1 | MIT OR Apache-2.0 | https://github.com/paholg/typenum |
| unicode-bidi | 0.3.18 | MIT OR Apache-2.0 | https://github.com/servo/unicode-bidi |
| unicode-general-category | 1.1.0 | Apache-2.0 | https://github.com/yeslogic/unicode-general-category |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/dtolnay/unicode-ident |
| unicode-linebreak | 0.1.5 | Apache-2.0 | https://github.com/axelf4/unicode-linebreak |
| unicode-properties | 0.1.5 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/unicode-rs/unicode-properties |
| unicode-script | 0.5.9 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-script |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-segmentation |
| unicode-vo | 0.1.0 | MIT/Apache-2.0 | https://github.com/RazrFalcon/unicode-vo |
| unicode-width | 0.2.3 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-width |
| usvg | 0.49.0 | Apache-2.0 OR MIT | https://github.com/linebender/resvg |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT | https://github.com/alacritty/vte |
| uuid | 1.29.2 | Apache-2.0 OR MIT | https://github.com/uuid-rs/uuid |
| vello_common | 0.2.0 | Apache-2.0 OR MIT | https://github.com/linebender/vello |
| vello_cpu | 0.2.0 | Apache-2.0 OR MIT | https://github.com/linebender/vello |
| wayland-backend | 0.3.17 | MIT | https://github.com/smithay/wayland-rs |
| wayland-client | 0.31.15 | MIT | https://github.com/smithay/wayland-rs |
| wayland-csd-frame | 0.3.0 | MIT | https://github.com/rust-windowing/wayland-csd-frame |
| wayland-cursor | 0.31.14 | MIT | https://github.com/smithay/wayland-rs |
| wayland-protocols | 0.32.13 | MIT | https://github.com/smithay/wayland-rs |
| wayland-protocols-plasma | 0.3.12 | MIT | https://github.com/smithay/wayland-rs |
| wayland-protocols-wlr | 0.3.12 | MIT | https://github.com/smithay/wayland-rs |
| wayland-scanner | 0.31.11 | MIT | https://github.com/smithay/wayland-rs |
| wayland-sys | 0.31.11 | MIT | https://github.com/smithay/wayland-rs |
| web-time | 1.1.0 | MIT OR Apache-2.0 | https://github.com/daxpedda/web-time |
| weezl | 0.1.12 | MIT OR Apache-2.0 | https://github.com/image-rs/weezl |
| wgpu | 30.0.2 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| wgpu-core | 30.0.2 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| wgpu-core-deps-windows-linux-android | 30.0.2 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| wgpu-hal | 30.0.2 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| wgpu-naga-bridge | 30.0.2 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| wgpu-types | 30.0.2 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu |
| winit | 0.30.13 | Apache-2.0 | https://github.com/rust-windowing/winit |
| winnow | 1.0.5 | MIT | https://github.com/winnow-rs/winnow |
| wl-clipboard-rs | 0.9.5 | MIT OR Apache-2.0 | https://github.com/YaLTeR/wl-clipboard-rs |
| x11-dl | 2.21.0 | MIT | https://github.com/AltF02/x11-rs.git |
| x11rb | 0.13.2 | MIT OR Apache-2.0 | https://github.com/psychon/x11rb |
| x11rb-protocol | 0.13.2 | MIT OR Apache-2.0 | https://github.com/psychon/x11rb |
| xattr | 1.7.0 | MIT OR Apache-2.0 | https://github.com/Stebalien/xattr |
| xcursor | 0.3.11 | MIT | https://github.com/esposm03/xcursor-rs |
| xkbcommon-dl | 0.4.2 | MIT | https://github.com/rust-windowing/xkbcommon-dl |
| xkeysym | 0.2.1 | MIT OR Apache-2.0 OR Zlib | https://github.com/notgull/xkeysym |
| xmlwriter | 0.1.0 | MIT | https://github.com/RazrFalcon/xmlwriter |
| yazi | 0.2.2 | Apache-2.0 OR MIT | https://github.com/dfrg/yazi |
| zbus | 5.19.0 | MIT | https://github.com/z-galaxy/zbus/ |
| zbus-lockstep | 0.5.2 | MIT | https://github.com/luukvanderduim/zbus-lockstep |
| zbus-lockstep-macros | 0.5.2 | MIT | https://github.com/luukvanderduim/zbus-lockstep |
| zbus_macros | 5.19.0 | MIT | https://github.com/z-galaxy/zbus/ |
| zbus_names | 4.3.5 | MIT | https://github.com/z-galaxy/zbus/ |
| zbus_xml | 5.2.1 | MIT | https://github.com/z-galaxy/zbus/ |
| zcheapstr | 1.2.0 | MIT | https://github.com/z-galaxy/zcheapstr/ |
| zeno | 0.3.3 | Apache-2.0 OR MIT | https://github.com/dfrg/zeno |
| zerocopy | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | https://github.com/google/zerocopy |
| zerocopy-derive | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | https://github.com/google/zerocopy |
| zlib-rs | 0.6.8 | Zlib | https://github.com/trifectatechfoundation/zlib-rs |
| zmij | 1.0.23 | MIT | https://github.com/dtolnay/zmij |
| zune-core | 0.5.3 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image |
| zune-jpeg | 0.5.15 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg |
| zvariant | 5.15.1 | MIT | https://github.com/z-galaxy/zbus/ |
| zvariant_derive | 5.15.1 | MIT | https://github.com/z-galaxy/zbus/ |
| zvariant_utils | 4.2.0 | MIT | https://github.com/z-galaxy/zbus/ |
