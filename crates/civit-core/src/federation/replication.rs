#![forbid(unsafe_code)]

use crate::federation::vector_clock::VectorClock;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegionId(pub String);

impl std::fmt::Display for RegionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationMessage {
    pub source_region: RegionId,
    pub target_region: RegionId,
    pub sequence: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload: ReplicationPayload,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicationPayload {
    SyncDelta {
        checkpoint_id: String,
        deltas: Vec<SyncDeltaEntry>,
    },
    FullSnapshot {
        data: Vec<u8>,
    },
    Heartbeat,
    PartitionNotice {
        partitioned_regions: Vec<RegionId>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDeltaEntry {
    pub table: String,
    pub operation: DeltaOperation,
    pub key: String,
    pub value: Option<Vec<u8>>,
    pub vector_clock: HashMap<RegionId, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaOperation {
    Insert,
    Update,
    Delete,
}

pub struct ReplicationPeer {
    pub region_id: RegionId,
    pub endpoint: String,
    pub healthy: Arc<RwLock<bool>>,
    pub last_seen: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl ReplicationPeer {
    pub fn new(region_id: RegionId, endpoint: String) -> Self {
        Self {
            region_id,
            endpoint,
            healthy: Arc::new(RwLock::new(true)),
            last_seen: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn is_healthy(&self) -> bool {
        *self.healthy.read().await
    }

    pub async fn mark_unhealthy(&self) {
        let mut h = self.healthy.write().await;
        *h = false;
    }

    pub async fn mark_healthy(&self) {
        let mut h = self.healthy.write().await;
        *h = true;
    }

    pub async fn update_last_seen(&self, ts: chrono::DateTime<chrono::Utc>) {
        let mut last = self.last_seen.write().await;
        *last = Some(ts);
    }

    pub async fn last_seen(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        *self.last_seen.read().await
    }
}

pub struct ReplicationTransport {
    region_id: RegionId,
    peers: Arc<RwLock<Vec<ReplicationPeer>>>,
    outbound_tx: tokio::sync::mpsc::Sender<ReplicationMessage>,
    inbound_rx: Arc<RwLock<Option<tokio::sync::mpsc::Receiver<ReplicationMessage>>>>,
    sequence: Arc<RwLock<u64>>,
}

impl ReplicationTransport {
    pub fn new(region_id: RegionId, buffer: usize) -> Self {
        let (outbound_tx, _) = tokio::sync::mpsc::channel(buffer);
        Self {
            region_id,
            peers: Arc::new(RwLock::new(Vec::new())),
            outbound_tx,
            inbound_rx: Arc::new(RwLock::new(None)),
            sequence: Arc::new(RwLock::new(0)),
        }
    }

    pub fn new_with_channels(
        region_id: RegionId,
        outbound_tx: tokio::sync::mpsc::Sender<ReplicationMessage>,
        inbound_rx: tokio::sync::mpsc::Receiver<ReplicationMessage>,
    ) -> Self {
        Self {
            region_id,
            peers: Arc::new(RwLock::new(Vec::new())),
            outbound_tx,
            inbound_rx: Arc::new(RwLock::new(Some(inbound_rx))),
            sequence: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn next_sequence(&self) -> u64 {
        let mut seq = self.sequence.write().await;
        *seq += 1;
        *seq
    }

    pub fn region_id(&self) -> &RegionId {
        &self.region_id
    }

    pub async fn register_peer(&self, peer: ReplicationPeer) {
        let mut peers = self.peers.write().await;
        let exists = peers.iter().any(|p| p.region_id == peer.region_id);
        if !exists {
            info!(
                peer = %peer.region_id.0,
                endpoint = %peer.endpoint,
                "registered replication peer"
            );
            peers.push(peer);
        }
    }

    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    pub async fn get_peer(&self, region_id: &RegionId) -> Option<Arc<tokio::sync::Mutex<()>>> {
        let peers = self.peers.read().await;
        peers
            .iter()
            .find(|p| &p.region_id == region_id)
            .map(|_| Arc::new(tokio::sync::Mutex::new(())))
    }

    pub async fn healthy_peers(&self) -> Vec<RegionId> {
        let peers = self.peers.read().await;
        let mut result = Vec::new();
        for peer in &*peers {
            if *peer.healthy.read().await {
                result.push(peer.region_id.clone());
            }
        }
        result
    }

    pub async fn send_to_peer(
        &self,
        target: &RegionId,
        payload: ReplicationPayload,
    ) -> Result<(), ReplicationError> {
        let peers = self.peers.read().await;
        let peer = peers
            .iter()
            .find(|p| &p.region_id == target)
            .ok_or(ReplicationError::PeerNotFound(target.clone()))?;

        if !*peer.healthy.read().await {
            return Err(ReplicationError::PeerUnhealthy(target.clone()));
        }

        let seq = self.next_sequence().await;
        let payload_bytes = serde_json::to_vec(&payload)
            .map_err(|e| ReplicationError::Serialization(e.to_string()))?;
        let checksum = compute_checksum(&payload_bytes);

        let msg = ReplicationMessage {
            source_region: self.region_id.clone(),
            target_region: target.clone(),
            sequence: seq,
            timestamp: Utc::now(),
            payload,
            checksum,
        };

        self.outbound_tx
            .send(msg)
            .await
            .map_err(|_| ReplicationError::ChannelClosed)?;

        debug!(
            seq = seq,
            target = %target.0,
            "sent replication message"
        );

        Ok(())
    }

    pub async fn receive(&self) -> Option<ReplicationMessage> {
        let mut rx = self.inbound_rx.write().await;
        if let Some(receiver) = rx.as_mut() {
            receiver.recv().await
        } else {
            None
        }
    }

    pub async fn set_inbound(&self, rx: tokio::sync::mpsc::Receiver<ReplicationMessage>) {
        let mut inbound = self.inbound_rx.write().await;
        *inbound = Some(rx);
    }

    pub async fn heartbeat_loop(
        &self,
        interval: std::time::Duration,
        mut cancel: tokio::sync::watch::Receiver<bool>,
    ) {
        let mut ticker = tokio::time::interval(interval);
        ticker.tick().await;
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let peers = self.peers.read().await;
                    let payload = ReplicationPayload::Heartbeat;
                    for peer in &*peers {
                        if !*peer.healthy.read().await {
                            continue;
                        }
                        let seq = self.next_sequence().await;
                        let payload_bytes = serde_json::to_vec(&payload).unwrap_or_default();
                        let msg = ReplicationMessage {
                            source_region: self.region_id.clone(),
                            target_region: peer.region_id.clone(),
                            sequence: seq,
                            timestamp: Utc::now(),
                            payload: ReplicationPayload::Heartbeat,
                            checksum: compute_checksum(&payload_bytes),
                        };
                        if self.outbound_tx.send(msg).await.is_err() {
                            break;
                        }
                    }
                }
                _ = cancel.changed() => {
                    break;
                }
            }
        }
    }

    pub async fn health_monitor(
        &self,
        timeout: std::time::Duration,
        mut cancel: tokio::sync::watch::Receiver<bool>,
    ) {
        let mut ticker = tokio::time::interval(timeout / 2);
        ticker.tick().await;
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let now = Utc::now();
                    let peers = self.peers.read().await;
                    for peer in &*peers {
                        let last = peer.last_seen.read().await;
                        if let Some(ts) = *last {
                            if now - ts > chrono::Duration::from_std(timeout).unwrap_or_default() {
                                warn!(
                                    peer = %peer.region_id.0,
                                    "peer marked unhealthy: heartbeat timeout"
                                );
                                drop(last);
                                peer.mark_unhealthy().await;
                            }
                        }
                    }
                }
                _ = cancel.changed() => {
                    break;
                }
            }
        }
    }
}

pub fn compute_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn verify_checksum(msg: &ReplicationMessage) -> Result<bool, ReplicationError> {
    let payload_bytes = serde_json::to_vec(&msg.payload)
        .map_err(|e| ReplicationError::Serialization(e.to_string()))?;
    let expected = compute_checksum(&payload_bytes);
    Ok(msg.checksum == expected)
}

#[derive(Debug, thiserror::Error)]
pub enum ReplicationError {
    #[error("peer not found: {0}")]
    PeerNotFound(RegionId),
    #[error("peer unhealthy: {0}")]
    PeerUnhealthy(RegionId),
    #[error("channel closed")]
    ChannelClosed,
    #[error("serialization error: {0}")]
    Serialization(String),
}

pub fn build_sync_delta_payload(
    checkpoint_id: String,
    table: &str,
    operation: DeltaOperation,
    key: &str,
    value: Option<Vec<u8>>,
    clock: &VectorClock<RegionId>,
) -> ReplicationPayload {
    let vc_map = clock
        .entries()
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    ReplicationPayload::SyncDelta {
        checkpoint_id,
        deltas: vec![SyncDeltaEntry {
            table: table.to_string(),
            operation,
            key: key.to_string(),
            value,
            vector_clock: vc_map,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_region(name: &str) -> RegionId {
        RegionId(name.to_string())
    }

    #[test]
    fn test_region_id_equality() {
        assert_eq!(make_region("us-east"), make_region("us-east"));
        assert_ne!(make_region("us-east"), make_region("eu-west"));
    }

    #[test]
    fn test_region_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(make_region("us-east"));
        set.insert(make_region("us-east"));
        set.insert(make_region("eu-west"));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_region_id_serialization() {
        let rid = make_region("ap-south");
        let json = serde_json::to_string(&rid).unwrap();
        let deser: RegionId = serde_json::from_str(&json).unwrap();
        assert_eq!(rid, deser);
    }

    #[test]
    fn test_compute_checksum_deterministic() {
        let data = b"hello world";
        let a = compute_checksum(data);
        let b = compute_checksum(data);
        assert_eq!(a, b);
        assert!(!a.is_empty());
    }

    #[test]
    fn test_compute_checksum_different_data() {
        let a = compute_checksum(b"hello");
        let b = compute_checksum(b"world");
        assert_ne!(a, b);
    }

    #[test]
    fn test_verify_checksum_valid() {
        let mut msg = ReplicationMessage {
            source_region: make_region("a"),
            target_region: make_region("b"),
            sequence: 1,
            timestamp: Utc::now(),
            payload: ReplicationPayload::Heartbeat,
            checksum: String::new(),
        };
        let payload_bytes = serde_json::to_vec(&msg.payload).unwrap();
        msg.checksum = compute_checksum(&payload_bytes);
        assert!(verify_checksum(&msg).unwrap());
    }

    #[test]
    fn test_verify_checksum_tampered() {
        let mut msg = ReplicationMessage {
            source_region: make_region("a"),
            target_region: make_region("b"),
            sequence: 1,
            timestamp: Utc::now(),
            payload: ReplicationPayload::Heartbeat,
            checksum: String::new(),
        };
        msg.checksum = "deadbeef".to_string();
        assert!(!verify_checksum(&msg).unwrap());
    }

    #[test]
    fn test_replication_message_serialization() {
        let msg = ReplicationMessage {
            source_region: make_region("us-east"),
            target_region: make_region("eu-west"),
            sequence: 42,
            timestamp: Utc::now(),
            payload: ReplicationPayload::Heartbeat,
            checksum: "abc".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deser: ReplicationMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.sequence, 42);
        assert_eq!(deser.source_region, make_region("us-east"));
        assert_eq!(deser.target_region, make_region("eu-west"));
    }

    #[test]
    fn test_sync_delta_payload_serialization() {
        let payload = ReplicationPayload::SyncDelta {
            checkpoint_id: "cp-1".to_string(),
            deltas: vec![SyncDeltaEntry {
                table: "repos".to_string(),
                operation: DeltaOperation::Insert,
                key: "repo-42".to_string(),
                value: Some(vec![1, 2, 3]),
                vector_clock: HashMap::new(),
            }],
        };
        let json = serde_json::to_string(&payload).unwrap();
        let deser: ReplicationPayload = serde_json::from_str(&json).unwrap();
        match deser {
            ReplicationPayload::SyncDelta {
                checkpoint_id,
                deltas,
            } => {
                assert_eq!(checkpoint_id, "cp-1");
                assert_eq!(deltas.len(), 1);
                assert_eq!(deltas[0].table, "repos");
            }
            _ => panic!("expected SyncDelta"),
        }
    }

    #[test]
    fn test_full_snapshot_payload_serialization() {
        let payload = ReplicationPayload::FullSnapshot {
            data: vec![0, 1, 2, 3, 4],
        };
        let json = serde_json::to_string(&payload).unwrap();
        let deser: ReplicationPayload = serde_json::from_str(&json).unwrap();
        match deser {
            ReplicationPayload::FullSnapshot { data } => {
                assert_eq!(data, vec![0, 1, 2, 3, 4]);
            }
            _ => panic!("expected FullSnapshot"),
        }
    }

    #[test]
    fn test_partition_notice_payload_serialization() {
        let payload = ReplicationPayload::PartitionNotice {
            partitioned_regions: vec![make_region("us-east"), make_region("eu-west")],
        };
        let json = serde_json::to_string(&payload).unwrap();
        let deser: ReplicationPayload = serde_json::from_str(&json).unwrap();
        match deser {
            ReplicationPayload::PartitionNotice {
                partitioned_regions,
            } => {
                assert_eq!(partitioned_regions.len(), 2);
            }
            _ => panic!("expected PartitionNotice"),
        }
    }

    #[test]
    fn test_build_sync_delta_payload() {
        let mut clock = VectorClock::new();
        clock.increment(&make_region("us-east"));
        let payload = build_sync_delta_payload(
            "cp-1".to_string(),
            "repos",
            DeltaOperation::Update,
            "repo-1",
            Some(vec![42]),
            &clock,
        );
        match payload {
            ReplicationPayload::SyncDelta {
                checkpoint_id,
                deltas,
            } => {
                assert_eq!(checkpoint_id, "cp-1");
                assert_eq!(deltas[0].operation, DeltaOperation::Update);
                assert_eq!(
                    deltas[0].vector_clock.get(&make_region("us-east")),
                    Some(&1)
                );
            }
            _ => panic!("expected SyncDelta"),
        }
    }

    #[tokio::test]
    async fn test_transport_new() {
        let transport = ReplicationTransport::new(make_region("us-east"), 64);
        assert_eq!(transport.region_id(), &make_region("us-east"));
        assert_eq!(transport.peer_count().await, 0);
    }

    #[tokio::test]
    async fn test_register_peer() {
        let transport = ReplicationTransport::new(make_region("us-east"), 64);
        transport
            .register_peer(ReplicationPeer::new(
                make_region("eu-west"),
                "https://eu-west.civitforge.internal:9090".to_string(),
            ))
            .await;
        assert_eq!(transport.peer_count().await, 1);
    }

    #[tokio::test]
    async fn test_register_peer_idempotent() {
        let transport = ReplicationTransport::new(make_region("us-east"), 64);
        transport
            .register_peer(ReplicationPeer::new(
                make_region("eu-west"),
                "https://eu-west:9090".to_string(),
            ))
            .await;
        transport
            .register_peer(ReplicationPeer::new(
                make_region("eu-west"),
                "https://eu-west:9091".to_string(),
            ))
            .await;
        assert_eq!(transport.peer_count().await, 1);
    }

    #[tokio::test]
    async fn test_send_to_nonexistent_peer() {
        let (tx, _rx) = tokio::sync::mpsc::channel(64);
        let transport = ReplicationTransport::new_with_channels(make_region("us-east"), tx, _rx);
        let result = transport
            .send_to_peer(&make_region("eu-west"), ReplicationPayload::Heartbeat)
            .await;
        assert!(matches!(result, Err(ReplicationError::PeerNotFound(_))));
    }

    #[tokio::test]
    async fn test_send_to_unhealthy_peer() {
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let transport = ReplicationTransport::new_with_channels(make_region("us-east"), tx, rx);
        let peer = ReplicationPeer::new(make_region("eu-west"), "https://eu-west:9090".to_string());
        peer.mark_unhealthy().await;
        transport.register_peer(peer).await;
        let result = transport
            .send_to_peer(&make_region("eu-west"), ReplicationPayload::Heartbeat)
            .await;
        assert!(matches!(result, Err(ReplicationError::PeerUnhealthy(_))));
    }

    #[tokio::test]
    async fn test_send_and_receive() {
        let (out_tx, out_rx) = tokio::sync::mpsc::channel(64);
        let (in_tx, in_rx) = tokio::sync::mpsc::channel(64);

        let sender = ReplicationTransport::new_with_channels(make_region("us-east"), out_tx, in_rx);
        let receiver =
            ReplicationTransport::new_with_channels(make_region("eu-west"), in_tx, out_rx);

        sender
            .register_peer(ReplicationPeer::new(
                make_region("eu-west"),
                "http://localhost:9090".to_string(),
            ))
            .await;

        sender
            .send_to_peer(&make_region("eu-west"), ReplicationPayload::Heartbeat)
            .await
            .unwrap();

        let msg = receiver.receive().await.unwrap();
        assert_eq!(msg.source_region, make_region("us-east"));
        assert_eq!(msg.target_region, make_region("eu-west"));
        assert!(matches!(msg.payload, ReplicationPayload::Heartbeat));
        assert!(!msg.checksum.is_empty());
    }

    #[tokio::test]
    async fn test_send_sequence_increments() {
        let (out_tx, _out_rx) = tokio::sync::mpsc::channel(64);
        let (in_tx, in_rx) = tokio::sync::mpsc::channel(64);

        let sender = ReplicationTransport::new_with_channels(make_region("us-east"), out_tx, in_rx);
        let _receiver =
            ReplicationTransport::new_with_channels(make_region("eu-west"), in_tx, _out_rx);

        sender
            .register_peer(ReplicationPeer::new(
                make_region("eu-west"),
                "http://localhost:9090".to_string(),
            ))
            .await;

        let before = sender.next_sequence().await;
        sender
            .send_to_peer(&make_region("eu-west"), ReplicationPayload::Heartbeat)
            .await
            .unwrap();
        let after = sender.next_sequence().await;
        assert_eq!(after - before, 2);
    }

    #[tokio::test]
    async fn test_healthy_peers() {
        let transport = ReplicationTransport::new(make_region("us-east"), 64);
        let peer1 = ReplicationPeer::new(make_region("eu-west"), "http://eu:9090".to_string());
        let peer2 = ReplicationPeer::new(make_region("ap-south"), "http://ap:9090".to_string());
        peer2.mark_unhealthy().await;
        transport.register_peer(peer1).await;
        transport.register_peer(peer2).await;
        let healthy = transport.healthy_peers().await;
        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy[0], make_region("eu-west"));
    }

    #[tokio::test]
    async fn test_peer_health_transitions() {
        let peer = ReplicationPeer::new(make_region("eu-west"), "http://eu:9090".to_string());
        assert!(peer.is_healthy().await);
        peer.mark_unhealthy().await;
        assert!(!peer.is_healthy().await);
        peer.mark_healthy().await;
        assert!(peer.is_healthy().await);
    }

    #[tokio::test]
    async fn test_peer_last_seen() {
        let peer = ReplicationPeer::new(make_region("eu-west"), "http://eu:9090".to_string());
        assert!(peer.last_seen().await.is_none());
        let now = Utc::now();
        peer.update_last_seen(now).await;
        assert!(peer.last_seen().await.is_some());
    }

    #[tokio::test]
    async fn test_heartbeat_loop_sends_heartbeats() {
        let (out_tx, mut out_rx) = tokio::sync::mpsc::channel(64);
        let (in_tx, in_rx) = tokio::sync::mpsc::channel(64);

        let sender = ReplicationTransport::new_with_channels(make_region("us-east"), out_tx, in_rx);
        let _receiver = ReplicationTransport::new_with_channels(
            make_region("eu-west"),
            in_tx,
            tokio::sync::mpsc::channel(1).1,
        );

        sender
            .register_peer(ReplicationPeer::new(
                make_region("eu-west"),
                "http://localhost:9090".to_string(),
            ))
            .await;

        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);

        let handle = tokio::spawn({
            let sender = Arc::new(sender);
            async move {
                sender
                    .heartbeat_loop(std::time::Duration::from_millis(50), cancel_rx)
                    .await;
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        cancel_tx.send(true).unwrap();

        let mut msgs = Vec::new();
        while let Ok(msg) = out_rx.try_recv() {
            msgs.push(msg);
        }
        handle.await.unwrap();
        assert!(!msgs.is_empty());
        assert!(
            msgs.iter()
                .all(|m| matches!(m.payload, ReplicationPayload::Heartbeat))
        );
    }

    #[tokio::test]
    async fn test_heartbeat_loop_stops_on_cancel() {
        let (out_tx, _out_rx) = tokio::sync::mpsc::channel(64);
        let (in_tx, in_rx) = tokio::sync::mpsc::channel(64);
        let sender = ReplicationTransport::new_with_channels(make_region("us-east"), out_tx, in_rx);
        let _receiver =
            ReplicationTransport::new_with_channels(make_region("eu-west"), in_tx, _out_rx);

        sender
            .register_peer(ReplicationPeer::new(
                make_region("eu-west"),
                "http://localhost:9090".to_string(),
            ))
            .await;

        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
        cancel_tx.send(true).unwrap();

        let sender = Arc::new(sender);
        sender
            .heartbeat_loop(std::time::Duration::from_secs(10), cancel_rx)
            .await;

        assert_eq!(sender.next_sequence().await, 1);
    }

    #[tokio::test]
    async fn test_health_monitor_marks_unhealthy_peers() {
        let transport = ReplicationTransport::new(make_region("us-east"), 64);
        let peer = ReplicationPeer::new(make_region("eu-west"), "http://eu:9090".to_string());
        peer.update_last_seen(Utc::now() - chrono::Duration::seconds(30))
            .await;
        transport.register_peer(peer).await;

        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
        let peers_ref = transport.peers.clone();
        let handle = tokio::spawn(async move {
            let transport = ReplicationTransport {
                region_id: make_region("us-east"),
                peers: peers_ref,
                outbound_tx: tokio::sync::mpsc::channel(1).0,
                inbound_rx: Arc::new(RwLock::new(None)),
                sequence: Arc::new(RwLock::new(0)),
            };
            transport
                .health_monitor(std::time::Duration::from_millis(100), cancel_rx)
                .await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        cancel_tx.send(true).unwrap();
        handle.await.unwrap();

        let peers = transport.peers.read().await;
        assert!(!peers[0].is_healthy().await);
    }
}
