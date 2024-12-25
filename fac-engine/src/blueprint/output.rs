use std::ops::{Deref, DerefMut};

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
    pub contexts: Vec<String>,
    pub subcontexts: Vec<String>,
}

impl<'c> FacItemOutput<'c> {
    pub fn new_admiral(client: &'c mut AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient {
                client,
                // all_items: Vec::new(),
            },
            dedupe: None,
            contexts: Vec::new(),
            subcontexts: Vec::new(),
        }
    }

    pub fn new_admiral_dedupe(client: &'c mut AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient {
                client,
                // all_items: Vec::new(),
            },
            dedupe: Some(Vec::new()),
            contexts: Vec::new(),
            subcontexts: Vec::new(),
        }
    }

    pub fn new_blueprint(blueprint: &'c mut BlueprintContents) -> Self {
        Self {
            otype: FacItemOutputType::Blueprint { blueprint },
            dedupe: None,
            contexts: Vec::new(),
            subcontexts: Vec::new(),
        }
    }

    pub fn write(&mut self, item: BlueprintItem) {
        let blueprint = item.to_blueprint(&self);
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

    pub fn context_handle<'s>(&'s mut self, new_context: String) -> OutputContextHandle<'s, 'c> {
        self.contexts.push(new_context);
        OutputContextHandle {
            output: self,
            is_subcontext: false,
        }
    }

    pub fn subcontext_handle<'s>(&'s mut self, new_context: String) -> OutputContextHandle<'s, 'c> {
        self.subcontexts.push(new_context);
        OutputContextHandle {
            output: self,
            is_subcontext: true,
        }
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

// Keeps the context alive for access during logging
pub struct OutputContextHandle<'o, 'c> {
    output: &'o mut FacItemOutput<'c>,
    is_subcontext: bool,
}

impl Drop for OutputContextHandle<'_, '_> {
    fn drop(&mut self) {
        let pruned = if self.is_subcontext {
            &mut self.subcontexts
        } else {
            &mut self.output.contexts
        };
        let _context = pruned.pop().unwrap();
        // println!("drop context {}", _context)
    }
}

impl<'c> Deref for OutputContextHandle<'_, 'c> {
    type Target = FacItemOutput<'c>;

    fn deref(&self) -> &Self::Target {
        self.output
    }
}

impl<'c> DerefMut for OutputContextHandle<'_, 'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.output
    }
}
