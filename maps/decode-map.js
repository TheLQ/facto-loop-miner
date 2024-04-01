// source: https://gist.github.com/Hornwitser/f291638024e7e3c0271b1f3a4723e05a
// with improvements to create factorio cli ready args for map gen

const zlib = require("zlib");
const util = require("util");
const fs = require("fs");
const path = require('node:path');
const process = require("process");

class Parser {
    constructor(buf) {
        this.pos = 0;
        this.buf = buf;
        this.last_position = {x: 0, y: 0};
    }
}

function read_bool(parser) {
    let value = read_uint8(parser) !== 0;
    return value;
}

function read_uint8(parser) {
    let value = parser.buf.readUInt8(parser.pos);
    parser.pos += 1;
    return value;
}

function read_int16(parser) {
    let value = parser.buf.readInt16LE(parser.pos);
    parser.pos += 2;
    return value;
}

function read_uint16(parser) {
    let value = parser.buf.readUInt16LE(parser.pos);
    parser.pos += 2;
    return value;
}

function read_int32(parser) {
    let value = parser.buf.readInt32LE(parser.pos);
    parser.pos += 4;
    return value;
}

function read_uint32(parser) {
    let value = parser.buf.readUInt32LE(parser.pos);
    parser.pos += 4;
    return value;
}

function read_uint32so(parser) {
    let value = read_uint8(parser);
    if (value === 0xff) {
        return read_uint32(parser);
    }

    return value;
}

function read_float(parser) {
    let value = parser.buf.readFloatLE(parser.pos);
    parser.pos += 4;
    return value;
}

function read_double(parser) {
    let value = parser.buf.readDoubleLE(parser.pos);
    parser.pos += 8;
    return value;
}

function read_string(parser) {
    let size = read_uint32so(parser);
    let data = parser.buf.slice(parser.pos, parser.pos + size).toString("utf-8");
    parser.pos += size;
    return data;
}

function read_optional(parser, read_value) {
    let load = read_uint8(parser) !== 0;
    if (!load) {
        return null;
    }
    return read_value(parser);
}

function read_array(parser, read_item) {
    let size = read_uint32so(parser);

    let array = [];
    for (let i = 0; i < size; i++) {
        let item = read_item(parser);
        array.push(item);
    }

    return array;
}

function read_dict(parser, read_key, read_value) {
    let size = read_uint32so(parser);

    let mapping = new Map();
    for (let i = 0; i < size; i++) {
        let key = read_key(parser);
        let value = read_value(parser);
        mapping.set(key, value);
    }

    return mapping;
}

function read_version(parser) {
    let major = read_uint16(parser);
    let minor = read_uint16(parser);
    let patch = read_uint16(parser);
    let developer = read_uint16(parser);
    return [major, minor, patch, developer];
}

function read_frequency_size_richness(parser) {
    return {
        frequency: read_float(parser),
        size: read_float(parser),
        richness: read_float(parser),
    }
}

function read_autoplace_setting(parser) {
    return {
        treat_missing_as_default: read_bool(parser),
        settings: map_to_object(read_dict(parser, read_string, read_frequency_size_richness)),
    };
}

function read_map_position(parser) {
    let x, y;
    let x_diff = read_int16(parser) / 256;
    if (x_diff === 0x7fff / 256) {
        x = read_int32(parser) / 256;
        y = read_int32(parser) / 256;
    } else {
        let y_diff = read_int16(parser) / 256;
        x = parser.last_position.x + x_diff;
        y = parser.last_position.y + y_diff;
    }
    parser.last_position.x = x;
    parser.last_position.x = y;
    return {x, y};
}

function read_bounding_box(parser) {
    return {
        left_top: read_map_position(parser),
        right_bottom: read_map_position(parser),
        orientation: {
            x: read_int16(parser),
            y: read_int16(parser)
        },
    };
}

function read_cliff_settings(parser) {
    return {
        name: read_string(parser),
        elevation_0: read_float(parser),
        elevation_interval: read_float(parser),
        richness: read_float(parser),
    };
}

function map_to_object(map) {
    let obj = {};
    for (let [key, value] of map) {
        obj[key] = value;
    }
    return obj;
}

function read_map_gen_settings(parser) {
    return {
        terrain_segmentation: read_float(parser),
        water: read_float(parser),
        autoplace_controls: map_to_object(read_dict(parser, read_string, read_frequency_size_richness)),
        autoplace_settings: map_to_object(read_dict(parser, read_string, read_autoplace_setting)),
        default_enable_all_autoplace_controls: read_bool(parser),
        seed: read_uint32(parser),
        width: read_uint32(parser),
        height: read_uint32(parser),
        area_to_generate_at_start: read_bounding_box(parser),
        starting_area: read_float(parser),
        peaceful_mode: read_bool(parser),
        starting_points: read_array(parser, read_map_position),
        property_expression_names: map_to_object(read_dict(parser, read_string, read_string)),
        cliff_settings: read_cliff_settings(parser),
    };
}

