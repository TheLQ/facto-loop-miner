-- Exports all relevant entities and tiles positions into JSON. Consumer can reconstruct the Factorio map
-- Local everything and wrapper function to hopefully make the Lua GC happy
--
-- Uses compressed one-big-array format: [name1, x1, y1, name2, ...]
-- Avoids significant JSON overhead from [{name: "coal", position: { x: -5.5, y: -5.5 }, ...]
--
-- Splits area into 4 sectors going to separate files.
-- On extremely large 2000x2000 chunk plus maps,
-- Factorio game.table_to_json() silently(!) won't write more than 4.2gb (u32::MAX) of JSON. Though JSON is valid.
--
-- /c
local function mega_export_tiles_compressed()
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
mega_export_tiles_compressed()

-- /c
local function mega_export_tiles_fat()
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
mega_export_tiles_fat()

-- /c
local function mega_export_entities_compressed()
    local chunks = 900 * 32
    local file = "big-entities-a.json"
    log("write " .. file .. "...")
    local output = {}
    local all_entities = game.surfaces[1].find_entities_filtered {
        area = { { -chunks, -chunks }, { chunks, chunks } },
        name = { "iron-ore", "copper-ore", "stone", "coal", "uranium-ore", "crude-oil", 'steel-chest' }
    }
    for _, entity in ipairs(all_entities) do
        table.insert(output, entity.name)
        table.insert(output, entity.position.x)
        table.insert(output, entity.position.y)
    end
    game.write_file(file, game.table_to_json(output))
end
mega_export_entities_compressed()

-- /c local test = game.surfaces[1].find_entity('steel-chest', {0.5,0.5}) if test == nill then log('nope') else log('exists') end
-- /c
local function insert_0x0_crate()
    game.surfaces[1].create_entity { name = "steel-chest", position = { 0.5, 0.5 } }
end
insert_0x0_crate()

-- /c
local function hyper_scan()
    local chunks = 150
    log('start game chunk generate with ' .. chunks .. " chunks...")
    game.surfaces[1].request_to_generate_chunks({ 0, 0 }, chunks)
    log('force_generate....')
    game.surfaces[1].force_generate_chunk_requests()
    log('exit function')
end
hyper_scan()

--log('chart_all...')
--game.forces[1].chart_all()

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
