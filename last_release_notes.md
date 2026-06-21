## Noita Entangled Worlds v1.6.8 (helix-dev0 fork)

Fixes the remote-player movement smoothing from v1.6.7, which didn't actually take effect.

### Changes since v1.6.7
- **Smooth remote-player movement (now actually works)** — v1.6.7 added receiver-side interpolation of other players' movement, but its render delay (60 ms) was *smaller* than the gap between incoming position updates, so it silently fell back to snapping — no visible smoothing. v1.6.8 makes the render delay **adaptive to the real update rate**, so the puppet is interpolated *between* the updates that bracket it, as intended. Running, jumping, and flying should now look smooth at your local framerate instead of stuttering. Still **render-only with no wire-format change** — fully interoperable with v1.6.7 / v1.6.6, and both players want v1.6.8 to see each other smoothed. Toggle under the mod's settings → **"smooth others movement"** (on by default).

This is on top of v1.6.7's smoothing groundwork, v1.6.6's host-side world-sync improvements, v1.6.5's Steam NoNagle + bounded world cuts, and v1.6.4's encode-once fan-out, quinn BBR tuning, TCP_NODELAY, message-length bounding, network instrumentation (`NP_NET_STATS=1`), and build-profile tuning.

## Installation

Download and unpack `noita_proxy-win.zip` or `noita_proxy-linux.zip` for your OS, then launch the proxy. The mod auto-installs from this fork — no need to grab `quant.ew.zip` manually.

## Updating

The button in the bottom-left of the proxy's main screen auto-updates to a newer version from this fork when one is available.