function read_pollution(parser) {
    let enabled;

    return {
        enabled: read_optional(parser, read_bool),
        diffusion_ratio: read_optional(parser, read_double),
        min_to_diffuse: read_optional(parser, read_double),
        ageing: read_optional(parser, read_double),
        expected_max_per_chunk: read_optional(parser, read_double),
        min_to_show_per_chunk: read_optional(parser, read_double),
        min_pollution_to_damage_trees: read_optional(parser, read_double),
        pollution_with_max_forest_damage: read_optional(parser, read_double),
        pollution_per_tree_damage: read_optional(parser, read_double),
        pollution_restored_per_tree_damage: read_optional(parser, read_double),
        max_pollution_to_restore_trees: read_optional(parser, read_double),
        enemy_attack_pollution_consumption_modifier: read_optional(parser, read_double),
    };
}

function read_real_steering(parser) {
    return {
        radius: read_optional(parser, read_double),
        separation_factor: read_optional(parser, read_double),
        separation_force: read_optional(parser, read_double),
        force_unit_fuzzy_goto_behavior: read_optional(parser, read_bool),
    };

}

function read_steering(parser) {
    return {
        default: read_real_steering(parser),
        moving: read_real_steering(parser),
    };
}

function read_enemy_evolution(parser) {
    return {
        enabled: read_optional(parser, read_bool),
        time_factor: read_optional(parser, read_double),
        destroy_factor: read_optional(parser, read_double),
        pollution_factor: read_optional(parser, read_double),
    };
}

function read_enemy_expansion(parser) {
    return {
        enabled: read_optional(parser, read_bool),
        max_expansion_distance: read_optional(parser, read_uint32),
        friendly_base_influence_radius: read_optional(parser, read_uint32),
        enemy_building_influence_radius: read_optional(parser, read_uint32),
        building_coefficient: read_optional(parser, read_double),
        other_base_coefficient: read_optional(parser, read_double),
        neighbouring_chunk_coefficient: read_optional(parser, read_double),
        neighbouring_base_chunk_coefficient: read_optional(parser, read_double),
        max_colliding_tiles_coefficient: read_optional(parser, read_double),
        settler_group_min_size: read_optional(parser, read_uint32),
        settler_group_max_size: read_optional(parser, read_uint32),
        min_expansion_cooldown: read_optional(parser, read_uint32),
        max_expansion_cooldown: read_optional(parser, read_uint32),
    };
}

function read_unit_group(parser) {
    return {
        min_group_gathering_time: read_optional(parser, read_uint32),
        max_group_gathering_time: read_optional(parser, read_uint32),
        max_wait_time_for_late_members: read_optional(parser, read_uint32),
        max_group_radius: read_optional(parser, read_double),
        min_group_radius: read_optional(parser, read_double),
        max_member_speedup_when_behind: read_optional(parser, read_double),
        max_member_slowdown_when_ahead: read_optional(parser, read_double),
        max_group_slowdown_factor: read_optional(parser, read_double),
        max_group_member_fallback_factor: read_optional(parser, read_double),
        member_disown_distance: read_optional(parser, read_double),
        tick_tolerance_when_member_arrives: read_optional(parser, read_uint32),
        max_gathering_unit_groups: read_optional(parser, read_uint32),
        max_unit_group_size: read_optional(parser, read_uint32),
    };
}

