#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LoadedPartSets {
    pub talk_set: String,
    pub radio_set: String,
    pub next_phase_name: String,
}

#[derive(Debug, Default)]
pub struct LoadedPartSetsWatch {
    last: Option<LoadedPartSets>,
}

impl LoadedPartSetsWatch {
    pub fn observe(&mut self, now: &LoadedPartSets) -> Option<String> {
        if self.last.as_ref() == Some(now) {
            return None;
        }
        self.last = Some(now.clone());
        Some(format!(
            "part sets: talk window holds {}, radio holds {}, phase {}",
            label(&now.talk_set),
            label(&now.radio_set),
            label(&now.next_phase_name)
        ))
    }
}

fn label(text: &str) -> &str {
    if text.is_empty() { "none" } else { text }
}
