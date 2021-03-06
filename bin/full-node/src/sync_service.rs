// Smoldot
// Copyright (C) 2019-2021  Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//! Background synchronization service.
//!
//! The [`SyncService`] manages a background task dedicated to synchronizing the chain with the
//! network.
//! Importantly, its design is oriented towards the particular use case of the full node.

// TODO: doc
// TODO: re-review this once finished

use core::{num::NonZeroU32, pin::Pin};
use futures::{
    channel::{mpsc, oneshot},
    lock::Mutex,
    prelude::*,
};
use smoldot::{chain::sync::optimistic, database::full_sled, libp2p, network};
use std::{collections::BTreeMap, sync::Arc, time::SystemTime};
use tracing::Instrument as _;

/// Configuration for a [`SyncService`].
pub struct Config {
    /// Closure that spawns background tasks.
    pub tasks_executor: Box<dyn FnMut(Pin<Box<dyn Future<Output = ()> + Send>>) + Send>,

    /// Database to use to read and write information about the chain.
    pub database: Arc<full_sled::SledFullDatabase>,
}

/// Event generated by [`SyncService::next_event`].
#[derive(Debug)]
pub enum Event {
    BlocksRequest {
        id: BlocksRequestId,
        target: libp2p::PeerId,
        request: network::protocol::BlocksRequestConfig,
    },
}

/// Identifier for a blocks request to be performed.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct BlocksRequestId(usize);

/// Summary of the state of the [`SyncService`].
#[derive(Debug, Clone)]
pub struct SyncState {
    pub best_block_number: u64,
    pub best_block_hash: [u8; 32],
    pub finalized_block_number: u64,
    pub finalized_block_hash: [u8; 32],
}

/// Background task that verifies blocks and emits requests.
pub struct SyncService {
    /// State kept up-to-date with the background task.
    sync_state: Arc<Mutex<SyncState>>,

    /// Sender of messages towards the background task.
    to_background: Mutex<mpsc::Sender<ToBackground>>,

    /// Receiver of events sent by the background task.
    from_background: Mutex<mpsc::Receiver<FromBackground>>,

    /// For each emitted blocks request, an element is stored here.
    ///
    /// A regular `Mutex` is used in order to avoid issues with futures cancellation.
    blocks_requests: parking_lot::Mutex<
        slab::Slab<oneshot::Sender<Result<Vec<network::protocol::BlockData>, ()>>>,
    >,
}

impl SyncService {
    /// Initializes the [`SyncService`] with the given configuration.
    #[tracing::instrument(skip(config))]
    pub async fn new(mut config: Config) -> Arc<Self> {
        let (to_foreground, from_background) = mpsc::channel(16);
        let (to_background, from_foreground) = mpsc::channel(16);
        let (to_database, messages_rx) = mpsc::channel(4);

        let finalized_block_hash = config.database.finalized_block_hash().unwrap();

        let sync_state = Arc::new(Mutex::new(SyncState {
            best_block_hash: [0; 32],      // TODO:
            best_block_number: 0,          // TODO:
            finalized_block_hash: [0; 32], // TODO:
            finalized_block_number: 0,     // TODO:
        }));

        (config.tasks_executor)(Box::pin(start_sync(
            config.database.clone(),
            sync_state.clone(),
            to_foreground,
            from_foreground,
            to_database,
        )));

        (config.tasks_executor)(Box::pin(
            start_database_write(config.database, messages_rx).instrument(
                tracing::debug_span!(parent: None, "database-write", root = ?finalized_block_hash), // TDOO: better display
            ),
        ));

        Arc::new(SyncService {
            sync_state,
            to_background: Mutex::new(to_background),
            from_background: Mutex::new(from_background),
            blocks_requests: parking_lot::Mutex::new(slab::Slab::new()),
        })
    }

    /// Returns a summary of the state of the service.
    ///
    /// > **Important**: This doesn't represent the content of the database.
    // TODO: maybe remove this in favour of the database; seems like a better idea
    #[tracing::instrument(skip(self))]
    pub async fn sync_state(&self) -> SyncState {
        self.sync_state.lock().await.clone()
    }

    /// Registers a new source for blocks.
    #[tracing::instrument(skip(self))]
    pub async fn add_source(&self, peer_id: libp2p::PeerId, best_block_number: u64) {
        self.to_background
            .lock()
            .await
            .send(ToBackground::PeerConnected(peer_id, best_block_number))
            .await
            .unwrap()
    }

