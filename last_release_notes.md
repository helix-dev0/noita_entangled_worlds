## Noita Entangled Worlds v1.6.5 (helix-dev0 fork)

A small follow-up to v1.6.4 with two more multiplayer improvements. Self-contained (mod + updates come from this fork). **Both changes are local-only with no wire-format change, so a v1.6.5 player and a v1.6.4 player can still play together.**

### Changes since v1.6.4
- **Steam: NoNagle on latency-sensitive traffic** — positions / camera / voice are sent immediately instead of waiting to coalesce, for snappier movement on the Steam path. Reliable bulk keeps its batching.
- **Host CPU: faster world cuts** — explosions / dig-throughs no longer clone and scan the *entire* loaded world; only the affected region is processed. Less host-side stutter in heavy scenes.

This is on top of v1.6.4's encode-once fan-out, quinn BBR tuning, TCP_NODELAY, message-length bounding, network instrumentation (`NP_NET_STATS=1`), and build-profile tuning.

## Installation

Download and unpack `noita_proxy-win.zip` or `noita_proxy-linux.zip` for your OS, then launch the proxy. The mod auto-installs from this fork — no need to grab `quant.ew.zip` manually.

## Updating

The button in the bottom-left of the proxy's main screen auto-updates to a newer version from this fork when one is available.
