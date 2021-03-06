/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

//! revisionstore - Data and history store for generic revision data (usually commit, manifest,
//! and file data)

mod contentstore;
mod dataindex;
mod edenapi;
#[cfg(all(fbcode_build, target_os = "linux"))]
mod facebook;
mod fanouttable;
mod historyindex;
mod indexedloghistorystore;
mod indexedlogutil;
mod lfs;
mod memcache;
mod metadatastore;
mod remotestore;
mod repack;
mod sliceext;
mod types;
mod unionstore;
mod util;

pub mod c_api;
pub mod datapack;
pub mod datastore;
pub mod error;
pub mod historypack;
pub mod historystore;
pub mod indexedlogdatastore;
pub mod localstore;
pub mod multiplexstore;
pub mod mutabledatapack;
pub mod mutablehistorypack;
pub mod mutablepack;
pub mod packstore;
pub mod packwriter;
pub mod uniondatastore;
pub mod unionhistorystore;

pub use crate::contentstore::{ContentStore, ContentStoreBuilder};
pub use crate::datapack::{DataEntry, DataPack, DataPackVersion};
pub use crate::datastore::{
    ContentDataStore, ContentMetadata, Delta, HgIdDataStore, HgIdMutableDeltaStore, RemoteDataStore,
};
pub use crate::edenapi::EdenApiHgIdRemoteStore;
pub use crate::historypack::{HistoryEntry, HistoryPack, HistoryPackVersion};
pub use crate::historystore::{HgIdHistoryStore, HgIdMutableHistoryStore, RemoteHistoryStore};
pub use crate::indexedlogdatastore::IndexedLogHgIdDataStore;
pub use crate::indexedloghistorystore::IndexedLogHgIdHistoryStore;
pub use crate::localstore::LocalStore;
pub use crate::memcache::MemcacheStore;
pub use crate::metadatastore::{MetadataStore, MetadataStoreBuilder};
pub use crate::multiplexstore::{MultiplexDeltaStore, MultiplexHgIdHistoryStore};
pub use crate::mutabledatapack::MutableDataPack;
pub use crate::mutablehistorypack::MutableHistoryPack;
pub use crate::packstore::{
    CorruptionPolicy, DataPackStore, HistoryPackStore, MutableDataPackStore,
    MutableHistoryPackStore,
};
pub use crate::remotestore::HgIdRemoteStore;
pub use crate::repack::{repack, RepackKind, RepackLocation, Repackable, ToKeys};
pub use crate::types::{ContentHash, StoreKey};
pub use crate::uniondatastore::UnionHgIdDataStore;
pub use crate::util::Error;

pub use indexedlog::Repair as IndexedlogRepair;
pub use revisionstore_types::*;

#[cfg(any(test, feature = "for-tests"))]
pub mod testutil;