    /// Removes a source of blocks.
    #[tracing::instrument(skip(self))]
    pub async fn remove_source(&self, peer_id: libp2p::PeerId) {
        self.to_background
            .lock()
            .await
            .send(ToBackground::PeerDisconnected(peer_id))
            .await
            .unwrap()
    }

    /// Updates the best known block of the source.
    ///
    /// Has no effect if the previously-known best block is lower than the new one.
    #[tracing::instrument(skip(self))]
    pub async fn raise_source_best_block(&self, peer_id: libp2p::PeerId, best_block_number: u64) {
        self.to_background
            .lock()
            .await
            .send(ToBackground::PeerRaiseBest {
                peer_id,
                best_block_number,
            })
            .await
            .unwrap()
    }

    /// Sets the answer to a previously-emitted [`Event::BlocksRequest`].
    ///
    /// After this has been called, the `id` is no longer valid.
    ///
    /// # Panic
    ///
    /// Panics if the `id` is invalid.
    ///
    #[tracing::instrument(skip(self, response))]
    pub async fn answer_blocks_request(
        &self,
        id: BlocksRequestId,
        response: Result<Vec<network::protocol::BlockData>, ()>,
    ) {
        let _ = self.blocks_requests.lock().remove(id.0).send(response);
    }

    /// Returns the next event that happened in the sync service.
    ///
    /// If this method is called multiple times simultaneously, the events will be distributed
    /// amongst the different calls in an unpredictable way.
    #[tracing::instrument(skip(self))]
    pub async fn next_event(&self) -> Event {
        match self.from_background.lock().await.next().await.unwrap() {
            FromBackground::RequestStart {
                target,
                request,
                send_back,
            } => {
                let id = BlocksRequestId(self.blocks_requests.lock().insert(send_back));
                Event::BlocksRequest {
                    id,
                    target,
                    request,
                }
            }
        }
    }
}

/// Message sent to the background task.
enum ToBackground {
    PeerConnected(libp2p::PeerId, u64),
    PeerDisconnected(libp2p::PeerId),
    PeerRaiseBest {
        peer_id: libp2p::PeerId,
        best_block_number: u64,
    },
}

/// Messsage sent from the background task and dedicated to the main [`SyncService`]. Processed
/// in [`SyncService::next_event`].
enum FromBackground {
    /// A blocks request must be started.
    RequestStart {
        target: libp2p::PeerId,
        request: network::protocol::BlocksRequestConfig,
        send_back: oneshot::Sender<Result<Vec<network::protocol::BlockData>, ()>>, // TODO: proper error
    },
}

enum ToDatabase {
    FinalizedBlocks(Vec<optimistic::Block<()>>),
}

