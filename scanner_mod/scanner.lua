-- Exports all relevant entities and tiles positions into JSON. Consumer can reconstruct the Factorio map
--
-- Uses compressed one-big-array format: [name1, x1, y1, name2, ...]
-- Scales to
-- this avoids significant JSON overhead from [{name: "coal", position: { x: -5.5, y: -5.5 }, ...]
--
-- Splits area into 4 sectors going to separate files.
-- On extremely large 2000x2000 chunk plus maps,
-- Factorio game.table_to_json() silently(!) won't write more than 4.2gb (u32::MAX) of JSON. Though JSON is valid.
--
-- Local everything and wrapper function to hopefully make the Lua GC happy
--
-- /c
local function megacall()
    local size = 32000
    local sectors = {
        { { -size, -size }, { 0, 0 } },
        { { 0, 0 }, { size, size } },
        { { -size, 0 }, { 0, size } },
        { { 0, -size }, { size, 0 } },
    }
    for i, sector in ipairs(sectors) do
        local file = "big-tiles" .. i .. ".json"
        log("write " .. file .. "...")
        local output = {}
        local all_tiles = game.player.surface.find_tiles_filtered {
            area = sector,
            name = { "water" }
        }
        for _, entity in ipairs(all_tiles) do
            table.insert(output, entity.name)
            table.insert(output, entity.position.x)
            table.insert(output, entity.position.y)
        end
        game.write_file(file, game.table_to_json(output))
    end
end
megacall()

-- /c
local function megacall()
    local size = 32000
    local sectors = {
        { { -size, -size }, { 0, 0 } },
    }
    for i, sector in ipairs(sectors) do
        local file = "big-tiles" .. i .. ".json"
        log("write " .. file .. "...")
        local output = {}
        local all_tiles = game.player.surface.find_tiles_filtered {
            area = sector,
            name = { "water" }
        }
        for j, entity in ipairs(all_tiles) do
            local out_entry = {
                name = entity.name,
                pos = entity.position,
            }
            table.insert(output, out_entry)
        end
    end

    game.write_file(file, game.table_to_json(output))
end
megacall()

-- /c
local function inner()
    local file = "big-entities-a.json"
    log("write " .. file .. "...")
    local output = {}
    local all_entities = game.player.surface.find_entities_filtered {
        area = { { -32000, -32000 }, { 32000, 32000 } },
        name = { "iron-ore", "copper-ore", "stone", "coal", "uranium-ore", "crude-oil" }
    }
    for _, entity in ipairs(all_entities) do
        table.insert(output, entity.name)
        table.insert(output, entity.position.x)
        table.insert(output, entity.position.y)
    end
    game.write_file(file, game.table_to_json(output))
end
inner()








--output.prototypes = {}
--for k, entity in pairs(game.entity_prototypes) do
--    local prefix = entity.type .. "," .. entity.name
--
--    table.insert(
--            output.prototypes,
--            {
--                type = entity.type,
--                name = entity.name,
--                collision_box = entity.collision_box,
--                tile_width = entity.tile_width,
--                tile_height = entity.tile_height,
--                secondary_collision_box = entity.secondary_collision_box,
--                map_generator_bounding_box = entity.map_generator_bounding_box,
--                selection_box = entity.selection_box,
--                drawing_box = entity.drawing_box,
--                collision_mask = entity.collision_mask,
--            }
--    )
--end
