/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use anyhow::Result;
use cpython::*;
use cpython_ext::{AnyhowResultExt, ResultPyErrExt};
use dag::{nameset::NameIter, NameSet, VertexName};
use std::cell::RefCell;

/// A wrapper around [`NameSet`] with Python integration added.
///
/// Differences from the `py_class` version:
/// - Auto converts from a wider range of Python types - not just nameset, but
///   also List[bytes], and Generator[bytes].
/// - Pure Rust. No need to take the Python GIL to create `Names`.
pub struct Names(pub NameSet);

// A wrapper around [`NameSet`].
py_class!(pub class nameset |py| {
    data inner: NameSet;

    def __new__(_cls, obj: PyObject) -> PyResult<Self> {
        Ok(Names::extract(py, &obj)?.to_py_object(py))
    }

    def __contains__(&self, name: PyBytes) -> PyResult<bool> {
        let name = VertexName::copy_from(name.data(py));
        Ok(self.inner(py).contains(&name).map_pyerr(py)?)
    }

    def __len__(&self) -> PyResult<usize> {
        Ok(self.inner(py).count().map_pyerr(py)?)
    }

    def __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.inner(py)))
    }

    def __add__(lhs, rhs) -> PyResult<Names> {
        let lhs = Names::extract(py, lhs)?;
        let rhs = Names::extract(py, rhs)?;
        Ok(Names(lhs.0.union(&rhs.0)))
    }

    def __and__(lhs, rhs) -> PyResult<Names> {
        let lhs = Names::extract(py, lhs)?;
        let rhs = Names::extract(py, rhs)?;
        Ok(Names(lhs.0.intersection(&rhs.0)))
    }

    def __sub__(lhs, rhs) -> PyResult<Names> {
        let lhs = Names::extract(py, lhs)?;
        let rhs = Names::extract(py, rhs)?;
        Ok(Names(lhs.0.difference(&rhs.0)))
    }

    def __iter__(&self) -> PyResult<nameiter> {
        self.iter(py)
    }

    def iterrev(&self) -> PyResult<nameiter> {
        let iter = self.inner(py).clone().iter_rev().map_pyerr(py)?;
        let iter: RefCell<Box<dyn NameIter>> = RefCell::new(iter);
        nameiter::create_instance(py, iter)
    }

    def iter(&self) -> PyResult<nameiter> {
        let iter = self.inner(py).clone().iter().map_pyerr(py)?;
        let iter: RefCell<Box<dyn NameIter>> = RefCell::new(iter);
        nameiter::create_instance(py, iter)
    }

    def first(&self) -> PyResult<Option<PyBytes>> {
        Ok(self.inner(py).first().map_pyerr(py)?.map(|name| PyBytes::new(py, name.as_ref())))
    }

    def last(&self) -> PyResult<Option<PyBytes>> {
        Ok(self.inner(py).last().map_pyerr(py)?.map(|name| PyBytes::new(py, name.as_ref())))
    }

    /// Test if the set is topologically sorted. A sorted set has heads first, roots last.
    def issorted(&self) -> PyResult<bool> {
        Ok(self.inner(py).is_topo_sorted())
    }

    /// Mark the set as sorted, and return the marked-sorted set.
    /// This is usually useful for generator sets where the generator function
    /// ensures the set is sorted.
    def marksorted(&self) -> PyResult<Names> {
        Ok(Names(self.inner(py).mark_sorted()))
    }
});

// A wrapper to [`NameIter`].
py_class!(pub class nameiter |py| {
    data iter: RefCell<Box<dyn NameIter>>;

    def __next__(&self) -> PyResult<Option<PyBytes>> {
        let mut iter = self.iter(py).borrow_mut();
        let next: Option<VertexName> = iter.next().transpose().map_pyerr(py)?;
        Ok(next.map(|name| PyBytes::new(py, name.as_ref())))
    }

    def __iter__(&self) -> PyResult<nameiter> {
        Ok(self.clone_ref(py))
    }
});

impl<'a> FromPyObject<'a> for Names {
    fn extract(py: Python, obj: &'a PyObject) -> PyResult<Self> {
        // type(obj) is nameset - convert to Names directly.
        if let Ok(pyset) = obj.extract::<nameset>(py) {
            return Ok(Names(pyset.inner(py).clone()));
        }

        // type(obj) is list - convert to StaticSet
        if let Ok(pylist) = obj.extract::<Vec<PyBytes>>(py) {
            let set = NameSet::from_static_names(
                pylist
                    .into_iter()
                    .map(|name| VertexName::copy_from(name.data(py))),
            );
            return Ok(Names(set));
        }

        // Others - convert to LazySet.
        let obj = obj.clone_ref(py);
        let iter = PyNameIter::new(py, obj)?;
        let set = NameSet::from_iter(iter);
        Ok(Names(set))
    }
}

/// Similar to `PyIterator`, but without lifetime and has `VertexName` as
/// output type.
struct PyNameIter {
    obj: PyObject,
    errored: bool,
}

impl PyNameIter {
    fn new(py: Python, obj: PyObject) -> PyResult<Self> {
        let _obj = obj.iter(py)?;
        Ok(Self {
            obj,
            errored: false,
        })
    }
}

impl Iterator for PyNameIter {
    type Item = Result<VertexName>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            return None;
        }
        (|| -> PyResult<Option<VertexName>> {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let mut iter = self.obj.iter(py)?;
            match iter.next() {
                None => Ok(None),
                Some(Ok(value)) => {
                    let value = value.extract::<PyBytes>(py)?;
                    Ok(Some(VertexName::copy_from(value.data(py))))
                }
                Some(Err(err)) => Err(err),
            }
        })()
        .into_anyhow_result()
        .transpose()
    }
}

impl ToPyObject for Names {
    type ObjectType = nameset;

    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        nameset::create_instance(py, self.0.clone()).unwrap()
    }
}
