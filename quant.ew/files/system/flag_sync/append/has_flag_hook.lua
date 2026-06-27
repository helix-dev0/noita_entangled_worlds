local old = HasFlagPersistent
local old_add_flag = AddFlagPersistent

-- Flags owned by third-party mods that keep their OWN per-player progression in
-- Noita's persistent flags must NOT ride EW's synced flag path: a joining client
-- would otherwise read the host's (or an empty) profile and clobber its own on
-- write-back. Route these to the original local-disk API so each player keeps an
-- independent profile. The Persistence mod (Steam Workshop 3253132683) stores its
-- entire profile under flags prefixed "persistence_".
local PRIVATE_FLAG_PREFIXES = { "persistence_" }

local function is_private_flag(flag)
    if type(flag) ~= "string" then
        return false
    end
    for _, prefix in ipairs(PRIVATE_FLAG_PREFIXES) do
        if flag:sub(1, #prefix) == prefix then
            return true
        end
    end
    return false
end

function HasFlagPersistent(flag)
    -- Per-player mods: read this player's own local persistent flag, never the
    -- host-synced set.
    if is_private_flag(flag) then
        return old(flag)
    end
    if EwHasPersistentFlag ~= nil then
        return EwHasPersistentFlag(flag)
    end
    if CrossCall ~= nil then
        return CrossCall("ew_has_flag", flag)
    end
    print("the flag, " .. flag .. " is not being called in a synced way")
    return old(flag)
end

function AddFlagPersistent(flag)
    -- Per-player mods: write only to this player's own local disk; skip the
    -- "ew_pf_" run-flag that feeds the synced read above.
    if is_private_flag(flag) then
        return old_add_flag(flag)
    end
    GameAddFlagRun("ew_pf_" .. flag)
    return old_add_flag(flag)
end

-- RemoveFlagPersistent is deliberately NOT hooked: it stays on the native
-- (local-disk) path. The carve-out above depends on the whole read/add/remove
-- cycle for private ("persistence_") flags being local -- routing remove through
-- the synced path would silently reintroduce cross-player profile corruption.
