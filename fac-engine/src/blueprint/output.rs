use enum_map::EnumMap;
use std::{cell::RefCell, rc::Rc, sync::Mutex};
use tracing::{debug, error, trace};
use unicode_segmentation::UnicodeSegmentation;

const FLAG_ENABLE_LINE_REWRITE: bool = false;

use crate::{
    admiral::{
        err::AdmiralResult,
        executor::{ExecuteResponse, LuaCompiler, client::AdmiralClient},
        lua_command::LuaCommand,
    },
    util::ansi::{
        C_BLOCK_LINE, C_FULL_BLOCK, Color, ansi_color, ansi_erase_line, ansi_previous_line,
    },
};

use super::{
    bpfac::{entity::FacBpEntity, position::FacBpPosition},
    bpitem::BlueprintItem,
    contents::BlueprintContents,
};

const CACHE_SIZE: usize = 1;

/// Middleware between entity output and blueprint/lua output
/// Instead of generating everything "post", obscuring errors and logic
pub struct FacItemOutput {
    otype: FacItemOutputType,
}

impl FacItemOutput {
    pub fn into_rc(self) -> Rc<Self> {
        Rc::new(self)
    }

    pub fn consume_rc(self: Rc<Self>) -> Self {
        Rc::into_inner(self).unwrap()
    }