/// Returns the background task of the sync service.
#[tracing::instrument(skip(database, sync_state, to_foreground, from_foreground, to_database))]
fn start_sync(
    database: Arc<full_sled::SledFullDatabase>,
    sync_state: Arc<Mutex<SyncState>>,
    mut to_foreground: mpsc::Sender<FromBackground>,
    mut from_foreground: mpsc::Receiver<ToBackground>,
    mut to_database: mpsc::Sender<ToDatabase>,
) -> impl Future<Output = ()> {
    let mut sync = optimistic::OptimisticSync::<_, libp2p::PeerId, ()>::new(optimistic::Config {
        chain_information: database
            .to_chain_information(&database.finalized_block_hash().unwrap())
            .unwrap(),
        sources_capacity: 32,
        blocks_capacity: {
            // This is the maximum number of blocks between two consecutive justifications.
            1024
        },
        source_selection_randomness_seed: rand::random(),
        blocks_request_granularity: NonZeroU32::new(128).unwrap(),
        download_ahead_blocks: {
            // Assuming a verification speed of 1k blocks/sec and a 95% latency of one second,
            // the number of blocks to download ahead of time in order to not block is 1000.
            1024
        },
        full: true,
    });

    // Holds, in parallel of the database, the storage of the latest finalized block.
    // At the time of writing, this state is stable around ~3MiB for Polkadot, meaning that it is
    // completely acceptable to hold it entirely in memory.
    // While reading the storage from the database is an option, doing so considerably slows down
    // the verification, and also makes it impossible to insert blocks in the database in
    // parallel of this verification.
    let mut finalized_block_storage = BTreeMap::<Vec<u8>, Vec<u8>>::new();
    for key in database
        .finalized_block_storage_top_trie_keys(&database.finalized_block_hash().unwrap(), &[])
        .unwrap()
    {
        if let Some(value) = database
            .finalized_block_storage_top_trie_get(&database.finalized_block_hash().unwrap(), &key)
            .unwrap()
        {
            finalized_block_storage.insert(key.to_owned(), value.to_owned());
        }
    }

    async move {
        let mut peers_source_id_map = hashbrown::HashMap::<_, _, fnv::FnvBuildHasher>::default();
        let mut block_requests_finished = stream::FuturesUnordered::new();

        loop {
            let unix_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();

            // Verify blocks that have been fetched from queries.
            let mut process = sync.process_one(unix_time);
            loop {
                match process {
                    optimistic::ProcessOne::Idle { sync: s } => {
                        sync = s;
                        break;
                    }
                    optimistic::ProcessOne::Reset {
                        sync: s,
                        previous_best_height,
                        reason,
                    } => {
                        tracing::warn!(%reason, %previous_best_height, "failed-block-verification");
                        process = s.process_one(unix_time);
                    }
                    optimistic::ProcessOne::Finalized {
                        sync: s,
                        finalized_blocks,
                    } => {
                        process = s.process_one(unix_time);

                        if let Some(last_finalized) = finalized_blocks.last() {
                            let mut lock = sync_state.lock().await;
                            lock.finalized_block_hash = last_finalized.header.hash();
                            lock.finalized_block_number = last_finalized.header.number;
                        }

                        // TODO: maybe write in a separate task? but then we can't access the finalized storage immediately after?
                        for block in &finalized_blocks {
                            for (key, value) in &block.storage_top_trie_changes {
                                if let Some(value) = value {
                                    finalized_block_storage.insert(key.clone(), value.clone());
                                } else {
                                    let _was_there = finalized_block_storage.remove(key);
                                    // TODO: if a block inserts a new value, then removes it in the next block, the key will remain in `finalized_block_storage`; either solve this or document this
                                    // assert!(_was_there.is_some());
                                }
                            }
                        }

                        to_database
                            .send(ToDatabase::FinalizedBlocks(finalized_blocks))
                            .await
                            .unwrap();
                    }

                    optimistic::ProcessOne::NewBest {
                        sync: s,
                        new_best_hash,
                        new_best_number,
                    } => {
                        // Processing has made a step forward.
                        // There is nothing to do, but this is used to update to best block
                        // shown on the informant.
                        let mut lock = sync_state.lock().await;
                        lock.best_block_hash = new_best_hash;
                        lock.best_block_number = new_best_number;
                        drop(lock);

                        process = s.process_one(unix_time);
                    }

                    optimistic::ProcessOne::FinalizedStorageGet(req) => {
                        let value = finalized_block_storage
                            .get(&req.key_as_vec())
                            .map(|v| &v[..]);
                        process = req.inject_value(value);
                    }
                    optimistic::ProcessOne::FinalizedStorageNextKey(req) => {
                        // TODO: to_vec() :-/
                        let req_key = req.key().to_vec();
                        // TODO: to_vec() :-/
                        let next_key = finalized_block_storage
                            .range(req.key().to_vec()..)
                            .find(move |(k, _)| k[..] > req_key[..])
                            .map(|(k, _)| k);
                        process = req.inject_key(next_key);
                    }
                    optimistic::ProcessOne::FinalizedStoragePrefixKeys(req) => {
                        // TODO: to_vec() :-/
                        let prefix = req.prefix().to_vec();
                        // TODO: to_vec() :-/
                        let keys = finalized_block_storage
                            .range(req.prefix().to_vec()..)
                            .take_while(|(k, _)| k.starts_with(&prefix))
                            .map(|(k, _)| k);
                        process = req.inject_keys(keys);
                    }
                }
            }

            // Update the current best block, used for CLI-related purposes.
            {
                let mut lock = sync_state.lock().await;
                lock.best_block_hash = sync.best_block_hash();
                lock.best_block_number = sync.best_block_number();
            }

            // Start requests that need to be started.
            // Note that this is done after calling `process_one`, as the processing of pending
            // blocks can result in new requests but not the contrary.
            while let Some(action) = sync.next_request_action() {
                match action {
                    optimistic::RequestAction::Start {
                        start,
                        block_height,
                        source,
                        num_blocks,
                        ..
                    } => {
                        let (send_back, rx) = oneshot::channel();

                        let send_result = to_foreground
                            .send(FromBackground::RequestStart {
                                target: source.clone(),
                                request: network::protocol::BlocksRequestConfig {
                                    start: network::protocol::BlocksRequestConfigStart::Number(
                                        block_height,
                                    ),
                                    desired_count: num_blocks,
                                    direction: network::protocol::BlocksRequestDirection::Ascending,
                                    fields: network::protocol::BlocksRequestFields {
                                        header: true,
                                        body: true,
                                        justification: true,
                                    },
                                },
                                send_back,
                            })
                            .await;

                        // If the channel is closed, the sync service has been closed too.
                        if send_result.is_err() {
                            return;
                        }

                        let (rx, abort) = future::abortable(rx);
                        let request_id = start.start(abort);
                        block_requests_finished.push(rx.map(move |r| (request_id, r)));
                    }
                    optimistic::RequestAction::Cancel { user_data, .. } => {
                        user_data.abort();
                    }
                }
            }

            futures::select! {
                message = from_foreground.next() => {
                    let message = match message {
                        Some(m) => m,
                        None => return,
                    };

                    match message {
                        ToBackground::PeerConnected(peer_id, best_block_number) => {
                            let id = sync.add_source(peer_id.clone(), best_block_number);
                            peers_source_id_map.insert(peer_id.clone(), id);
                        },
                        ToBackground::PeerDisconnected(peer_id) => {
                            let id = peers_source_id_map.remove(&peer_id).unwrap();
                            let (_, rq_list) = sync.remove_source(id);
                            for (_, rq) in rq_list {
                                rq.abort();
                            }
                        },
                        ToBackground::PeerRaiseBest { peer_id, best_block_number } => {
                            let id = *peers_source_id_map.get(&peer_id).unwrap();
                            sync.raise_source_best_block(id, best_block_number);
                        }
                    }
                },

                (request_id, result) = block_requests_finished.select_next_some() => {
                    // `result` is an error if the block request got cancelled by the sync state
                    // machine.
                    // TODO: clarify this piece of code
                    if let Ok(result) = result {
                        let result = result.map_err(|_| ()).and_then(|v| v);
                        let _ = sync.finish_request(request_id, result.map(|v| v.into_iter().map(|block| optimistic::RequestSuccessBlock {
                            scale_encoded_header: block.header.unwrap(), // TODO: don't unwrap
                            scale_encoded_extrinsics: block.body.unwrap(), // TODO: don't unwrap
                            scale_encoded_justification: block.justification,
                            user_data: (),
                        })).map_err(|()| optimistic::RequestFail::BlocksUnavailable));
                    }
                },
            }
        }
    }
}

