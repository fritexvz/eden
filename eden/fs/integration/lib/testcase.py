#!/usr/bin/env python3
#
# Copyright (c) 2016, Facebook, Inc.
# All rights reserved.
#
# This source code is licensed under the BSD-style license found in the
# LICENSE file in the root directory of this source tree. An additional grant
# of patent rights can be found in the PATENTS file in the same directory.

import os
import shutil
import tempfile
import unittest
from . import edenclient
from . import hgrepo
from . import gitrepo


@unittest.skipIf(not edenclient.can_run_eden(), "unable to run edenfs")
class EdenTestCase(unittest.TestCase):
    '''
    Base class for eden integration test cases.

    This starts an eden daemon during setUp(), and cleans it up during
    tearDown().
    '''
    def setUp(self):
        self.tmp_dir = None
        self.eden = None
        self.old_home = None

        # Call setup_eden_test() to do most of the setup work, and call
        # tearDown() on any error.  tearDown() won't be called by default if
        # setUp() throws.
        try:
            self.setup_eden_test()
        except Exception as ex:
            self.tearDown()
            raise

    def setup_eden_test(self):
        self.tmp_dir = tempfile.mkdtemp(prefix='eden_test.')

        # The eden config directory
        self.eden_dir = os.path.join(self.tmp_dir, 'eden')
        # The home directory, to make sure eden looks at this rather than the
        # real home directory of the user running the tests.
        self.home_dir = os.path.join(self.tmp_dir, 'homedir')
        os.mkdir(self.home_dir)
        self.old_home = os.getenv('HOME')
        os.environ['HOME'] = self.home_dir
        # Parent directory for any git/hg repositories created during the test
        self.repos_dir = os.path.join(self.tmp_dir, 'repos')
        os.mkdir(self.repos_dir)
        # Parent directory for eden mount points
        self.mounts_dir = os.path.join(self.tmp_dir, 'mounts')
        os.mkdir(self.mounts_dir)

        self.eden = edenclient.EdenFS(self.eden_dir, home_dir=self.home_dir)
        self.eden.start()

    def tearDown(self):
        error = None
        try:
            if self.eden is not None:
                self.eden.cleanup()
        except Exception as ex:
            error = ex

        if self.old_home is not None:
            os.environ['HOME'] = self.old_home
            self.old_home = None

        if self.tmp_dir is not None:
            shutil.rmtree(self.tmp_dir, ignore_errors=True)
            self.tmp_dir = None

        # Re-raise any error that occurred, after we finish
        # trying to clean up our directories.
        if error is not None:
            raise error

    def get_thrift_client(self):
        '''
        Get a thrift client to the edenfs daemon.
        '''
        return self.eden.get_thrift_client()


class EdenRepoTestBase(EdenTestCase):
    '''
    Base class for EdenHgTest and EdenGitTest.

    This sets up a repository and mounts it before starting each test function.
    '''
    def setup_eden_test(self):
        super().setup_eden_test()

        self.repo_name = 'main'
        repo_path = os.path.join(self.repos_dir, self.repo_name)
        self.mount = os.path.join(self.mounts_dir, self.repo_name)

        os.mkdir(repo_path)
        self.repo = self.create_repo(repo_path)
        self.repo.init()
        self.populate_repo()

        self.eden.add_repository(self.repo_name, repo_path)
        self.eden.clone(self.repo_name, self.mount)

    def populate_repo(self):
        raise NotImplementedError('individual test classes must implement '
                                  'populate_repo()')


class EdenHgTest(EdenRepoTestBase):
    '''
    Subclass of EdenTestCase which uses a single mercurial repository and
    eden mount.

    The repository is available as self.repo, and the client mount path is
    available as self.mount
    '''
    def create_repo(self, path):
        return hgrepo.HgRepository(path)


class EdenGitTest(EdenRepoTestBase):
    '''
    Subclass of EdenTestCase which uses a single mercurial repository and
    eden mount.

    The repository is available as self.repo, and the client mount path is
    available as self.mount
    '''
    def create_repo(self, path):
        return gitrepo.GitRepository(path)
