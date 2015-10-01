  $ $PYTHON -c 'import remotenames' || exit 80
  $ cat >> $HGRCPATH << EOF
  > [ui]
  > ssh = python "$RUNTESTDIR/dummyssh"
  > [extensions]
  > remotenames =
  > pushrebase = $TESTDIR/../pushrebase.py
  > [remotenames]
  > allownonfastforward=True
  > [experimental]
  > bundle2-exp=True
  > EOF

Set up server repository

  $ hg init server
  $ cd server
  $ echo foo > a
  $ echo foo > b
  $ hg commit -Am 'initial'
  adding a
  adding b
  $ hg book master
  $ cd ..

Set up client repository

  $ hg clone ssh://user@dummy/server client -q

Test that pushing to a remotename gets rebased

  $ cd server
  $ hg up -q master
  $ echo x >> a && hg commit -m "master's commit"
  $ cd ../client
  $ echo x >> b && hg commit -m "client's commit"
  $ hg log -G -T '{rev} "{desc}" {remotebookmarks}'
  @  1 "client's commit"
  |
  o  0 "initial" default/master
  

  $ hg push --to master
  pushing rev 5c3cfb78df2f to destination ssh://user@dummy/server bookmark master
  searching for changes
  adding changesets
  adding manifests
  adding file changes
  added 2 changesets with 1 changes to 2 files (+1 heads)
  updating bookmark master

  $ hg log -G -T '{rev} "{desc}" {remotebookmarks}'
  o  3 "client's commit" default/master
  |
  o  2 "master's commit"
  |
  | @  1 "client's commit"
  |/
  o  0 "initial"
  

  $ cd ../server
  $ hg log -G -T '{rev} "{desc}" {bookmarks}'
  o  2 "client's commit" master
  |
  @  1 "master's commit"
  |
  o  0 "initial"
  
Test pushing a new bookmark
  $ cd ..
  $ hg -R client push --to newbook
  pushing rev 5c3cfb78df2f to destination ssh://user@dummy/server bookmark newbook
  searching for changes
  abort: not creating new bookmark
  (use --force to create a new bookmark)
  [255]

  $ hg -R client push --to newbook -f
  pushing rev 5c3cfb78df2f to destination ssh://user@dummy/server bookmark newbook
  searching for changes
  remote: adding changesets
  remote: adding manifests
  remote: adding file changes
  remote: added 1 changesets with 0 changes to 1 files (+1 heads)
  exporting bookmark newbook
  $ hg -R server book
   * master                    2:796d44dcaae0
     newbook                   3:5c3cfb78df2f
  $ hg -R server log -G -T '{rev} "{desc}" {bookmarks}'
  o  3 "client's commit" newbook
  |
  | o  2 "client's commit" master
  | |
  | @  1 "master's commit"
  |/
  o  0 "initial"
  
Test doing a non-fastforward bookmark move

  $ hg -R client push --to newbook -r master -f
  pushing rev 796d44dcaae0 to destination ssh://user@dummy/server bookmark newbook
  searching for changes
  no changes found
  updating bookmark newbook
  [1]
  $ hg -R server log -G -T '{rev} "{desc}" {bookmarks}'
  o  3 "client's commit"
  |
  | o  2 "client's commit" master newbook
  | |
  | @  1 "master's commit"
  |/
  o  0 "initial"
  

Test a push that comes with out-of-date bookmark discovery

  $ hg -R server strip -q 0 --config extensions.strip=
  $ hg -R client strip -q 0 --config extensions.strip=
  $ rm server/.hg/bookmarks*
  $ rm client/.hg/bookmarks*
  $ echo a >> server/a
  $ hg -R server commit -qAm 'aa'
  $ hg -R server bookmark bm -i
  $ echo b >> server/b
  $ hg -R server commit -qAm 'bb'
  $ hg -R server log -G -T '{rev} "{desc}" {bookmarks}'
  @  1 "bb"
  |
  o  0 "aa" bm
  

  $ cat >> $TESTTMP/move.py <<EOF
  > def movebookmark(ui, repo, **kwargs):
  >     bm = repo._bookmarks
  >     bm['bm'] = repo[1].node()
  >     bm.write()
  >     print "moved bookmark to rev 1"
  > EOF
  $ cat >> server/.hg/hgrc <<EOF
  > [hooks]
  > pretxnopen.movebook = python:$TESTTMP/move.py:movebookmark
  > EOF
  $ hg -R client pull -q -r 0
  $ hg -R client update -q 0
  $ echo c >> client/c
  $ hg -R client commit -qAm 'cc'
  $ hg -R client log -G -T '{rev} "{desc}" {bookmarks}'
  @  1 "cc"
  |
  o  0 "aa"
  
  $ hg -R client push --to bm
  pushing rev 5db65b93a12b to destination ssh://user@dummy/server bookmark bm
  searching for changes
  remote: moved bookmark to rev 1
  adding changesets
  adding manifests
  adding file changes
  added 2 changesets with 1 changes to 2 files (+1 heads)
  updating bookmark bm
  $ hg -R server log -G -T '{rev} "{desc}" {bookmarks}'
  o  2 "cc" bm
  |
  @  1 "bb"
  |
  o  0 "aa"
  
  $ hg -R client log -G -T '{rev} "{desc}" {bookmarks}'
  o  3 "cc"
  |
  o  2 "bb"
  |
  | @  1 "cc"
  |/
  o  0 "aa"
  

Test that we still don't allow non-ff bm changes

  $ echo d > client/d
  $ hg -R client commit -qAm "dd"
  $ hg -R client log -G -T '{rev} "{desc}" {bookmarks}'
  @  4 "dd"
  |
  | o  3 "cc"
  | |
  | o  2 "bb"
  | |
  o |  1 "cc"
  |/
  o  0 "aa"
  

  $ hg -R client push --to bm
  pushing rev efec53e7b035 to destination ssh://user@dummy/server bookmark bm
  searching for changes
  remote: moved bookmark to rev 1
  remote: transaction abort!
  remote: rollback completed
  abort: updating bookmark bm failed!
  [255]
