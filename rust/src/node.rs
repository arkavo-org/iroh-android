//! Minimal IrohNode for the Android binding.
//!
//! Blob-only surface (put/get + lifecycle) — no docs/gossip on Android yet,
//! since closurekb's Android client only needs blob transfer. Mirrors the
//! shape of `iroh-swift/rust/src/node.rs` so wire interop is one-to-one.

use anyhow::{Context, Result};
use iroh::endpoint::presets;
use iroh::endpoint::RelayMode;
use iroh::{Endpoint, RelayMap, RelayUrl, protocol::Router};
use iroh_blobs::{ALPN as BLOBS_ALPN, BlobsProtocol, store::fs::FsStore, ticket::BlobTicket};
use std::path::PathBuf;
use tokio::runtime::Runtime;

/// A self-contained Iroh blob node.
///
/// Owns its own tokio runtime so JNI calls can be blocking without
/// fighting any Kotlin/Java thread pool. Each `IrohNode` is a full peer:
/// endpoint + persistent FsStore + blob router.
pub struct IrohNode {
    runtime: Runtime,
    endpoint: Endpoint,
    store: FsStore,
    /// Router must be retained — dropping it closes accept loops.
    #[allow(dead_code)]
    router: Router,
}

impl IrohNode {
    pub fn new(
        storage_path: PathBuf,
        relay_enabled: bool,
        custom_relay_url: Option<String>,
    ) -> Result<Self> {
        let runtime = Runtime::new().context("Failed to create tokio runtime")?;

        let (endpoint, store, router) = runtime.block_on(async {
            let store = FsStore::load(&storage_path)
                .await
                .context("Failed to load blob store")?;

            let mut builder = Endpoint::builder(presets::N0);
            if !relay_enabled {
                builder = builder.relay_mode(RelayMode::Disabled);
            } else if let Some(url) = custom_relay_url {
                let relay_url: RelayUrl = url.parse().context("Invalid relay URL")?;
                let relay_map = RelayMap::from(relay_url);
                builder = builder.relay_mode(RelayMode::Custom(relay_map));
            }

            let endpoint = builder.bind().await.context("Failed to bind endpoint")?;
            if relay_enabled {
                let _ = endpoint.online().await;
            }

            let blobs = BlobsProtocol::new(&store, None);
            let router = Router::builder(endpoint.clone())
                .accept(BLOBS_ALPN, blobs)
                .spawn();

            Ok::<_, anyhow::Error>((endpoint, store, router))
        })?;

        Ok(Self {
            runtime,
            endpoint,
            store,
            router,
        })
    }

    /// Hex-encoded node id (matches what other peers see).
    pub fn node_id(&self) -> String {
        self.endpoint.id().to_string()
    }

    /// Add bytes locally; return a `BlobTicket` string others can use to fetch.
    pub fn put(&self, data: &[u8]) -> Result<String> {
        self.runtime.block_on(async {
            let tag = self
                .store
                .add_slice(data)
                .await
                .context("Failed to add bytes to blob store")?;
            let addr = self.endpoint.addr();
            let ticket = BlobTicket::new(addr, tag.hash, tag.format);
            Ok(ticket.to_string())
        })
    }

    /// Fetch bytes referenced by a ticket. Downloads from the remote peer if
    /// not already present locally.
    pub fn get(&self, ticket_str: &str) -> Result<Vec<u8>> {
        self.runtime.block_on(async {
            let ticket: BlobTicket = ticket_str.parse().context("Failed to parse ticket")?;
            let downloader = self.store.downloader(&self.endpoint);
            downloader
                .download(ticket.hash(), [ticket.addr().id])
                .await
                .context("Failed to download blob")?;
            let bytes = self
                .store
                .get_bytes(ticket.hash())
                .await
                .context("Failed to read bytes from store")?;
            Ok(bytes.to_vec())
        })
    }
}
