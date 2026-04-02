use std::collections::HashMap;

use sova_core::vm::{EvaluationContext, event::ConcreteEvent, variable::VariableValue};

use crate::boinx::ast::BoinxItem;

pub fn make_internal_event(ctx: &EvaluationContext, item: BoinxItem) -> Option<ConcreteEvent> {
    let BoinxItem::ArgMap(map) = item else {
        return None;
    };
    let mut map: HashMap<String, VariableValue> = map
        .into_iter()
        .filter_map(|(key, value)| {
            if !value.is_primitive() {
                None
            } else {
                Some((key, VariableValue::from(value)))
            }
        })
        .collect();
    let cmd = map.remove("sched").unwrap().as_str(ctx);
    match cmd.as_str() {
        "exe" => {
            let l_i = map.remove("line").unwrap_or_default().as_integer(ctx) as usize;
            let f_i = map.remove("frame").unwrap_or_default().as_integer(ctx) as usize;
            Some(ConcreteEvent::ExecuteFrame(l_i, f_i))
        }
        "en" => {
            let l_i = map.remove("line").unwrap_or_default().as_integer(ctx) as usize;
            let f_i = map.remove("frame").unwrap_or_default().as_integer(ctx) as usize;
            let en = map.remove("value").unwrap_or_default().as_bool(ctx);
            Some(ConcreteEvent::SetFrameEnabled(l_i, f_i, en))
        }
        "dur" => {
            let l_i = map.remove("line").unwrap_or_default().as_integer(ctx) as usize;
            let f_i = map.remove("frame").unwrap_or_default().as_integer(ctx) as usize;
            let dur = map.remove("value").unwrap_or_default().as_float(ctx);
            Some(ConcreteEvent::SetFrameDuration(l_i, f_i, dur))
        }
        "looping" => {
            let l_i = map.remove("line").unwrap_or_default().as_integer(ctx) as usize;
            let looping = map.remove("value").unwrap_or_default().as_bool(ctx);
            Some(ConcreteEvent::SetLineLooping(l_i, looping))
        }
        "trailing" => {
            let l_i = map.remove("line").unwrap_or_default().as_integer(ctx) as usize;
            let trailing = map.remove("value").unwrap_or_default().as_bool(ctx);
            Some(ConcreteEvent::SetLineTrailing(l_i, trailing))
        }
        "manual" => {
            let l_i = map.remove("line").unwrap_or_default().as_integer(ctx) as usize;
            let manual = map.remove("value").unwrap_or_default().as_bool(ctx);
            Some(ConcreteEvent::SetLineManual(l_i, manual))
        }
        "speed" => {
            let l_i = map.remove("line").unwrap_or_default().as_integer(ctx) as usize;
            let sp = map.remove("value").unwrap_or_default().as_float(ctx);
            Some(ConcreteEvent::SetLineSpeedFactor(l_i, sp))
        }
        "edit" => {
            let l_i = map.remove("line").unwrap_or_default().as_integer(ctx) as usize;
            let f_i = map.remove("frame").unwrap_or_default().as_integer(ctx) as usize;
            let lang = map.remove("lang").unwrap_or_default().as_str(ctx);
            let text = map.remove("text").unwrap_or_default().as_str(ctx);
            Some(ConcreteEvent::SetFrame(l_i, f_i, lang, text))
        }
        "kill" => {
            let l_i = map.remove("line").unwrap_or_default().as_integer(ctx) as usize;
            let f_i = map.remove("frame").unwrap_or_default().as_integer(ctx) as usize;
            Some(ConcreteEvent::KillExecutions(l_i, f_i))
        }
        "tempo" => {
            let t = map.remove("value").unwrap_or_default().as_float(ctx);
            Some(ConcreteEvent::SetTempo(t))
        }
        _ => None,
    }
}