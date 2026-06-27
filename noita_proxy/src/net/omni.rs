use super::steam_networking::{self, ExtraPeerState};
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use steamworks::{LobbyId, SteamError, SteamId};
use tangled::{PeerId, Reliability};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Decode, Encode)]
pub struct OmniPeerId(pub u64);

impl From<shared::PeerId> for OmniPeerId {
    fn from(value: shared::PeerId) -> Self {
        OmniPeerId(value.0)
    }
}

impl From<OmniPeerId> for shared::PeerId {
    fn from(value: OmniPeerId) -> Self {
        shared::PeerId(value.0)
    }
}

impl From<PeerId> for OmniPeerId {
    fn from(value: PeerId) -> Self {
        Self(value.0.into())
    }
}

impl From<SteamId> for OmniPeerId {
    fn from(value: SteamId) -> Self {
        Self(value.raw())
    }
}

impl From<OmniPeerId> for PeerId {
    fn from(value: OmniPeerId) -> Self {
        Self(
            value
                .0
                .try_into()
                .expect("Assuming PeerId was stored here, so conversion should succeed"),
        )
    }
}

impl From<OmniPeerId> for SteamId {
    fn from(value: OmniPeerId) -> Self {
        Self::from_raw(value.0)
    }
}

impl Display for OmniPeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl OmniPeerId {
    pub fn from_hex(val: &str) -> Option<Self> {
        let raw = u64::from_str_radix(val, 16).ok()?;
        Some(Self(raw))
    }

    pub(crate) fn as_hex(&self) -> String {
        format!("{:016x}", self.0)
    }
}

pub enum OmniNetworkEvent {
    PeerConnected(OmniPeerId),
    PeerDisconnected(OmniPeerId),
    Message { src: OmniPeerId, data: Vec<u8> },
}

impl From<tangled::NetworkEvent> for OmniNetworkEvent {
    fn from(value: tangled::NetworkEvent) -> Self {
        match value {
            tangled::NetworkEvent::PeerConnected(id) => Self::PeerConnected(id.into()),
            tangled::NetworkEvent::PeerDisconnected(id) => Self::PeerDisconnected(id.into()),
            tangled::NetworkEvent::Message(msg) => Self::Message {
                src: msg.src.into(),
                data: msg.data,
            },
        }
    }
}

#[allow(clippy::large_enum_variant)]
pub enum PeerVariant {
    Tangled(tangled::Peer),
    Steam(steam_networking::SteamPeer),
}

/// Transport-level send outcome for the omni layer.
///
/// We can't add a variant to `tangled::NetError` (it lives in the vendored
/// `tangled` crate, outside this batch's touch set), so the omni layer wraps it
/// and adds the one distinction `net.rs` needs for backpressure handling
/// (issue #19): a *full reliable send queue*. A reliable message that hits this
/// must be buffered + retried in order, never silently dropped.
#[derive(Debug)]
pub(crate) enum NetSendError {
    /// The per-connection reliable send queue is full
    /// (Steam `k_EResultLimitExceeded`). The message was NOT handed to the
    /// transport.
    QueueFull,
    /// Any other transport error; carries the underlying `tangled::NetError`.
    Other(tangled::NetError),
}

impl Display for NetSendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetSendError::QueueFull => write!(f, "reliable send queue full"),
            NetSendError::Other(e) => write!(f, "{e}"),
        }
    }
}

/// Map a Steam send error to our omni error. `LimitExceeded` (a full send
/// buffer) becomes [`NetSendError::QueueFull`] so the caller can buffer + retry
/// instead of dropping a reliable message; connection-loss errors stay
/// `Disconnected`.
fn map_steam_err(e: SteamError) -> NetSendError {
    match e {
        SteamError::LimitExceeded => NetSendError::QueueFull,
        SteamError::InvalidSteamID => NetSendError::Other(tangled::NetError::UnknownPeer),
        SteamError::Ignored => NetSendError::Other(tangled::NetError::Dropped),
        SteamError::InvalidParameter => NetSendError::Other(tangled::NetError::MessageTooLong),
        SteamError::NoConnection | SteamError::InvalidState => {
            NetSendError::Other(tangled::NetError::Disconnected)
        }
        _ => NetSendError::Other(tangled::NetError::Other),
    }
}

impl PeerVariant {
    /// Send already-encoded bytes without re-encoding. For fan-out to many
    /// peers: encode once, then reuse the buffer here (tangled copies it into a
    /// Vec, Steam sends the slice directly).
    pub(crate) fn send_encoded(
        &self,
        peer: OmniPeerId,
        bytes: &[u8],
        reliability: Reliability,
    ) -> Result<(), NetSendError> {
        match self {
            PeerVariant::Tangled(p) => p
                .send(peer.into(), bytes.to_vec(), reliability)
                .map_err(NetSendError::Other),
            PeerVariant::Steam(p) => p
                .send_message(peer.into(), bytes, reliability)
                .map_err(map_steam_err),
        }
    }

