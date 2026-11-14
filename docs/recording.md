# Recording guide

How to record a video's narration with `vidni record`, and get takes you won't have to redo. [The spec](spec.md#recording-vidni-record) has every key and flag; this page is the order to do things in, and why.

## Before you record

1. **Finish the script first.** Draft it, watch it, change it, and ratify it (`vidni ratify`). A take belongs to a line's words: change a word and the take goes stale, and you read the line again. Punctuation and capitals don't count.
2. **Print the sheet.** `vidni plan` lists every line with how long the draft voice took over it. Read it aloud once. A line you trip over is a line to rewrite now, not after recording.
3. **Check the room and the microphone:**

   ```sh
   vidni record --test
   ```

   It records the room for three seconds with nobody speaking, then one line from you, and measures both. It records no take.
   - **A room louder than −55 dBFS is called noisy.** Turn off the fan, close the window, move the laptop away from the microphone, and check again.
   - **Your voice should peak well below 0 dBFS.** The review warns when a take is too hot or too quiet. Set the level with the microphone's own gain, and then leave it alone for the whole session.
   - **The room tone is kept** as `takes/_room/tone.wav`, in place of the one before.

## The session

```sh
vidni record
```

It shows one line at a time, with the words to say, and takes single keys with no Enter.

| When | Keys |
|---|---|
| Ready | `space` record · `p` play the saved take · `s` play the section's takes · `←` `→` move between lines · `q` quit |
| Recording | `space` or `enter` stop · `esc` cancel |
| Review | `enter` keep and go on · `p` play · `r` record again · `esc` discard · `q` keep and quit |

- **Press space, wait a beat, speak, wait a beat, press space.** vidni cuts the keypresses off both ends and trims the silence, keeping 0.15 s before your words and 0.25 s after them. You don't need to time the keys.
- **Listen to each take before you keep it** (`p`). The numbers say whether the level is right. Only you can say whether it sounds right.
- **Nothing replaces a line's take until you keep the new one.** Discard or cancel, and the line is as it was. Every take you stopped stays in `raw/`, so nothing you said is lost.
- **You can stop at any time** (`q`). The next session starts at the first line that still has no take of its present words.
- **A line that several cuts use is recorded once.** A short's own lines are recorded with the rest; `--cut short` goes through that cut's lines only.

## Keeping the takes consistent

A video recorded over several sessions sounds like one only if the sessions sound alike.
- **Same microphone, same distance, same gain.** Mark where you sit.
- **Same room, same time of day** if the room has traffic or a neighbour.
- **Run `vidni record --test` at the start of every session,** and compare its numbers with last time's.
- **Listen across the joins.** `s` plays a section's kept takes one after another. A line that stands out gets read again: `vidni record --only <line-id>`.

## After the session

```sh
vidni takes            # every line: current, stale or missing
vidni hear             # listens to each take, and lists the ones worth a second listen
vidni final            # builds the cuts from your takes
```

- **`vidni hear` only advises.** It flags a take that whisper heard differently from the script: often a dropped word, sometimes just a name it can't spell. It changes nothing, and a low score fails nothing. It knows English only.
- **Back the takes up.** They are the one thing vidni can't make again:

  ```sh
  vidni backup --to /mnt/disk/vidni --to /mnt/elsewhere/vidni
  ```

  It wants two places, and one of them should be off the machine. `vidni verify` checks them, and `vidni restore --from` brings the takes back.

## Changing a line later

Edit the line's words, and `vidni status` says the cut has changed. Then:
1. `vidni draft` to hear the new wording in the draft voice.
2. `vidni ratify` to freeze it again.
3. `vidni record` goes straight to the changed line: its old take is stale, and it is the first line without a take of its present words.
4. `vidni final`, then `vidni approve` for a new release.

Every other line keeps its take.

## Where things are

```text
takes/<line-id>/<hash>.wav          the kept take for that wording
takes/<line-id>/raw/<hash>-<n>.wav  every raw take for it, numbered from 1
takes/_room/tone.wav                the room, from the last mic check
```

The takes stay out of git. The microphone is the system's default input, or the one whose name holds `$VIDNI_MIC`; takes play through the default output, or the one whose name holds `$VIDNI_SPEAKER`.
