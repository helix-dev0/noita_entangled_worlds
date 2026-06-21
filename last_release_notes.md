## Noita Entangled Worlds v1.6.9 (helix-dev0 fork)

Fixes a slow performance creep where the host — and clients — got laggier the longer a run went on.

### Changes since v1.6.8
- **No more lag-over-time creep** — the proxy's world-sync layer kept several per-chunk bookkeeping maps (and the deferred-explosion buffers) growing for the entire session: entries were added as you explored but only ever freed on a world/dimension change, never when a chunk unloaded. Over a long run this steady memory growth degraded allocator/cache behaviour and made the game feel progressively laggier — for the host **and** every client. This release frees that per-chunk state the moment a chunk unloads (and when a peer disconnects), clears the buffers a world change had been leaking, and reclaims the explosion buffers once their pending work drains. **Proxy-only, with no wire-format change** — fully interoperable with v1.6.8 / v1.6.7, and it does not affect terrain, sync correctness, or bandwidth.

This is on top of v1.6.8's adaptive remote-player smoothing, v1.6.6's host-side world-sync improvements, v1.6.5's Steam NoNagle + bounded world cuts, and v1.6.4's encode-once fan-out, quinn BBR tuning, TCP_NODELAY, message-length bounding, network instrumentation (`NP_NET_STATS=1`), and build-profile tuning.

## Installation

Download and unpack `noita_proxy-win.zip` or `noita_proxy-linux.zip` for your OS, then launch the proxy. The mod auto-installs from this fork — no need to grab `quant.ew.zip` manually.

## Updating

The button in the bottom-left of the proxy's main screen auto-updates to a newer version from this fork when one is available.
