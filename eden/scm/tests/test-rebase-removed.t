Rebase changes made on copied (forked) source code:

  $ newrepo
  $ drawdag <<'EOS'
  > D E # E/C=1\n2\n3e\n
  > | |
  > B C # C/C=1\n2\n3\n
  > |/  # B/A=1b\n2\n3\n
  > A   # A/A=1\n2\n3\n
  > EOS

 (try normal rebase - fails)
  $ hg rebase -r $E -d $D
  rebasing 8c0ff6bd3515 "E"
  other [source] changed C which local [dest] deleted
  use (c)hanged version, leave (d)eleted, leave (u)nresolved, or input (r)enamed path? u
  unresolved conflicts (see hg resolve, then hg rebase --continue)
  [1]
  $ hg rebase --abort
  rebase aborted

 (try rebase with a script saying "C" was renamed to "A")
  $ hg rebase -r $E -d $D --config experimental.rename-cmd='echo A'
  rebasing 8c0ff6bd3515 "E"
  running 'echo A' to find rename destination of C
   trying rename destination: A
  merging A
 (changed in "A" developed in copied "C" are merged back to "A")
  $ hg log -r tip -T '{desc}\n' -p --git
  E
  diff --git a/A b/A
  --- a/A
  +++ b/A
  @@ -1,3 +1,3 @@
   1b
   2
  -3
  +3e
  diff --git a/E b/E
  new file mode 100644
  --- /dev/null
  +++ b/E
  @@ -0,0 +1,1 @@
  +E
  \ No newline at end of file
  

A similar setup. C/C is marked as copied from A.
  $ newrepo
  $ drawdag <<'EOS'
  > D E # E/C=1\n2\n3e\n
  > | |
  > B C # C/C=1\n2\n3\n (copied from A)
  > |/  # B/A=1b\n2\n3\n
  > A   # A/A=1\n2\n3\n
  > EOS

BUG: Changes to the file "C" made in commit "E" shouldn't get lost:
  $ hg rebase -r $E -d $D
  rebasing a0b6e0c8e32c "E"

  $ hg log -r tip -T '{desc}\n' -p --git
  E
  diff --git a/E b/E
  new file mode 100644
  --- /dev/null
  +++ b/E
  @@ -0,0 +1,1 @@
  +E
  \ No newline at end of file
  