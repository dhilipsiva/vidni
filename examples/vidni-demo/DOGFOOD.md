# What making this video found

vidni was used here as anyone would use it: `vidni new`, edit the file, `vidni draft`, look, repeat. This is what got in the way on 2026-11-15, and what became of each.

## Fixed in vidni

1. **`vidni draft` panicked.** A line on one picture, then a line on a smaller one: the first faded out under the second's framing, which reads past its edge. Every earlier example had one page, or cards between its pictures. Each picture now fades under its own framing, and a test builds such a cut.
2. **A picture that wasn't the cut's shape was cut off.** The Mermaid diagram showed its middle third, the recording sheet lost its left edge, and a portrait picture showed its top. A picture, a session and code are now shown whole, with their own ground around them, clear of the captions and the section's label (DECISIONS.md, T40).
3. **The last rows of a session sat under the captions,** and the first under the section's label. The same change keeps them clear.
4. **An overlay in a 9:16 cut for platforms** was wider than the part the apps leave free, and lay over the view. QA said so, which is how it was found. It is as wide as the free part now.
5. **A terminal can't be read in a 9:16 cut:** a band of lines is as wide as the window, so no camera could frame less than all 100 columns. A target may now name `columns`.
6. **`vidni repro` on a project with no takes** only said the takes were missing. It now adds that `--draft` checks the drafts.
7. **`vidni new examples/vidni-demo`** only said that isn't a name. It now says to use `--parent`.

## Open

1. **A section's label sits on a terminal's title bar** when a camera frames the window's first lines in a 16:9 cut. The label is drawn over the picture, as on a web page. A label of `""` with a `chapter` avoids it, at the cost of the label.
2. **A scene can't differ by cut.** The short shares lines with the walkthrough, so it shares their scenes: a session 100 columns wide, where a short wants one of 50. `camera_vertical` and `columns` make it readable, but the short's own session would be better.
3. **An overlay is the same for every cut.** Two chips fit a 9:16 cut in one row; a third made a second row over half the view, so the walkthrough lost its third chip too.
4. **The ground around a terminal is the colour of its bar,** its first pixel, not of its text's ground. It looks deliberate, and wasn't.
5. **A changed target takes every capture of its scene again,** though the picture is the same: the scene's targets are part of a capture's key. It costs seconds here, and would cost more on a web scene.
6. **The review frames are one per section.** To look at every line I cut frames out of the draft with ffmpeg. A sheet of one frame per line would be the thing to look at.
7. **`just app-shot` includes the compositor's bar,** with a clock. The picture here was cut by hand.
8. **The walkthrough runs 2:30,** short of the three minutes planned. `vidni plan` gives the length without the pauses, so the whole length is known only after a draft.

## Not found wanting

The file was written once and validated first time. The cards, the chapters, `[publish]`, `only` lines, the code scene at a commit and the Mermaid diagram worked as the spec says. A second draft took under a second, and `vidni repro --draft` agreed with the build.
