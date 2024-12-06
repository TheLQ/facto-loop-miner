use facto_loop_miner_fac_engine::{
    blueprint::{blueprint::Blueprint, bpitem::BlueprintItem, contents::BlueprintContents},
    common::{entity::FacEntity, vpoint::VPoint},
    game_blocks::rail_station::RailStation,
    game_entities::assembler::{FacAssembler, FacAssemblerLevel},
    visualizer::visualizer::visualize_blueprint,
};

fn main() {
    let mut bpContents = BlueprintContents::new();

    match 2 {
        1 => basic_build_bp(&mut bpContents),
        2 => basic_build_gen(&mut bpContents),
        _ => panic!("asdf"),
    }

    visualize_blueprint(&bpContents);
}

fn basic_build_bp(bp: &mut BlueprintContents) {
    {
        let entity = FacAssembler::new(FacAssemblerLevel::Tier1, "something".into());
        bp.add_entity_each(BlueprintItem::new(entity.into_boxed(), VPoint::new(1, 1)));
    }

    {
        let entity = FacAssembler::new(FacAssemblerLevel::Tier1, "something2".into());
        bp.add_entity_each(BlueprintItem::new(entity.into_boxed(), VPoint::new(1, 4)));
    }
}

fn basic_build_gen(bp: &mut BlueprintContents) {
    let station = RailStation::new(3);

    for entity in station.generate(VPoint::new(5, 5)) {
        bp.add_entity_each(entity);
    }
}