function read_path_finder(parser) {
    return {
        fwd2bwd_ratio: read_optional(parser, read_int32),
        goal_pressure_ratio: read_optional(parser, read_double),
        use_path_cache: read_optional(parser, read_bool),
        max_steps_worked_per_tick: read_optional(parser, read_double),
        max_work_done_per_tick: read_optional(parser, read_uint32),
        short_cache_size: read_optional(parser, read_uint32),
        long_cache_size: read_optional(parser, read_uint32),
        short_cache_min_cacheable_distance: read_optional(parser, read_double),
        short_cache_min_algo_steps_to_cache: read_optional(parser, read_uint32),
        long_cache_min_cacheable_distance: read_optional(parser, read_double),
        cache_max_connect_to_cache_steps_multiplier: read_optional(parser, read_uint32),
        cache_accept_path_start_distance_ratio: read_optional(parser, read_double),
        cache_accept_path_end_distance_ratio: read_optional(parser, read_double),
        negative_cache_accept_path_start_distance_ratio: read_optional(parser, read_double),
        negative_cache_accept_path_end_distance_ratio: read_optional(parser, read_double),
        cache_path_start_distance_rating_multiplier: read_optional(parser, read_double),
        cache_path_end_distance_rating_multiplier: read_optional(parser, read_double),
        stale_enemy_with_same_destination_collision_penalty: read_optional(parser, read_double),
        ignore_moving_enemy_collision_distance: read_optional(parser, read_double),
        enemy_with_different_destination_collision_penalty: read_optional(parser, read_double),
        general_entity_collision_penalty: read_optional(parser, read_double),
        general_entity_subsequent_collision_penalty: read_optional(parser, read_double),
        extended_collision_penalty: read_optional(parser, read_double),
        max_clients_to_accept_any_new_request: read_optional(parser, read_uint32),
        max_clients_to_accept_short_new_request: read_optional(parser, read_uint32),
        direct_distance_to_consider_short_request: read_optional(parser, read_uint32),
        short_request_max_steps: read_optional(parser, read_uint32),
        short_request_ratio: read_optional(parser, read_double),
        min_steps_to_check_path_find_termination: read_optional(parser, read_uint32),
        start_to_goal_cost_multiplier_to_terminate_path_find: read_optional(parser, read_double),
        overload_levels: read_optional(parser, (p) => read_array(p, read_uint32)),
        overload_multipliers: read_optional(parser, (p) => read_array(p, read_double)),
        negative_path_cache_delay_interval: read_optional(parser, read_uint32),
    };
}

function read_difficulty_settings(parser) {
    return {
        recipe_difficulty: read_uint8(parser),
        technology_difficulty: read_uint8(parser),
        technology_price_multiplier: read_double(parser),
        research_queue_setting: ["always", "after-victory", "never"][read_uint8(parser)],
    };
}

function read_map_settings(parser) {
    return {
        pollution: read_pollution(parser),
        steering: read_steering(parser),
        enemy_evolution: read_enemy_evolution(parser),
        enemy_expansion: read_enemy_expansion(parser),
        unit_group: read_unit_group(parser),
        path_finder: read_path_finder(parser),
        max_failed_behavior_count: read_uint32(parser),
        difficulty_settings: read_difficulty_settings(parser),
    };
}

function decode(s) {
    s = s.replace(/[ \t\n\r]+/g, "");
    if (!/>>>[0-9a-zA-Z\/+]+={0,3}<<</.test(s)) {
        return "Not a map exchange string";
    }

    let buf = Buffer.from(s.slice(3, -3), "base64");
    buf = zlib.inflateSync(buf);

    let parser = new Parser(buf);

    let data = {
        version: read_version(parser),
        unknown: read_uint8(parser),
        map_gen_settings: read_map_gen_settings(parser),
        map_settings: read_map_settings(parser),
        checksum: read_uint32(parser),
    };

    if (parser.pos != buf.length) {
        return "data after end";
    }

    // format of game.table_to_json(game.parse_map_exchange_string(..))
    data = {
        map_settings: data.map_settings,
        map_gen_settings: data.map_gen_settings,
        checksum: data.checksum
    }

    return data;
}

const file = process.argv[2];
if (!file) {
    console.log(`${process.argv[0]} ${process.argv[1]} [map exchange file]`);
    return;
}

function prettyJson(obj) {
    let data = util.inspect(obj, {
        depth: 20,
        compact: false
    })
    data = data.replaceAll(/([a-z0-9_]+):/g, '"$1":')
    data = data.replaceAll("'", '"');
    return data;
}

fs.readFile(file, 'utf8', (err, data) => {
    let decoded = decode(data)

    // let decodedJsonConsole = util.inspect(decoded, {depth: 20, colors: true});
    // let decodedJsonFile = util.inspect(decoded, {depth: 20, compact: false}).replaceAll(/([a-z0-9_]+):/g, '"$1":');

    let filePrefix = path.basename(file).split(".")[0]

    let mapGenSettingsFile = `${filePrefix}.mapGenSettings.json`
    let mapGenSettingsData = prettyJson(decoded.map_settings);
    fs.writeFileSync(mapGenSettingsFile, mapGenSettingsData)
    console.log(`write to ${mapGenSettingsFile}`);

    let mapSettingsFile = `${filePrefix}.mapSettings.json`
    let mapSettingsData = prettyJson(decoded.map_settings);
    fs.writeFileSync(mapSettingsFile, mapSettingsData)
    console.log(`write to ${mapSettingsFile}`);
})

