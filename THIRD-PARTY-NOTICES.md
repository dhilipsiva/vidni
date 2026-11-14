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

The `vidni` binaries are built from these 155 crates, each under the licence its authors chose (16 licence expressions in all). Each crate's own licence files are in its source, at the address given.

| Crate | Version | Licence | Source |
|---|---|---|---|
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | https://github.com/oyvindln/adler2 |
| aho-corasick | 1.1.5 | Unlicense OR MIT | https://github.com/BurntSushi/aho-corasick |
| alsa | 0.11.0 | Apache-2.0/MIT | https://github.com/diwic/alsa-rs |
| alsa-sys | 0.4.0 | MIT | https://github.com/diwic/alsa-sys |
| anstream | 1.0.0 | MIT OR Apache-2.0 | https://github.com/rust-cli/anstyle.git |
| anstyle | 1.0.14 | MIT OR Apache-2.0 | https://github.com/rust-cli/anstyle.git |
| anstyle-parse | 1.0.0 | MIT OR Apache-2.0 | https://github.com/rust-cli/anstyle.git |
| anstyle-query | 1.1.5 | MIT OR Apache-2.0 | https://github.com/rust-cli/anstyle.git |
| arrayref | 0.3.9 | BSD-2-Clause | https://github.com/droundy/arrayref |
| arrayvec | 0.7.8 | MIT OR Apache-2.0 | https://github.com/bluss/arrayvec |
| base64 | 0.23.1 | MIT OR Apache-2.0 | https://github.com/marshallpierce/rust-base64 |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | https://github.com/bitflags/bitflags |
| block-buffer | 0.12.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| bytemuck_derive | 1.12.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/cfg-if |
| clap | 4.6.7 | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| clap_builder | 4.6.7 | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| clap_derive | 4.6.7 | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| clap_lex | 1.1.1 | MIT OR Apache-2.0 | https://github.com/clap-rs/clap |
| colorchoice | 1.0.5 | MIT OR Apache-2.0 | https://github.com/rust-cli/anstyle.git |
| command-fds | 0.3.3 | Apache-2.0 | https://github.com/google/command-fds/ |
| const-oid | 0.10.3 | Apache-2.0 OR MIT | https://github.com/RustCrypto/formats |
| convert_case | 0.10.0 | MIT | https://github.com/rutrum/convert-case |
| core_maths | 0.1.1 | MIT | https://github.com/robertbastian/core_maths |
| cosmic-text | 0.19.0 | MIT OR Apache-2.0 | https://github.com/pop-os/cosmic-text |
| cpal | 0.17.3 | Apache-2.0 | https://github.com/RustAudio/cpal |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| crc32fast | 1.5.5 | MIT OR Apache-2.0 | https://github.com/srijs/rust-crc32fast |
| crossterm | 0.29.0 | MIT | https://github.com/crossterm-rs/crossterm |
| crypto-common | 0.2.2 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| ctrlc | 3.5.2 | MIT/Apache-2.0 | https://github.com/Detegr/rust-ctrlc.git |
| dasp_sample | 0.11.0 | MIT OR Apache-2.0 | https://github.com/rustaudio/sample.git |
| data-url | 0.3.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url |
| derive_more | 2.1.1 | MIT | https://github.com/JelteF/derive_more |
| derive_more-impl | 2.1.1 | MIT | https://github.com/JelteF/derive_more |
| digest | 0.11.3 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| document-features | 0.2.12 | MIT OR Apache-2.0 | https://github.com/slint-ui/document-features |
| dyn-clone | 1.0.20 | MIT OR Apache-2.0 | https://github.com/dtolnay/dyn-clone |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/equivalent |
| errno | 0.3.15 | MIT OR Apache-2.0 | https://github.com/lambda-fairy/rust-errno |
| euclid | 0.22.14 | MIT OR Apache-2.0 | https://github.com/servo/euclid |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/fastrand |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | https://github.com/image-rs/fdeflate |
| flate2 | 1.1.11 | MIT OR Apache-2.0 | https://github.com/rust-lang/flate2-rs |
| float-cmp | 0.9.0 | MIT | https://github.com/mikedilger/float-cmp |
| font-types | 0.11.3 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| font-types | 0.12.7 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| fontdb | 0.23.0 | MIT | https://github.com/RazrFalcon/fontdb |
| fontdb | 0.24.0 | MIT | https://github.com/RazrFalcon/fontdb |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| harfrust | 0.12.0 | MIT | https://github.com/harfbuzz/harfrust |
| harfrust | 0.5.2 | MIT | https://github.com/harfbuzz/harfrust |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| heck | 0.5.0 | MIT OR Apache-2.0 | https://github.com/withoutboats/heck |
| hound | 3.5.1 | Apache-2.0 | https://github.com/ruuda/hound |
| hybrid-array | 0.4.15 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hybrid-array |
| imagesize | 0.15.0 | MIT | https://github.com/Roughsketch/imagesize |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/indexmap |
| is_terminal_polyfill | 1.70.2 | MIT OR Apache-2.0 | https://github.com/polyfill-rs/is_terminal_polyfill |
| itoa | 1.0.19 | MIT OR Apache-2.0 | https://github.com/dtolnay/itoa |
| kurbo | 0.13.1 | Apache-2.0 OR MIT | https://github.com/linebender/kurbo |
| libc | 0.2.191 | MIT OR Apache-2.0 | https://github.com/rust-lang/libc |
| libm | 0.2.16 | MIT | https://github.com/rust-lang/compiler-builtins |
| linebender_resource_handle | 0.1.1 | Apache-2.0 OR MIT | https://github.com/linebender/raw_resource_handle |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/sunfishcode/linux-raw-sys |
| litrs | 1.0.1 | MIT OR Apache-2.0 | https://github.com/LukasKalbertodt/litrs |
| lock_api | 0.4.14 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| log | 0.4.35 | MIT OR Apache-2.0 | https://github.com/rust-lang/log |
| memchr | 2.8.3 | Unlicense OR MIT | https://github.com/BurntSushi/memchr |
| memmap2 | 0.9.12 | MIT OR Apache-2.0 | https://github.com/RazrFalcon/memmap2-rs |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| mio | 1.2.5 | MIT | https://github.com/tokio-rs/mio |
| nix | 0.31.3 | MIT | https://github.com/nix-rust/nix |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | https://github.com/rust-num/num-traits |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | https://github.com/matklad/once_cell |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| pico-args | 0.5.0 | MIT | https://github.com/RazrFalcon/pico-args |
| png | 0.18.1 | MIT OR Apache-2.0 | https://github.com/image-rs/image-png |
| polycool | 0.4.0 | MIT OR Apache-2.0 | https://github.com/linebender/kurbo |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | https://github.com/dtolnay/proc-macro2 |
| quote | 1.0.47 | MIT OR Apache-2.0 | https://github.com/dtolnay/quote |
| rand_core | 0.10.1 | MIT OR Apache-2.0 | https://github.com/rust-random/rand_core |
| rangemap | 1.8.0 | MIT/Apache-2.0 | https://github.com/jeffparsons/rangemap |
| read-fonts | 0.37.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| read-fonts | 0.41.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| ref-cast | 1.0.27 | MIT OR Apache-2.0 | https://github.com/dtolnay/ref-cast |
| ref-cast-impl | 1.0.27 | MIT OR Apache-2.0 | https://github.com/dtolnay/ref-cast |
| regex | 1.13.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| regress | 0.12.0 | MIT OR Apache-2.0 | https://github.com/ridiculousfish/regress |
| resvg | 0.49.0 | Apache-2.0 OR MIT | https://github.com/linebender/resvg |
| rgb | 0.8.53 | MIT | https://github.com/kornelski/rust-rgb |
| roxmltree | 0.21.2 | MIT OR Apache-2.0 | https://github.com/RazrFalcon/roxmltree |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT | https://github.com/rust-lang/rustc-hash |
| rustix | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/rustix |
| schemars | 1.2.3 | MIT | https://github.com/GREsau/schemars |
| schemars_derive | 1.2.3 | MIT | https://github.com/GREsau/schemars |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 | https://github.com/bluss/scopeguard |
| self_cell | 1.3.0 | Apache-2.0 OR GPL-2.0-only | https://github.com/Voultapher/self_cell |
| serde | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_derive_internals | 0.30.0 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | https://github.com/serde-rs/json |
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
| slotmap | 1.1.1 | Zlib | https://github.com/orlp/slotmap |
| smallvec | 1.16.5 | MIT OR Apache-2.0 | https://github.com/servo/rust-smallvec |
| smol_str | 0.3.7 | MIT OR Apache-2.0 | https://github.com/rust-lang/rust-analyzer/tree/master/lib/smol_str |
| strict-num | 0.1.1 | MIT | https://github.com/RazrFalcon/strict-num |
| strsim | 0.11.1 | MIT | https://github.com/rapidfuzz/strsim-rs |
| svgtypes | 0.16.2 | Apache-2.0 OR MIT | https://github.com/linebender/svgtypes |
| swash | 0.2.10 | Apache-2.0 OR MIT | https://github.com/dfrg/swash |
| syn | 2.0.119 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| syn | 3.0.10 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| sys-locale | 0.3.2 | MIT OR Apache-2.0 | https://github.com/1Password/sys-locale |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | https://github.com/Stebalien/tempfile |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| tiny-skia | 0.12.0 | BSD-3-Clause | https://github.com/linebender/tiny-skia |
| tiny-skia-path | 0.12.0 | BSD-3-Clause | https://github.com/linebender/tiny-skia/tree/master/path |
| tinyvec | 1.13.3 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/tinyvec |
| toml | 1.2.1+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| toml_datetime | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| toml_edit | 0.25.19+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| toml_writer | 1.2.0+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| ttf-parser | 0.25.1 | MIT OR Apache-2.0 | https://github.com/harfbuzz/ttf-parser |
| typenum | 1.20.1 | MIT OR Apache-2.0 | https://github.com/paholg/typenum |
| unicode-bidi | 0.3.18 | MIT OR Apache-2.0 | https://github.com/servo/unicode-bidi |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/dtolnay/unicode-ident |
| unicode-linebreak | 0.1.5 | Apache-2.0 | https://github.com/axelf4/unicode-linebreak |
| unicode-script | 0.5.9 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-script |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-segmentation |
| unicode-vo | 0.1.0 | MIT/Apache-2.0 | https://github.com/RazrFalcon/unicode-vo |
| usvg | 0.49.0 | Apache-2.0 OR MIT | https://github.com/linebender/resvg |
| utf8parse | 0.2.2 | Apache-2.0 OR MIT | https://github.com/alacritty/vte |
| winnow | 1.0.5 | MIT | https://github.com/winnow-rs/winnow |
| xmlwriter | 0.1.0 | MIT | https://github.com/RazrFalcon/xmlwriter |
| yazi | 0.2.2 | Apache-2.0 OR MIT | https://github.com/dfrg/yazi |
| zeno | 0.3.3 | Apache-2.0 OR MIT | https://github.com/dfrg/zeno |
| zlib-rs | 0.6.8 | Zlib | https://github.com/trifectatechfoundation/zlib-rs |
| zmij | 1.0.23 | MIT | https://github.com/dtolnay/zmij |
