use std::rc::Rc;

use crate::{
    blueprint::{
        bpitem::BlueprintItem,
        output::{ContextLevel, FacItemOutput},
    },
    common::{entity::FacEntity, names::FacEntityName, vpoint::VPoint},
    game_entities::{
        assembler::{FacEntAssembler, FacEntAssemblerModSlice},
        belt::FacEntBeltType,
        direction::FacDirectionQuarter,
        inserter::FacEntInserterType,
        lamp::FacEntLamp,
        tier::FacTier,
    },
};

use super::{assembler_thru::FacBlkAssemblerThru, belt_bettel::FacBlkBettelBelt, block::FacBlock};

pub struct FacBlkIndustry {
    pub belt: FacEntBeltType,
    pub inserter_input: FacEntInserterType,
    pub inserter_output: FacEntInserterType,
    pub assembler_tier: FacTier,
    pub assembler_modules: FacEntAssemblerModSlice,
    pub thru: Vec<IndustryThru>,
    pub output: Rc<FacItemOutput>,
}

#[derive(Debug)]
pub struct IndustryThru {
    pub recipe: FacEntityName,
    pub input_belts: usize,
    pub width: usize,
    pub height: usize,
    pub custom_inserter_input: Option<FacEntInserterType>,
    pub custom_inserter_output: Option<FacEntInserterType>,
    pub custom_assembler_modules: Option<FacEntAssemblerModSlice>,
}

impl FacBlock for FacBlkIndustry {
    fn generate(&self, origin: VPoint) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Block, "Industry".into());

        // let mut output_belts = self.place_input_belts(origin);

        let thru_blocks = self.place_thru(origin);
        self.place_input_belts(&thru_blocks, origin);

        let hj = Self::get_thru_output_x(thru_blocks);
    }
}

impl FacBlkIndustry {
    fn place_input_belts(&self, thru_block: &[FacBlkAssemblerThru], origin: VPoint) {
        let _ = &mut self
            .output
            .context_handle(ContextLevel::Micro, format!("Input Belts"));
        let max_width: usize = thru_block
            .iter()
            .map(|v| v.total_point_width())
            .max()
            .unwrap();

        // stick out as much as output belts from underground
        let output_padding: usize = 2;

        let mut cur_total_belts: usize = 0;
        for (thru_num, thru) in self.thru.iter().enumerate() {
            for cur_thr_belt in 0..thru.input_belts {
                let bettel_origin = origin.move_xy(
                    (max_width + output_padding) as i32 - /*??*/1,
                    -(cur_total_belts as i32 + /*above assemblers*/1),
                );

                let mut belt = FacBlkBettelBelt::new(
                    self.belt,
                    bettel_origin,
                    FacDirectionQuarter::West,
                    self.output.clone(),
                );

                // come back from the output side
                belt.add_straight(max_width + output_padding as usize);

                // going down to assembler thru inputs
                belt.add_straight(cur_total_belts);
                belt.add_turn90(false);
                belt.add_straight(cur_total_belts);

                // intra assembler skip
                for inner_thru in &thru_block[0..thru_num] {
                    belt.add_straight(inner_thru.total_point_height());
                }

                // going to assembler thru input
                belt.add_straight(cur_thr_belt);
                belt.add_turn90(false);
                belt.add_straight(cur_total_belts);

                cur_total_belts += 1;
            }
        }
    }

    fn place_thru(&self, origin: VPoint) -> Vec<FacBlkAssemblerThru> {
        let max_width = self.max_thru_cell_width();

        let mut thru_blocks: Vec<FacBlkAssemblerThru> = Vec::new();
        let mut next_start = origin;
        for thru in &self.thru {
            let block = FacBlkAssemblerThru {
                assembler: FacEntAssembler::new(
                    self.assembler_tier,
                    thru.recipe.clone(),
                    thru.custom_assembler_modules
                        .unwrap_or(self.assembler_modules),
                ),
                belt_type: self.belt,
                height: thru.height,
                width: thru.width,
                output_padding_width: Some(max_width),
                inserter_input: thru.custom_inserter_input.unwrap_or(self.inserter_input),
                inserter_output: thru.custom_inserter_output.unwrap_or(self.inserter_output),
                integ_input: Default::default(),
                integ_output: Default::default(),
                output: self.output.clone(),
            };
            block.generate(next_start);
            next_start = next_start.move_y_usize(block.total_point_height() /*+ arbitrary5*/);

            thru_blocks.push(block);
        }

        for block in &thru_blocks {
            // for input_point in block.integ_input.borrow().iter() {
            //     self.output.write(BlueprintItem::new(
            //         FacEntLamp::new().into_boxed(),
            //         input_point.move_x(-1),
            //     ));
            // }

            for belt in block.integ_output.borrow().iter() {
                self.output.write(BlueprintItem::new(
                    FacEntLamp::new().into_boxed(),
                    belt.next_insert_position(),
                ));
            }
        }
        thru_blocks
    }

    fn place_output_belts() {}

    fn max_thru_cell_width(&self) -> usize {
        self.thru.iter().map(|v| v.width).max().unwrap()
    }

    fn get_thru_output_x(thru_blocks: Vec<FacBlkAssemblerThru>) -> i32 {
        let pos = thru_blocks[0].integ_output.borrow()[0].next_insert_position();
        for thru_block in thru_blocks {
            for output_belt in thru_block.integ_output.borrow().iter() {
                let output_pos = output_belt.next_insert_position();
                if output_pos.x() != pos.x() {
                    panic!(
                        "unexpected X haystack {} needle {}",
                        output_pos.display(),
                        pos.display()
                    );
                }
            }
        }
        pos.x()
    }
}
