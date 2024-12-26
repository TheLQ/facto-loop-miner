use enum_map::EnumMap;
use std::{cell::RefCell, rc::Rc, sync::Mutex};
use tracing::{debug, error};
use unicode_segmentation::UnicodeSegmentation;

const FLAG_ENABLE_LINE_REWRITE: bool = false;

use crate::{
    admiral::{
        err::AdmiralResult,
        executor::{ExecuteResponse, LuaCompiler, client::AdmiralClient},
        lua_command::LuaCommand,
    },
    common::{
        ascii_color::{Color, ansi_color, ascii_erase_line, ascii_previous_line},
        vpoint::C_BLOCK_LINE,
    },
};

use super::{
    bpfac::{entity::FacBpEntity, position::FacBpPosition},
    bpitem::BlueprintItem,
    contents::BlueprintContents,
};

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
                // log_info: FacItemOutputLogInfo::new(),
            })),
        }
    }

    pub fn new_admiral_dedupe(client: AdmiralClient) -> Self {
        Self {
            otype: FacItemOutputType::AdmiralClient(RefCell::new(OutputData {
                inner: client,
                dedupe: Some(Vec::new()),
                // log_info: FacItemOutputLogInfo::new(),
            })),
        }
    }

    pub fn new_blueprint() -> Self {
        Self {
            otype: FacItemOutputType::Blueprint(RefCell::new(OutputData {
                inner: BlueprintContents::new(),
                dedupe: None,
                // log_info: FacItemOutputLogInfo::new(),
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
                    print!("{}", ascii_previous_line());
                }
            };
            (contexts, subcontexts, log_info.total_with_context)
        });
        let subcontexts = pad_grapheme(&subcontexts, 40);
        const C_FULL_BLOCK: &str = "\u{2588}";
        let total_progress = C_FULL_BLOCK.repeat(total_with_context);
        debug!(
            "{}blueprint pos {:6} facpos {:10} {:65} {contexts:42} {total_with_context:2} {subcontexts} total {total_progress}",
            ascii_erase_line(),
            item.position().display(),
            blueprint.position.display(),
            item_debug,
        );

        self.otype.write(item, blueprint)
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

    pub fn into_blueprint_contents(self) -> BlueprintContents {
        match self.otype {
            FacItemOutputType::Blueprint(inner) => {
                let output_data = inner.into_inner();
                output_data.inner
            }
            FacItemOutputType::AdmiralClient(_) => panic!("not a blueprint"),
        }
    }
}

fn pad_grapheme(input: &str, up_to: usize) -> String {
    let actual_len: usize = input
        .graphemes(false)
        .map(|v| if v.len() > 1 { 2 } else { 1 })
        .sum();
    let diff = up_to - actual_len;
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

impl FacItemOutputType {
    fn write(&self, item: BlueprintItem, blueprint: FacBpEntity) {
        match self {
            Self::AdmiralClient(cell) => {
                let OutputData { inner, dedupe, .. } = &mut *cell.borrow_mut();
                dedupe_position(dedupe, &blueprint);
                let res = inner.execute_checked_command(blueprint.to_lua().into_boxed());
                // all_items.push(item);
                // Vec::push() does not normally fail
                // For API sanity, do not make every FacBlk need to pass up the error
                if let Err(e) = res {
                    error!("⛔⛔⛔ Write failed {}", e);
                }
            }
            Self::Blueprint(cell) => {
                let OutputData { inner, dedupe, .. } = &mut *cell.borrow_mut();
                dedupe_position(dedupe, &blueprint);
                inner.add(item, blueprint);
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

fn dedupe_position(dedupe: &mut Option<Vec<FacBpPosition>>, blueprint: &FacBpEntity) {
    if let Some(dedupe) = dedupe {
        let bppos = &blueprint.position;
        if dedupe.contains(bppos) {
            return;
        } else {
            dedupe.push(bppos.clone());
        }
    }
}

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
    // log_info: FacItemOutputLogInfo,
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
