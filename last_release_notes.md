## Noita Entangled Worlds v1.6.4 (helix-dev0 fork)

A fork build on top of upstream v1.6.3, focused on **multiplayer connection reliability and performance**. The Lua mod is functionally unchanged from upstream; the changes are in the proxy / networking layer.

This build is self-contained: it downloads its mod and checks for updates from this fork, not upstream.

### Networking / performance
- **Encode-once fan-out** — the proxy no longer re-encodes + recompresses a message once per recipient. Entity-sync messages are encoded once and the same bytes are sent to every peer (O(peers) → O(1) encode/compress on that path).
- **quinn transport tuning** — BBR congestion control (recovers far better than the default Cubic on lossy home/wireless links) plus the ACK-frequency extension, on both the direct-IP client and server.
- **TCP_NODELAY** on the game↔proxy localhost link (disables Nagle on the latency-critical path).
- **Network instrumentation** — set `NP_NET_STATS=1` to log per-message-type traffic (counts, wire vs raw bytes) every few seconds.
- **Build profiles** tuned for more representative local testing and a leaner shipped binary.

### Robustness / safety
- Bounded incoming message length on the direct-IP path — an oversized or malicious length is rejected before allocating, instead of risking OOM or a panic.
- Voice chat disables gracefully if the audio codec fails to initialize (previously a panic).
- Removed a process-wide working-directory change + unwrap from the game launcher.

## Installation

Download and unpack `noita_proxy-win.zip` or `noita_proxy-linux.zip` for your OS, then launch the proxy.

The proxy downloads and installs the mod automatically — there is no need to download `quant.ew.zip` manually.

You'll be prompted for the path to `noita.exe` on first launch; it's auto-detected for the Steam version when Steam is running.

## Updating

The button in the bottom-left of the proxy's main screen auto-updates to a newer version from this fork when one is available.
