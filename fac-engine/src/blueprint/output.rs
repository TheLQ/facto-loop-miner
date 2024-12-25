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
    log_info: FacItemOutputLogInfo,
}

impl<'c> FacItemOutput<'c> {
    pub fn new_admiral(client: &'c mut AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient {
                client,
                // all_items: Vec::new(),
            },
            dedupe: None,
            log_info: FacItemOutputLogInfo::new(),
        }
    }

    pub fn new_admiral_dedupe(client: &'c mut AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient {
                client,
                // all_items: Vec::new(),
            },
            dedupe: Some(Vec::new()),
            log_info: FacItemOutputLogInfo::new(),
        }
    }

    pub fn new_blueprint(blueprint: &'c mut BlueprintContents) -> Self {
        Self {
            otype: FacItemOutputType::Blueprint { blueprint },
            dedupe: None,
            log_info: FacItemOutputLogInfo::new(),
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
        self.log_info.contexts.push(new_context);
        OutputContextHandle {
            output: self,
            is_subcontext: false,
        }
    }

    pub fn subcontext_handle<'s>(&'s mut self, new_context: String) -> OutputContextHandle<'s, 'c> {
        self.log_info.subcontexts.push(new_context);
        OutputContextHandle {
            output: self,
            is_subcontext: true,
        }
    }

    pub fn log_info(&self) -> &FacItemOutputLogInfo {
        &self.log_info
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
                    panic!("⛔⛔⛔ Write failed {}", e);
                }
            }
            Self::Blueprint {
                blueprint: blueprint_contents,
            } => {
                blueprint_contents.add(item, blueprint);
            }
        }
    }
}

pub struct FacItemOutputLogInfo {
    pub contexts: Vec<String>,
    pub subcontexts: Vec<String>,
}

impl FacItemOutputLogInfo {
    pub const fn new() -> Self {
        Self {
            contexts: Vec::new(),
            subcontexts: Vec::new(),
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
            &mut self.log_info.subcontexts
        } else {
            &mut self.log_info.contexts
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
