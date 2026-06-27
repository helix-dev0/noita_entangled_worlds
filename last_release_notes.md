## Noita Entangled Worlds v1.8.1 (helix-dev0 fork)

Adds co-op compatibility for the **Persistence** mod so each player keeps their own profile.

### Changes since v1.8.0

- **Persistence mod profiles are now per-player.** With the [Persistence](https://steamcommunity.com/sharedfiles/filedetails/?id=3253132683) mod, a player joining a co-op lobby had their own saved profile (stashed money, researched spells, wands) replaced by the host's — or wiped to empty — and could overwrite their real save on death. EW's flag-sync no longer routes Persistence's `persistence_`-prefixed data through the host, so each player reads and writes their **own** profile independently. Normal co-op progression sharing (spell/perk unlocks, orbs) is unchanged. Mod-only change — fully interoperable with v1.8.0.

## Installation

Download and unpack `noita_proxy-win.zip` or `noita_proxy-linux.zip` for your OS, then launch the proxy. The mod auto-installs from this fork — no need to grab `quant.ew.zip` manually.

## Updating

The button in the bottom-left of the proxy's main screen auto-updates to a newer version from this fork when one is available.
