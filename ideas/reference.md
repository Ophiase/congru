# What would make it the reference

**[sketch]** The claim worth owning: **a pose it can defend**. Fast ICP exists
already, in Open3D, PCL and small_gicp. Nothing here beats them by being one more
ICP. What is rare is saying how far an answer can be from the best one.

Ranked by how much each one changes what a stranger may believe. Each has what
would close it.

| Rank | Direction | What it gives | Closed if |
| --- | --- | --- | --- |
| 1 | [Graded against a proof](method/certificate.md) | Methods scored on a proved optimum, not on themselves | The bound stays open on every useful size |
| 2 | Confidently wrong as the headline | The rate at which a believed pose is false | No answer can be refused without losing most frames |
| 3 | Real recordings with a truth | The first number that is not simulated | Results collapse: then that is the finding |
| 4 | Compared on the same frames | Where Open3D and small_gicp win | Never: losing is a result too |
| 5 | [Scale, then slight deformation](method/not-quite-the-model.md) | How far a rigid answer holds, a breaking curve | A rigid fit holds past any real error |
| 6 | Where to put the marks | A placement chosen for what it pins down | It does not beat hand-placed marks |
| 7 | More realistic scenes | Better pictures | **[caution]** They prove nothing a real recording would not |

## Why this order

- **A truth is not an optimum**: a synthetic truth says if the pose is right; a
  proof says whether the search missed something better.
- **A wrong answer costs more than no answer**: the wider-box runs show a pose held
  wrong for a whole film ([the box](method/box-as-test.md)).
- **Simulated scenes can flatter**: they were tuned on, so they cannot testify.

## How each is presented

- **What it does, what it does not**, side by side, at the top.
- **Each claim with its trust**: proved, tested or inferred.
- **Attacks published with the result**: what tried to break it, and how it did.
- **Where it loses** is said as plainly as where it wins.

## Questions for the author

- **[question]** [The usual real sets](https://bop.felk.cvut.cz/datasets/) come as
  archives, and the download rule allows single files only. **[to check]**
  Licences and sizes.
- **[question]** Is a similarity (scale) still the first extension, as
  [the vision](vision.md) says? **[guess]** Yes: a wrong unit or depth scale is
  common and it is a closed form.
