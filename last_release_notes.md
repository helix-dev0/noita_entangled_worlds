## Noita Entangled Worlds v1.8.2 (helix-dev0 fork)

Fixes a co-op desync where enemies killed by the host could linger as inert "ghosts" on the other player's screen.

### Changes since v1.8.1

- **No more ghost enemies.** When the host killed an enemy, a client that had been silently dropped from the host's "interested" set (e.g. on a world/biome transition) never received the removal — it kept a frozen, inert copy of the dead enemy that the host couldn't see. The death broadcast now carries the entity id and removes the client's stale copy. **Wire change — both players must update to v1.8.2** (older versions won't connect).

## Installation

Download and unpack `noita_proxy-win.zip` or `noita_proxy-linux.zip` for your OS, then launch the proxy. The mod auto-installs from this fork — no need to grab `quant.ew.zip` manually.

## Updating

The button in the bottom-left of the proxy's main screen auto-updates to a newer version from this fork when one is available.
