## Noita Entangled Worlds v1.8.0 (helix-dev0 fork)

A netcode reliability + performance pass: fixes a busy-scene desync, entity-sync skips, a late-game crash, and trims host CPU. **No network-protocol change** — interoperable with v1.7.0 — but it updates the bundled mod + ewext, so both players should be on v1.8.0.

### Highlights since v1.7.0

- **Busy scenes stay in sync.** Under heavy load (big fights, explosions, lots of digging) the proxy used to silently drop reliable sync messages whenever Steam's send queue filled up — things would desync until they happened to re-sync. Reliable messages are now buffered and retried in order instead of dropped; only a genuinely stuck connection escalates to a clean disconnect. (#19)
- **Entities no longer skip or duplicate over time.** The entity position-sync batcher could skip or double-send entities — and never synced any entity past a certain count at all. Now every tracked entity is synced exactly once per cycle.
- **Fixes a late-game crash.** A memory-safety bug in ewext (`to_integer_array`) that could corrupt LuaJIT's stack on explosion / level-load frames is fixed. (#16)
- **Lower host CPU, smoother world sync.** Pixel/world sync now uses a bitset + sparse run-length encoding (skips unchanged regions), bulk pixel fills, reused per-frame buffers, and O(1) entity lookups. Voice chat moved to an unreliable channel (lower latency, less queue pressure). Several previously-fatal error paths now degrade gracefully instead of crashing.

⚠️ These netcode changes are covered by tests + review but are best proven in real play — please report any sync oddities.

## Installation

Download and unpack `noita_proxy-win.zip` or `noita_proxy-linux.zip` for your OS, then launch the proxy. The mod auto-installs from this fork — no need to grab `quant.ew.zip` manually.

## Updating

The button in the bottom-left of the proxy's main screen auto-updates to a newer version from this fork when one is available.
