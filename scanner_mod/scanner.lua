-- Dumps everything into
-- /c

local size = 25000
local output = {}

output.entities = {}
local all_entities = game.player.surface.find_entities({ { -size, -size }, { size, size } })
for _, entity in pairs(all_entities) do
    local out_entry = {
        type = entity.type,
        name = entity.name,
        pos = entity.position,
    }
    table.insert(output.entities, out_entry)
end
game.write_file("mega-dump.json", game.table_to_json(output))

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
