use crate::admiral::{
    executor::{LuaCompiler, client::AdmiralClient},
    lua_command::LuaCommand,
};

use super::{
    bpfac::{entity::FacBpEntity, position::FacBpPosition},
    bpitem::BlueprintItem,
    contents::BlueprintContents,
};

pub struct FacItemOutput<'c> {
    otype: FacItemOutputType<'c>,
    dedupe: Option<Vec<FacBpPosition>>,
}

impl<'c> FacItemOutput<'c> {
    pub fn new_admiral(client: &'c mut AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient { client },
            dedupe: None,
        }
    }

    pub fn new_admiral_dedupe(client: &'c mut AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient { client },
            dedupe: Some(Vec::new()),
        }
    }

    pub fn new_blueprint(blueprint: &'c mut BlueprintContents) -> Self {
        Self {
            otype: FacItemOutputType::Blueprint { blueprint },
            dedupe: None,
        }
    }

    pub fn write(&mut self, item: BlueprintItem) {
        let blueprint = item.to_blueprint();
        if let Some(dedupe) = &mut self.dedupe {
            let bppos = &blueprint.position;
            if dedupe.contains(bppos) {
                return;
            } else {
                dedupe.push(bppos.clone());
            }
        }

        self.otype.write(item, blueprint)
    }
}

pub enum FacItemOutputType<'c> {
    AdmiralClient {
        client: &'c mut AdmiralClient,
    },
    Blueprint {
        blueprint: &'c mut BlueprintContents,
    },
}

impl FacItemOutputType<'_> {
    pub fn write(&mut self, item: BlueprintItem, blueprint: FacBpEntity) {
        match self {
            Self::AdmiralClient { client } => {
                let res = client.execute_checked_command(blueprint.to_lua().into_boxed());
                // Vec::push() does not normally fail
                // For API sanity, do not make every FacBlk need to pass up the error
                if let Err(e) = res {
                    panic!("write failed {}", e);
                }
            }
            Self::Blueprint { blueprint: bp } => {
                bp.add_entity_each(item);
            }
        }
    }
}
