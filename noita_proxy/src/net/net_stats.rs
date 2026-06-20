//! Env-gated (`NP_NET_STATS`) per-message-type traffic counters.
//!
//! Zero overhead when disabled (a single bool check per send/recv). When
//! enabled, a background thread logs per-`NetMsg`-kind counts and byte volumes
//! (wire vs raw) every few seconds and resets. This is Phase 0 of
//! MULTIPLAYER_AUDIT_AND_PLAN.md: it makes the encode-once (Phase 2) and
//! compression (Phase 6) work measurable instead of asserted.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rustc_hash::FxHashMap;
use tracing::info;

use super::messages::NetMsg;

const LOG_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Default)]
struct MsgCounter {
    count: u64,
    raw_bytes: u64,
    wire_bytes: u64,
}

#[derive(Default)]
struct Counters {
    outbound: FxHashMap<&'static str, MsgCounter>,
    inbound: FxHashMap<&'static str, MsgCounter>,
}

pub(crate) struct NetStats {
    enabled: bool,
    counters: Arc<Mutex<Counters>>,
}

impl NetStats {
    pub(crate) fn new() -> Self {
        let enabled = std::env::var_os("NP_NET_STATS").is_some();
        let counters = Arc::new(Mutex::new(Counters::default()));
        if enabled {
            info!("NP_NET_STATS enabled: logging per-message-type traffic every 5s");
            let counters = Arc::clone(&counters);
            thread::spawn(move || {
                loop {
                    thread::sleep(LOG_INTERVAL);
                    log_and_clear(&mut counters.lock().unwrap());
                }
            });
        }
        Self { enabled, counters }
    }

    /// Record a message actually sent over the network (skip the loopback path).
    /// `raw_bytes` is the bitcode size, `wire_bytes` is after compression.
    pub(crate) fn record_outbound(&self, msg: &NetMsg, raw_bytes: usize, wire_bytes: usize) {
        self.record(true, msg, raw_bytes, wire_bytes);
    }

    /// Record a message received from a peer. `raw_bytes` is the decompressed
    /// size, `wire_bytes` is the bytes that arrived.
    pub(crate) fn record_inbound(&self, msg: &NetMsg, raw_bytes: usize, wire_bytes: usize) {
        self.record(false, msg, raw_bytes, wire_bytes);
    }

    fn record(&self, outbound: bool, msg: &NetMsg, raw_bytes: usize, wire_bytes: usize) {
        if !self.enabled {
            return;
        }
        let mut c = self.counters.lock().unwrap();
        let map = if outbound {
            &mut c.outbound
        } else {
            &mut c.inbound
        };
        let entry = map.entry(netmsg_kind(msg)).or_default();
        entry.count += 1;
        entry.raw_bytes += raw_bytes as u64;
        entry.wire_bytes += wire_bytes as u64;
    }
}

fn log_and_clear(c: &mut Counters) {
    log_dir("out", &c.outbound);
    log_dir(" in", &c.inbound);
    c.outbound.clear();
    c.inbound.clear();
}

fn log_dir(dir: &str, map: &FxHashMap<&'static str, MsgCounter>) {
    let mut rows: Vec<_> = map.iter().collect();
    rows.sort_unstable_by_key(|(_, c)| std::cmp::Reverse(c.wire_bytes));
    for (kind, c) in rows {
        let ratio = if c.raw_bytes > 0 {
            c.wire_bytes as f64 / c.raw_bytes as f64
        } else {
            1.0
        };
        info!(
            "net-stats {dir} {kind:>18}: {:>5} msgs {:>9} B wire {:>9} B raw ({ratio:.2}x)",
            c.count, c.wire_bytes, c.raw_bytes
        );
    }
}

/// Stable label per `NetMsg` kind. Exhaustive on purpose (no `_` arm) so adding
/// a variant is a compile error here until it gets a label.
fn netmsg_kind(msg: &NetMsg) -> &'static str {
    match msg {
        NetMsg::Welcome => "Welcome",
        NetMsg::RequestMods => "RequestMods",
        NetMsg::Mods { .. } => "Mods",
        NetMsg::EndRun => "EndRun",
        NetMsg::Kick => "Kick",
        NetMsg::PeerDisconnected { .. } => "PeerDisconnected",
        NetMsg::StartGame { .. } => "StartGame",
        NetMsg::ModRaw { .. } => "ModRaw",
        NetMsg::ModCompressed { .. } => "ModCompressed",
        NetMsg::WorldMessage(..) => "WorldMessage",
        NetMsg::PlayerColor(..) => "PlayerColor",
        NetMsg::RemoteMsg(..) => "RemoteMsg",
        NetMsg::ForwardDesToProxy(..) => "ForwardDesToProxy",
        NetMsg::ForwardProxyToDes(..) => "ForwardProxyToDes",
        NetMsg::NoitaDisconnected => "NoitaDisconnected",
        NetMsg::Flags(..) => "Flags",
        NetMsg::RespondFlagNormal(..) => "RespondFlagNormal",
        NetMsg::RespondFlagSlow(..) => "RespondFlagSlow",
        NetMsg::RespondFlagMoon(..) => "RespondFlagMoon",
        NetMsg::PlayerPosition(..) => "PlayerPosition",
        NetMsg::RespondFlagStevari(..) => "RespondFlagStevari",
        NetMsg::AudioData(..) => "AudioData",
        NetMsg::MapData(..) => "MapData",
        NetMsg::MatData(..) => "MatData",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_labels_match_variants() {
        assert_eq!(netmsg_kind(&NetMsg::Welcome), "Welcome");
        assert_eq!(netmsg_kind(&NetMsg::MapData(Default::default())), "MapData");
        assert_eq!(
            netmsg_kind(&NetMsg::PlayerPosition(0, 0, false, false)),
            "PlayerPosition"
        );
    }

    #[test]
    fn disabled_stats_are_a_noop() {
        let stats = NetStats {
            enabled: false,
            counters: Arc::new(Mutex::new(Counters::default())),
        };
        stats.record_outbound(&NetMsg::Welcome, 10, 5);
        assert!(stats.counters.lock().unwrap().outbound.is_empty());
    }
}