/// Starts the task that writes blocks to the database.
#[tracing::instrument(skip(database, messages_rx))]
async fn start_database_write(
    database: Arc<full_sled::SledFullDatabase>,
    mut messages_rx: mpsc::Receiver<ToDatabase>,
) {
    loop {
        match messages_rx.next().await {
            None => break,
            Some(ToDatabase::FinalizedBlocks(finalized_blocks)) => {
                let span = tracing::trace_span!("blocks-db-write", len = finalized_blocks.len());
                let _enter = span.enter();

                let new_finalized_hash = if let Some(last_finalized) = finalized_blocks.last() {
                    Some(last_finalized.header.hash())
                } else {
                    None
                };

                for block in finalized_blocks {
                    // TODO: overhead for building the SCALE encoding of the header
                    let result = database.insert(
                        &block.header.scale_encoding().fold(Vec::new(), |mut a, b| {
                            a.extend_from_slice(b.as_ref());
                            a
                        }),
                        true, // TODO: is_new_best?
                        block.body.iter(),
                        block
                            .storage_top_trie_changes
                            .iter()
                            .map(|(k, v)| (k, v.as_ref())),
                    );

                    match result {
                        Ok(()) => {}
                        Err(full_sled::InsertError::Duplicate) => {} // TODO: this should be an error ; right now we silence them because non-finalized blocks aren't loaded from the database at startup, resulting in them being downloaded again
                        Err(err) => panic!("{}", err),
                    }
                }

                if let Some(new_finalized_hash) = new_finalized_hash {
                    database.set_finalized(&new_finalized_hash).unwrap();
                }
            }
        }
    }
}
