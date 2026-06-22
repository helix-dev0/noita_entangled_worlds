## Noita Entangled Worlds v1.7.0 (helix-dev0 fork)

Fixes fire desync — fire one player starts now stays in sync for the other.

### Changes since v1.6.9
- **Fire stays in sync between players** — fire in Noita is simulated independently on each player's machine, and world sync only re-sent burning chunks on a slow rotating schedule (the chunk you're standing in every ~4 frames, farther ones up to every ~16), so the two simulations drifted apart and one player could see flames the other didn't ("you're standing in fire" when you weren't). Burning chunks are now detected and re-synced **every frame** while they're on fire, cutting the divergence window from ~60–250 ms down to ~16 ms, so both players see essentially the same fire as it spreads and dies. This is a bundled mod + ewext change with **no network-protocol change** — fully interoperable with v1.6.9 and older; both players want v1.7.0 for fire to match.

This is on top of v1.6.9's host/client lag-over-time fix, v1.6.8's adaptive remote-player smoothing, and the earlier world-sync, Steam NoNagle, and networking work.

## Installation

Download and unpack `noita_proxy-win.zip` or `noita_proxy-linux.zip` for your OS, then launch the proxy. The mod auto-installs from this fork — no need to grab `quant.ew.zip` manually.

## Updating

The button in the bottom-left of the proxy's main screen auto-updates to a newer version from this fork when one is available.
