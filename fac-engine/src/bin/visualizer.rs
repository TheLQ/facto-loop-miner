use facto_loop_miner_fac_engine::{
    admiral::lua_command::fac_surface_create_entity::FacSurfaceCreateEntity,
    blueprint::{blueprint::Blueprint, bpitem::BlueprintItem, contents::BlueprintContents},
    common::{entity::FacEntity, vpoint::VPoint},
    game_blocks::rail_station::RailStation,
    game_entities::{assembler::FacAssembler, chest::FacChestType, tier::FacTier},
    visualizer::visualizer::visualize_blueprint,
};

fn main() {
    let mut bp_contents = BlueprintContents::new();

    match 2 {
        1 => basic_build_bp(&mut bp_contents),
        2 => basic_build_gen(&mut bp_contents),
        _ => panic!("asdf"),
    }

    visualize_blueprint(&bp_contents);

    let blueprint = Blueprint::new(bp_contents);

    let res = encode_blueprint_to_string(&blueprint.to_fac()).unwrap();
    println!("bp {}", res)
}

fn basic_build_bp(bp: &mut BlueprintContents) {
    {
        let entity = FacAssembler::new_basic(FacTier::Tier1, "something".into());
        bp.add_entity_each(BlueprintItem::new(entity.into_boxed(), VPoint::new(1, 1)));
    }

    {
        let entity = FacAssembler::new_basic(FacTier::Tier1, "something2".into());
        bp.add_entity_each(BlueprintItem::new(entity.into_boxed(), VPoint::new(1, 4)));
    }
}

fn basic_build_gen(bp: &mut BlueprintContents) {
    let station = RailStation::new(3, Some(FacChestType::Passive), 2);

    for entity in station.generate(VPoint::new(5, 5)) {
        bp.add_entity_each(entity);
    }
}
