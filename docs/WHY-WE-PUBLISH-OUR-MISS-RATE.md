# Why we publish our miss rate

INVAR catches cross-repository API breaks. On held-out real breaks it
currently catches 38 of 70. We print that on every verdict, in every
release note, and in the first screen of the README. Here is why a tool
would lead with the number most tools would bury.

**A skip is never a pass.** The worst output a checker can produce is
silence that reads as safety. Every mainstream scanner has a severity
filter, and severity filters are where true findings go to die: the
finding fired, someone tuned a threshold, and the dashboard went green.
INVAR has no severity levels. Anything it could not evaluate is stated,
with a stable reason code, and the exit code says so. Exit 4 means "I
did not look at everything", and it fails your CI by default, because
unexamined and clean are different facts and your pipeline deserves to
know which one it got.

**The benchmark was sealed before you could see it.** The 70-case
held-out set was fixed, hashed (BLAKE3 in the same commit), and
committed while this repository was still private. Nobody can quietly
swap cases in or out after the fact, including us. Each detection rule
was frozen and hashed before the batch that scored it was even
sourced, so no rule has ever been graded on data it was tuned against.

**You can reproduce the number without trusting us.** One command runs
a standalone, dependency-free scorer against the committed corpus
export and the committed engine-of-record results, and prints 38 of
70. Our own CI runs that same check on every push and inside the build
that ships every release binary. If the number ever stops reproducing,
our own tests fail before you ever see a release.

**Misses are recorded, not reframed.** When a frozen rule misses a
break, the miss goes in the record with the case pinned to real
commits in real repositories. When a rule catches nothing new three
times in a row, we froze rule expansion and said so. The hypothesis
that text rules could reach a 20% miss rate is recorded, in our own
STATUS file, as FALSIFIED.

Why live this way? Because the teams INVAR serves have been burned by
tools that claimed everything and whispered their limits in a footnote.
Trust in this category is not built by a bigger number; it is built by
a number that cannot lie. 38 of 70 is not where we want to stay. But it
is where we are, it is measured, it is sealed, and you can check it
yourself in under a minute. That is the product.