    pub(crate) fn flush(&self) {
        if let PeerVariant::Steam(p) = self {
            p.flush()
        }
    }

    /// Force-drop our connection to a single `peer`, surfacing it through the
    /// normal peer-disconnect path. Used as the explicit escalation when a
    /// peer's reliable retry buffer overflows (issue #19): a clean, observable
    /// disconnect beats unbounded buffering or silently corrupting sync state.
    pub(crate) fn disconnect_peer(&self, peer: OmniPeerId) {
        match self {
            PeerVariant::Tangled(p) => p.remove(peer.into()),
            PeerVariant::Steam(p) => p.disconnect_peer(peer),
        }
    }

    pub(crate) fn my_id(&self) -> OmniPeerId {
        match self {
            PeerVariant::Tangled(p) => p
                .my_id()
                .map(OmniPeerId::from)
                .expect("Peer id to be available"),
            PeerVariant::Steam(p) => p.my_id().into(),
        }
    }

    pub fn iter_peer_ids(&self) -> Vec<OmniPeerId> {
        match self {
            PeerVariant::Tangled(p) => p.iter_peer_ids().map(OmniPeerId::from).collect(),
            PeerVariant::Steam(p) => p.get_peer_ids().into_iter().map(OmniPeerId::from).collect(),
        }
    }

    /// Count connected peers whose id is not `exclude`, without collecting the
    /// peer ids into a `Vec`. Equivalent to
    /// `self.iter_peer_ids().into_iter().filter(|p| *p != exclude).count()` but
    /// avoids the intermediate allocation on the per-frame hot path. The Tangled
    /// backend iterates lazily; the Steam backend still briefly clones its peer
    /// set inside `get_peer_ids`, exactly as `iter_peer_ids` does.
    pub fn count_peer_ids_excluding(&self, exclude: OmniPeerId) -> usize {
        match self {
            PeerVariant::Tangled(p) => p
                .iter_peer_ids()
                .filter(|id| OmniPeerId::from(*id) != exclude)
                .count(),
            PeerVariant::Steam(p) => p
                .get_peer_ids()
                .into_iter()
                .filter(|id| OmniPeerId::from(*id) != exclude)
                .count(),
        }
    }

    pub(crate) fn recv(&self) -> Vec<OmniNetworkEvent> {
        match self {
            PeerVariant::Tangled(p) => p.recv().map(OmniNetworkEvent::from).collect(),
            PeerVariant::Steam(p) => p.recv(),
        }
    }

    pub fn state(&self) -> ExtraPeerState {
        match self {
            PeerVariant::Tangled(p) => ExtraPeerState::Tangled(p.state()),
            PeerVariant::Steam(p) => p.state(),
        }
    }

    pub fn host_id(&self) -> OmniPeerId {
        match self {
            PeerVariant::Tangled(_) => PeerId::HOST.into(),
            PeerVariant::Steam(p) => p.host_id().into(),
        }
    }

    pub fn lobby_id(&self) -> Option<LobbyId> {
        match self {
            PeerVariant::Tangled(_) => None,
            PeerVariant::Steam(p) => p.lobby_id(),
        }
    }

    pub fn is_steam(&self) -> bool {
        matches!(self, PeerVariant::Steam(_))
    }

    pub fn is_host(&self) -> bool {
        match self {
            PeerVariant::Tangled(_) => self.host_id() == self.my_id(),
            PeerVariant::Steam(p) => p.is_host(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NetSendError, map_steam_err};
    use steamworks::SteamError;

    #[test]
    fn steam_limit_exceeded_maps_to_queue_full() {
        // The whole point of issue #19: a full reliable send buffer must be
        // distinguishable so net.rs can retry instead of silently dropping.
        assert!(matches!(
            map_steam_err(SteamError::LimitExceeded),
            NetSendError::QueueFull
        ));
    }

    #[test]
    fn steam_connection_loss_maps_to_disconnected() {
        assert!(matches!(
            map_steam_err(SteamError::NoConnection),
            NetSendError::Other(tangled::NetError::Disconnected)
        ));
        assert!(matches!(
            map_steam_err(SteamError::InvalidState),
            NetSendError::Other(tangled::NetError::Disconnected)
        ));
    }

    #[test]
    fn steam_other_errors_are_not_queue_full() {
        // Anything that isn't a full queue must NOT be treated as QueueFull,
        // or we'd buffer messages that can never be delivered.
        for e in [
            SteamError::InvalidSteamID,
            SteamError::Ignored,
            SteamError::InvalidParameter,
        ] {
            assert!(matches!(map_steam_err(e), NetSendError::Other(_)));
        }
    }
}
