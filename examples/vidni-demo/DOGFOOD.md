# What making this video found

vidni was used here as anyone would use it: `vidni new`, edit the file, `vidni draft`, look, repeat. This is what got in the way on 2026-11-15, and what became of each.

## Fixed in vidni

1. **`vidni draft` panicked.** A line on one picture, then a line on a smaller one: the first faded out under the second's framing, which reads past its edge. Every earlier example had one page, or cards between its pictures. Each picture now fades under its own framing, and a test builds such a cut.
2. **A picture that wasn't the cut's shape was cut off.** The Mermaid diagram showed its middle third, the recording sheet lost its left edge, and a portrait picture showed its top. A picture, a session and code are now shown whole, with their own ground around them, clear of the captions and the section's label (DECISIONS.md, T40).
3. **The last rows of a session sat under the captions,** and the first under the section's label. The same change keeps them clear.
4. **An overlay in a 9:16 cut for platforms** was wider than the part the apps leave free, and lay over the view. QA said so, which is how it was found. It is as wide as the free part now.
5. **A terminal can't be read in a 9:16 cut:** a band of lines is as wide as the window, so no camera could frame less than all 100 columns. A target may now name `columns`.
6. **`vidni repro` on a project with no takes** only said the takes were missing. It now adds that `--draft` checks the drafts.
7. **`vidni new examples/vidni-demo`** only said that isn't a name. It now says to use `--in`.

## Fixed next, when the owner asked for the open ones

8. **A section's label sat on a terminal's title bar** when a camera framed the window's first lines. Such a framing now starts at the window's top, below the label. Rows in the middle of a long session can still pass under the label, as a web page's do.
9. **A scene couldn't differ by cut,** so the short had the walkthrough's sessions, 100 columns wide. A cut can now `swap` a scene: the short shows each session in a window 46 columns wide, where long lines go on in the next row.
10. **An overlay was the same for every cut.** The same `swap` gives the short two chips and the walkthrough its third.
11. **The ground around a terminal was the colour of its bar.** It is the window gone on now: the bar across the top, the text's ground at the sides.
12. **A changed target took every capture of its scene again.** A target of a picture, a terminal or code only frames it, and is read when the cut is built. A web scene's targets are found on its page, so there it still takes the scene again.
13. **The review frames were one per section.** Each build now writes sheets of every line, `review/<cut>/…/lines-01.png`. They are what this second pass was looked over with.
14. **`just app-shot` included the compositor's bar.** It takes the window alone, and `app.png` here is as it came.
15. **The whole length was known only after a draft.** `vidni plan` now prints it for each cut, pauses and all: `long about 2:30, short about 0:52`, which is what the drafts are.

## Still as it was

- **The walkthrough runs 2:30,** not the three minutes planned. Nothing is wrong with vidni there: the script is that long. A section on themes or music would fill it.
- **The code scene in the short** is still framed by `columns`, cut off at the right. A narrower code scene swapped in would carry its long lines on in the next row, as a terminal does; I left this one as it was.

## Not found wanting

The file was written once and validated first time. The cards, the chapters, `[publish]`, `only` lines, the code scene at a commit and the Mermaid diagram worked as the spec says. A second draft took under a second, and `vidni repro --draft` agreed with the build.