    pub fn new_admiral(client: AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient(RefCell::new(OutputData {
                inner: client,
                dedupe: None,
                cache: Vec::new(),
                total_write: 0,
            })),
        }
    }

    pub fn new_admiral_dedupe(client: AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient(RefCell::new(OutputData {
                inner: client,
                dedupe: Some(Vec::new()),
                cache: Vec::new(),
                total_write: 0,
            })),
        }
    }

    pub fn new_blueprint() -> Self {
        Self {
            otype: FacItemOutputType::Blueprint(RefCell::new(OutputData {
                inner: BlueprintContents::new(),
                dedupe: None,
                cache: Vec::new(),
                total_write: 0,
            })),
        }
    }

    pub fn write(&self, item: BlueprintItem) {
        let blueprint = item.to_blueprint();
        let item_debug = format!("{:?}", item.entity());
        let (contexts, subcontexts, total_with_context) = get_global_context_map(|log_info| {
            let contexts = ansi_color(
                log_info.context_map[ContextLevel::Block].join(&format!(" {C_BLOCK_LINE} ")),
                Color::Green,
            );
            let subcontexts = ansi_color(
                log_info.context_map[ContextLevel::Micro].join(" ! "),
                Color::Purple,
            );
            let key = [item_debug.as_str(), contexts.as_str(), subcontexts.as_str()].concat();
            if log_info.last_context != key {
                log_info.last_context = key;
                log_info.total_with_context = 1;
                // print!("\n");
            } else {
                log_info.total_with_context += 1;
                if FLAG_ENABLE_LINE_REWRITE {
                    print!("{}", ansi_previous_line());
                }
            };
            (contexts, subcontexts, log_info.total_with_context)
        });
        let subcontexts = pad_grapheme(&subcontexts, 40);

        let total_progress = C_FULL_BLOCK.repeat(total_with_context.min(30));
        debug!(
            "{}blueprint pos {:6} facpos {:10} {:80} {contexts:42} {total_with_context:2} {subcontexts} total {total_progress}",
            ansi_erase_line(),
            item.position().display(),
            blueprint.position.display(),
            item_debug,
        );

        self.otype.write(FacItemOutputWrite { item, blueprint })
    }
    // pub fn any_handle<'o>(&'o mut self) -> OutputContextHandle<'o> {
    //     OutputContextHandle {
    //         output: self,
    //         htype: OutputContextHandleType::Empty,
    //     }
    // }

    pub fn context_handle(
        &self,
        context_level: ContextLevel,
        new_context: String,
    ) -> OutputContextHandle {
        let res = OutputContextHandle {
            context_level: context_level.clone(),
        };
        get_global_context_map(move |log_info| {
            log_info.context_map[context_level.clone()].push(new_context.clone())
        });
        res
    }

    pub fn log_info(&self) -> FacItemOutputLogInfo {
        get_global_context_map(|log_info| log_info.questitionable_clone())
    }

    pub fn admiral_execute_command(
        &self,
        lua: Box<dyn LuaCommand>,
    ) -> AdmiralResult<ExecuteResponse> {
        self.otype.admiral_execute_command(lua)
    }

    #[cfg(test)]
    pub fn last_blueprint_write_last(&self) -> test::LastWrite {
        use test::LastWrite;

        let blueprint = &self.unwrap_blueprint().inner;
        let last_item = blueprint.items().last().unwrap();
        LastWrite {
            blueprint: blueprint.fac_entities().last().unwrap().clone(),
            size: last_item.entity().rectangle_size(),
        }
    }

    pub fn into_blueprint_contents(self) -> BlueprintContents {
        match self.otype {
            FacItemOutputType::Blueprint(inner) => {
                let output_data = inner.into_inner();
                output_data.inner
            }
            FacItemOutputType::AdmiralClient(_) => panic!("not a blueprint"),
        }
    }

    pub fn flush(&self) {
        self.otype.flush_cache();
    }

    #[cfg(test)]
    fn unwrap_blueprint(&self) -> std::cell::Ref<'_, OutputData<BlueprintContents>> {
        match &self.otype {
            FacItemOutputType::Blueprint(inner) => inner.borrow(),
            FacItemOutputType::AdmiralClient(_) => panic!("not a blueprint"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{blueprint::bpfac::entity::FacBpEntity, common::entity::Size};

    pub struct LastWrite {
        pub blueprint: FacBpEntity,
        pub size: Size,
    }
}

fn pad_grapheme(input: &str, up_to: usize) -> String {
    let actual_len: usize = input
        .graphemes(false)
        .map(|v| if v.len() > 1 { 2 } else { 1 })
        .sum();
    let diff = up_to.saturating_sub(actual_len);
    if diff > 0 {
        format!("{}{:diff$}", input, "")
    } else {
        input.to_string()
    }
}

#[derive(Clone, enum_map::Enum)]
pub enum ContextLevel {
    Block,
    Micro,
}

/// TODO: lifetime soup with multiple muts is hard. New plan: Global!
fn get_global_context_map<A, R>(mut action: A) -> R
where
    A: FnMut(&mut FacItemOutputLogInfo) -> R,
{
    // thread_local! {
    static CUR: Mutex<Option<FacItemOutputLogInfo>> = Mutex::new(None);
    // }

    // enum_map::enum_map! {
    // ContextLevel::Block => Vec::new(),
    // ContextLevel::Micro => Vec::new(),
    // }

    let mut lock = CUR.lock().unwrap();
    let map = lock.get_or_insert_default();
    action(map)
}

enum FacItemOutputType {
    AdmiralClient(RefCell<OutputData<AdmiralClient>>),
    Blueprint(RefCell<OutputData<BlueprintContents>>),
}

struct FacItemOutputWrite {
    item: BlueprintItem,
    blueprint: FacBpEntity,
}

impl FacItemOutputType {
    fn write(&self, write: FacItemOutputWrite) {
        self.push_cache(write);
        self.flush_cache_maybe();
    }

    fn push_cache(&self, write: FacItemOutputWrite) {
        match self {
            Self::AdmiralClient(cell) => {
                let OutputData { cache, .. } = &mut *cell.borrow_mut();
                cache.push(write);
            }
            Self::Blueprint(cell) => {
                let OutputData { cache, .. } = &mut *cell.borrow_mut();
                cache.push(write);
            }
        }
    }

    fn flush_cache_maybe(&self) {
        let size = match self {
            Self::AdmiralClient(cell) => {
                let OutputData { cache, .. } = &*cell.borrow();
                cache.len()
            }
            Self::Blueprint(cell) => {
                let OutputData { cache, .. } = &*cell.borrow();
                cache.len()
            }
        };
        if size < CACHE_SIZE {
            return;
        }
        self.flush_cache();
    }

    fn flush_cache(&self) {
        match self {
            Self::AdmiralClient(cell) => {
                let OutputData {
                    inner,
                    dedupe: _,
                    cache,
                    total_write,
                } = &mut *cell.borrow_mut();
                // dedupe_position(dedupe, &blueprint);

                let mut lua_commands = Vec::new();
                for FacItemOutputWrite { item, blueprint } in cache.drain(0..) {
                    *total_write += 1;
                    lua_commands.push(blueprint.to_lua().into_boxed());
                }
                let flush_count = lua_commands.len();
                if flush_count > 2 {
                    // don't spam the console on micro writes
                    trace!("Flush Cache {} total {}", flush_count, total_write)
                }
                let res = inner.execute_checked_commands_in_wrapper_function(lua_commands);

                // Vec::push() does not normally fail
                // For API sanity, do not make every FacBlk need to pass up the error
                if let Err(e) = res {
                    error!("⛔⛔⛔ Write failed {}", e);
                }
            }
            Self::Blueprint(cell) => {
                let OutputData {
                    inner,
                    dedupe: _,
                    cache,
                    total_write,
                } = &mut *cell.borrow_mut();
                let mut flush_count = 0;
                for FacItemOutputWrite { item, blueprint } in cache.drain(0..) {
                    flush_count += 1;
                    inner.add(item, blueprint);
                }
                *total_write += flush_count;

                if flush_count > 2 {
                    // don't spam the console on micro writes
                    trace!("Flush Cache {} total {}", flush_count, total_write)
                }
            }
        }
    }

    // fn push_context(&self, context_level: ContextLevel, new_context: String) {
    //     match self {
    //         Self::AdmiralClient(cell) => {
    //             cell.borrow_mut().log_info.context_map[context_level].push(new_context)
    //         }
    //         Self::Blueprint(cell) => {
    //             cell.borrow_mut().log_info.context_map[context_level].push(new_context)
    //         }
    //     }
    // }

    // fn pop_context(&self, context_level: ContextLevel) {
    //     match self {
    //         Self::AdmiralClient(cell) => {
    //             cell.borrow_mut().log_info.context_map[context_level].pop()
    //         }
    //         Self::Blueprint(cell) => cell.borrow_mut().log_info.context_map[context_level].pop(),
    //     }
    //     .unwrap();
    // }

    // fn get_context(&self) -> FacItemOutputLogInfo {
    //     match self {
    //         Self::AdmiralClient(cell) => cell.borrow().log_info.questitionable_clone(),
    //         Self::Blueprint(cell) => cell.borrow().log_info.questitionable_clone(),
    //     }
    // }

    fn admiral_execute_command(&self, lua: Box<dyn LuaCommand>) -> AdmiralResult<ExecuteResponse> {
        match self {
            Self::AdmiralClient(cell) => cell.borrow_mut().inner.execute_checked_command(lua),
            Self::Blueprint(_) => panic!("not a admiral"),
        }
    }
}

// fn dedupe_position(dedupe: &mut Option<Vec<FacBpPosition>>, blueprint: &FacBpEntity) {
//     if let Some(dedupe) = dedupe {
//         let bppos = &blueprint.position;
//         if dedupe.contains(bppos) {
//             return;
//         } else {
//             dedupe.push(bppos.clone());
//         }
//     }
// }

#[derive(Default)]
pub struct FacItemOutputLogInfo {
    pub context_map: EnumMap<ContextLevel, Vec<String>>,
    pub last_context: String,
    pub total_with_context: usize,
}

impl FacItemOutputLogInfo {
    fn questitionable_clone(&self) -> Self {
        Self {
            context_map: self.context_map.clone(),
            last_context: self.last_context.clone(),
            total_with_context: self.total_with_context,
        }
    }
}

struct OutputData<T> {
    inner: T,
    dedupe: Option<Vec<FacBpPosition>>,
    cache: Vec<FacItemOutputWrite>,
    total_write: usize,
}

// Keeps the context alive for access during logging
pub struct OutputContextHandle {
    context_level: ContextLevel,
}

impl Drop for OutputContextHandle {
    fn drop(&mut self) {
        get_global_context_map(|log_info| log_info.context_map[self.context_level.clone()].pop());
    }
}

// impl Deref for OutputContextHandle {
//     type Target = FacItemOutput;

//     fn deref(&self) -> &Self::Target {
//         self.output
//     }
// }
