## Noita Entangled Worlds v1.6.7 (helix-dev0 fork)

Makes the *other* players move smoothly instead of stuttering. Self-contained (mod + updates come from this fork). **This is a render-only change with no wire-format change, so a v1.6.7 player and a v1.6.6/v1.6.5/v1.6.4 player can still play together** — but both players want v1.6.7 to see each other move smoothly, because the smoothing happens on the side that's *watching*.

### Changes since v1.6.6
- **Smooth remote-player movement** — other players' characters are now interpolated between the position updates that arrive over the network, instead of being snapped to each one. Running, jumping, and flying look smooth at your local framerate instead of stuttering with network jitter. Teleports still snap instantly, and the puppet briefly extrapolates from its last-known velocity if an update is late. Toggle it under the mod's settings → **"smooth others movement"** (on by default) if you prefer the old behavior.

This is on top of v1.6.6's host-side world-sync improvements (delta-once, broadcast-when-all-listen), v1.6.5's Steam NoNagle + bounded world cuts, and v1.6.4's encode-once fan-out, quinn BBR tuning, TCP_NODELAY, message-length bounding, network instrumentation (`NP_NET_STATS=1`), and build-profile tuning.

## Installation

Download and unpack `noita_proxy-win.zip` or `noita_proxy-linux.zip` for your OS, then launch the proxy. The mod auto-installs from this fork — no need to grab `quant.ew.zip` manually.

## Updating

The button in the bottom-left of the proxy's main screen auto-updates to a newer version from this fork when one is available.
