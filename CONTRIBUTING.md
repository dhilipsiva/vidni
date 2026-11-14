# Contributing to vidni

Thank you for wanting to help. This page says how to set up, what a change needs before it is merged, and what is worth asking about first.

## Setting up

vidni builds inside the shell its flake provides, which has the Rust toolchain and every tool vidni drives at the pinned versions. You need Linux on x86_64 and [Nix](https://nixos.org/download/) with flakes on ([the quickstart](docs/quickstart.md#what-you-need) says how).

```sh
git clone https://github.com/dhilipsiva/vidni
cd vidni
nix develop
just lint     # formatting and clippy, with warnings as errors
just test     # the unit and integration tests that need no browser or voice
```

The `Justfile` has every recurring command, each with a comment. The ones a change usually needs:

| Command | What it checks |
|---|---|
| `just lint` and `just test` | Always |
| `just test-build` | A project from draft to approved, with the real tools. Run it when you touch a build, a gate or a command. |
| `just test-app` | The desktop app's tests that need the tools |
| `just test-capture`, `just test-record`, `just test-hear` | Captures, the recorder, the listening check |
| `just schema`, `just keys`, `just notices` | Regenerate the schema, the key reference and the notices after changing the project model, a pin or a dependency. CI fails when one is out of date. |
| `nix flake check` | The package builds and its tests pass in Nix's sandbox |

No test may record from a microphone, play through a speaker, or open a window or a file manager on the machine it runs on. The recorder's tests use a made-up input and ALSA's null device.

## What a change needs

- **Tests that fail without it.** A test that can't fail proves nothing; say in the test what it shows.
- **The same result every time.** vidni's point is that a video builds to the same bytes. Nothing in a build may depend on the clock, the machine's fonts, the order of a hash map, or the network. `vidni repro` and `docs/determinism.md` say how that is checked.
- **Documentation in the same change.** `docs/spec.md` describes every key and command; `CHANGELOG.md` gets a line under Unreleased when a user would notice the change.
- **Plain English,** in code comments, messages and docs: short sentences, and the word for the thing. Error messages say what is wrong and what to do about it.
- **A commit message that says why.** The first line names what changed; the body gives the reason.

## Ask first

Open an issue before you write the code for any of these, since they change a promise vidni makes ([docs/releasing.md](docs/releasing.md#versions)):

- a change to `vidni.toml`'s schema, or to what an existing key means;
- a change to a command's name, flags or exit codes;
- moving a pin: a tool's version, a font, a voice;
- a new dependency, above all one that isn't pure Rust;
- anything that would make a build need an LLM or the network. vidni works without either, and stays that way.

`DECISIONS.md` records why vidni is the way it is, and `TODO.md` what is planned. A change that goes against a decision needs a reason that the decision didn't weigh.

## Licence

vidni is licensed under MIT OR Apache-2.0. By contributing, you agree that your contribution is licensed the same way, with no other terms.

Files you add that come from somewhere else (a font, a picture, a recording, a piece of code) need a licence that allows it, and a line in the notices saying what it is.

## Conduct, and reporting a security problem

Everyone taking part is expected to follow the [code of conduct](CODE_OF_CONDUCT.md).

For a security problem, don't open a public issue. Report it privately through [GitHub's security advisories](https://github.com/dhilipsiva/vidni/security/advisories/new).
