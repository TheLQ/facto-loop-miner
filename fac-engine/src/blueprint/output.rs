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
            otype: FacItemOutputType::AdmiralClient {
                client,
                // all_items: Vec::new(),
            },
            dedupe: None,
        }
    }

    pub fn new_admiral_dedupe(client: &'c mut AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient {
                client,
                // all_items: Vec::new(),
            },
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
        // TODO: Might be slow with full map generates?
        // all_items: Vec<BlueprintItem>,
    },
    Blueprint {
        blueprint: &'c mut BlueprintContents,
    },
}

impl FacItemOutputType<'_> {
    pub fn write(&mut self, item: BlueprintItem, blueprint: FacBpEntity) {
        match self {
            Self::AdmiralClient {
                client, /* , all_items */
            } => {
                let res = client.execute_checked_command(blueprint.to_lua().into_boxed());
                // all_items.push(item);
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
