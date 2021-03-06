#chg-compatible

TODO: configure mutation
  $ configure noevolution
  $ enable rebase
  $ setconfig phases.publish=false

Rebasing using a single transaction

  $ hg init singletr && cd singletr
  $ setconfig rebase.singletransaction=true
  $ hg debugdrawdag <<'EOF'
  >   Z
  >   |
  >   | D
  >   | |
  >   | C
  >   | |
  >   Y B
  >   |/
  >   A
  > EOF
- We should only see two status stored messages. One from the start, one from
- the end.
  $ hg rebase --debug -b D -d Z | grep 'status stored'
  rebase status stored
  rebase status stored
  $ tglog
  o  5: a701fddfacec 'D' D
  |
  o  4: abc67d0cf023 'C' C
  |
  o  3: 9a6b5541d0c0 'B' B
  |
  o  2: e9b22a392ce0 'Z' Z
  |
  o  1: 633ae0eca5f4 'Y' Y
  |
  o  0: 426bada5c675 'A' A
  
  $ cd ..
