use crate::fight_sync::FightWindowData;
use crate::script_functions::script_function;

pub fn is_start_chain_of(window: &FightWindowData, name: &str) -> bool {
    window.start_ids.contains(&name)
        || script_function(window.pack, name).is_some_and(|function| {
            function
                .callees
                .iter()
                .any(|callee| window.start_ids.contains(callee))
        })
}
