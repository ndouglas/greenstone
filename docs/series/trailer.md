# The series trailer

The pitch for the whole series: what neural networks can do, can't do,
might do and shouldn't do; where the series is going; and why it moves
slowly. It began as the opening of 001's introduction, and moved here
(Nate, 2026-10-31) because it wants pictures from videos not yet made,
and 001 should be finishable with the visual language it has.

## Rules

- **Made last, and remade as the series grows.** It shows off the
  series' best pictures, which won't exist until late.
- **Only what's real.** Every result or failure it shows is one the
  series has actually produced, or a cited result redrawn with its
  source. A placeholder never ships. No failure is borrowed because it
  fits the line: if we haven't produced an annoying behaviour of our
  own, we build things until we do (Nate).
- Until it exists, 001 makes no promise about it: no link to a trailer
  that isn't there.

## The lines

Moved from 001 as they stood. Nate's takes for all ten are kept in
`tools/series/videos/trailer/takes/` (gitignored, like every take), in
the recorder's layout; the text may change before it's built.

1. **series:** "In this series, we'll explore neural networks, and learn
   how they work from the ground up."
   Board: a real network we trained on the games, thousands of neurons in
   a dense web, its actual activations rippling warm and cool through it
   as Mario plays beside it.
2. **series-2:** "As we learn what they can do, we'll also learn what
   they can't do…"
   Board: Super Mario Bros. mirrored, the flag now to the left; a network
   that learned to play keeps running right, into the wall at the start.
3. **series-3:** "…what they might do…"
   Board: superposition, five features squeezed as five arrows into a
   plane, two neurons holding them all. Cites Elhage et al., 2022
   (below).
4. **series-4:** "…and what they almost certainly shouldn't do."
   Board: to be decided (see the candidates below).
5. **promise:** "Here's where we're going. We'll start with a single
   artificial neuron and build up: neurons that learn, layers that bend
   straight lines, networks that look at the game screen itself."
   Board: a quick montage of the series' pictures, one per step.
6. **promise-2:** "Along the way, we'll build some broken on purpose,
   with a flaw we planted, to check whether our tools can find it."
   Board: a network with one dot secretly marked; a magnifying glass
   sweeps over it and finds the mark.
7. **promise-3:** "And we'll take on some of the field's open questions,
   questions nobody thought to ask, and questions only a mad scientist
   would ask."
   Board: three cards: one neuron lit by three different game moments; a
   strange byte glowing in the memory grid; a network shrunk and
   squeezed under a bell jar.
8. **promise-4:** "Whatever we find, we'll test, and show you how."
   Board: a result beside its baseline bar and its held-out test, the
   checks lighting one by one.
9. **slow:** "You may wonder at times why we're moving so slowly."
   Board: one dot, alone, the rest of the network faint and far behind
   it.
10. **slow-2:** "But I believe, if you stay with me, we'll see some
    pretty amazing things."
    Board: the faint network behind the dot begins to light, neuron by
    neuron.

## What "shouldn't do" could show

- **Dropped: Murphy's Tetris pause** (Murphy, 2013: playfun pauses
  Tetris just before the piece that would end it, and leaves it paused).
  It's a search, not a neural network, and not ours; it stays out unless
  the series produces the same failure itself.
- **Candidate: the Punch-Out!! brains that stand still.** Some of our
  own trained brains stand still, and Don Flamenco never attacks a Mac
  who stands still (`docs/ideas/brain-cartridge.md`). Before it's used,
  find out exactly why each one stood still, and whether that was the
  letter of its goal.

## Sources held for it

```toml
[elhage2022]
title = "Toy Models of Superposition (Transformer Circuits Thread): five features stored in two dimensions as a pentagon"
authors = "N. Elhage, T. Hume, C. Olsson, et al."
year = 2022
url = "https://transformer-circuits.pub/2022/toy_model/index.html"

[murphy2013]
title = "The First Level of Super Mario Bros. is Easy with Lexicographic Orderings and Time Travel . . . after that it gets a little tricky (SIGBOVIK 2013): in Tetris, playfun pauses the game right before the next piece would end it, and leaves it paused"
authors = "T. Murphy VII"
year = 2013
url = "http://tom7.org/mario/mario.pdf"
```
