## Noita Entangled Worlds v1.6.6 (helix-dev0 fork)

A small follow-up to v1.6.5 with two more host-side world-sync improvements. Self-contained (mod + updates come from this fork). **Both changes are local-only with no wire-format change, so a v1.6.6 player and a v1.6.5/v1.6.4 player can still play together.**

### Changes since v1.6.5
- **Host CPU: world deltas computed once per frame** — when sending pixel updates, the host no longer scans each changed chunk twice to build its delta; it reuses the one it already computed. About halves that per-frame work on the host (measured −52% on the affected path). Helps at any player count.
- **Bandwidth/CPU at 3+ players: broadcast shared world updates** — when every connected player needs the same chunk update, it's now encoded once and broadcast instead of re-encoded per player. No effect with 2 players (it falls back to the existing per-player path); a win only once a third player joins.

This is on top of v1.6.5's Steam NoNagle + bounded world cuts, and v1.6.4's encode-once fan-out, quinn BBR tuning, TCP_NODELAY, message-length bounding, network instrumentation (`NP_NET_STATS=1`), and build-profile tuning.

## Installation

Download and unpack `noita_proxy-win.zip` or `noita_proxy-linux.zip` for your OS, then launch the proxy. The mod auto-installs from this fork — no need to grab `quant.ew.zip` manually.

## Updating

The button in the bottom-left of the proxy's main screen auto-updates to a newer version from this fork when one is available.
