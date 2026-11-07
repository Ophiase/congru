# Next session

**[defined]** The author's orders after 0.1.27, to read first, in autonomy.
Done means it runs and is measured, not that it is written here.

## In order

1. **Real public sets.** The agent may fetch them: the author lifts the archive
   limit of the download rule for data. Licence and sums recorded, nothing
   executed. **[to check]** Which sets, their licences and sizes.
2. **Detection is not this project.** Use existing models or the detections
   published with a set for the boxes, and say which. No more boxes from the
   truth on real data.
3. **Speed is the big focus.** Think for several kinds of hardware, possibly with
   different algorithms on each, and ready to use:
   - one core, many cores, GPU, chosen at run time when available;
   - threads in tracking and vision, which still run on one;
   - a GPU for what really costs, nearest neighbours above all, and not only
     rendering, the one thing measured so far;
   - the CPU alone keeps working everywhere.
4. **`f32`, `no_std`, a C ABI that covers the library.** To implement, not to
   discuss. The ABI is not frozen and need not be: see below.
5. **The failing ones.** A part of the demos with films of the scenes that fail
   today, removed as each one is fixed.
6. **Weak search and tracker.** Multistart on marks, 7 of 32 near the truth; the
   tracker, 33.7 % over every model.
7. **The proof, reviewed in autonomy.** There is no outside reviewer. Write the
   radius lemma out in full, read the sources again, check it in exact or
   interval arithmetic, keep the attacks.
8. **More realistic, more complex scenes.** Not hard, and it makes the algorithm
   more credible. After the above, but not dropped.

## The repository

- **`docs/hot/`, with a README**: what is under way, experimental, not part of
  the documentation, kept so that agents on other computers can take over. The
  night reports go there, and the `*-step.md` journals with their figures.
- **[to check]** The history weighs about 280 MB: clean it before going public?

## Not to bring up again

- **A frozen C ABI**: with no users and a private repository it means nothing;
  the question comes past 100 stars. What matters is to have one.
- **Having no users, no citations**: not constructive.
