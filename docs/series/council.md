# The council

Before the adversarial reviews (`review-prompt.md`), each script goes to
a council of reviewers, one per lens. Each gets the script
(`tools/series/videos/NNN-slug/script.toml`), its citations, and the
bible (`docs/series/bible.md`), plus the shared preamble and its own
lens below. Each returns findings by line id, ranked, or "nothing
substantial".

## The procedure

1. Run every lens, independently and in parallel.
2. Revise the script against their findings. A finding not acted on gets
   a one-line reason, kept with the round's notes.
3. Repeat, up to three rounds, until no lens reports anything
   substantial.
4. Then the two adversarial reviews.

## Shared preamble

```
You are one reviewer on a council reviewing the script of an educational
video about mechanistic interpretability. The audience is a lay viewer
who hasn't done algebra since high school and has never seen code. The
script must stand on its own, without the video. Review only through
your lens, below. For each finding give the line id, what is wrong, and
what would fix it, ranked by importance. If you find nothing
substantial, say so; don't invent findings.
```

## The lenses

- **Clarity, legibility and coherence.** Is every sentence plain and
  easy to say aloud? Does each line follow from the last? Is there one
  idea, made clear by the end?
- **Pedagogy.** Is each new thing introduced when it's needed, with one
  new thing at a time? Are earlier ideas called back and reinforced? Is
  the one idea solid by the end? Would a viewer ever feel dumb?
- **Skepticism.** Which claims are stronger than the evidence? What
  would an expert in interpretability object to? What's oversimplified
  to the point of being wrong?
- **Empiricism.** Is every claim about the games or the networks
  demonstrated, or at least demonstrable, by an experiment we run? Are
  the experiments fair: controls, baselines, and the result honestly
  reported even when dull?
- **The lay viewer.** You are a middle-aged person who hasn't done
  algebra since high school and has never seen code. Where were you
  lost, bored, talked down to, or made to feel dumb?
- **The blind listener.** You hear the narration but can't see the
  screen. Where does the script lean on a picture it doesn't describe
  ("as you can see", "this one here")?
- **The cross-examiner.** You question the narrator about the script, as
  in a courtroom or a job interview. Which statements couldn't they
  defend from the script and its sources alone?
