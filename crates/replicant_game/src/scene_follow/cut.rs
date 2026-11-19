use replicant_proto::{MAX_SCENE_CUT_NAME_BYTES, SceneCut, StoryCall, StoryCallArg};

use crate::scene_follow::{EVENT_CUT_LOAD_FUNCTION, EVENT_CUT_LOAD_NO_AUDIO_NUMBER};

/// What this game's event manager holds: whether a cut is loaded and the event name it was
/// loaded under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalCut {
    pub loaded: bool,
    pub event: Option<String>,
}

/// Why a scene follow's cut load that still waits for the script slot must not run any more.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaleCutLoad {
    /// The cut it loads is loaded here now (this game's own scripts loaded it).
    CutLoadedHere,
    /// This game plays a scene: the load's first step frees the loaded cut and ends that scene
    /// with no actor restore, leaving the actors it moved on freed motion data.
    OwnSceneRuns,
}

/// The text of a fixed name buffer up to its NUL, when it is a short printable name.
pub fn cut_name_from_buffer(buffer: &[u8]) -> Option<String> {
    let end = buffer.iter().position(|&byte| byte == 0)?;
    let name = &buffer[..end];
    let printable = name.iter().all(u8::is_ascii_graphic);
    (!name.is_empty() && name.len() <= MAX_SCENE_CUT_NAME_BYTES && printable)
        .then(|| String::from_utf8_lossy(name).into_owned())
}

/// The cut the host sends with its scene: none unless a cut is loaded under a readable name.
pub fn host_scene_cut(loaded: bool, event_buffer: &[u8], audio_buffer: &[u8]) -> Option<SceneCut> {
    if !loaded {
        return None;
    }
    let event = cut_name_from_buffer(event_buffer)?;
    let audio = cut_name_from_buffer(audio_buffer).unwrap_or_default();
    Some(SceneCut { event, audio })
}

/// The host's cut when this game lacks it: nothing loaded, or a different event loaded.
pub fn cut_to_load(local: &LocalCut, host: Option<&SceneCut>) -> Option<SceneCut> {
    let host = host?;
    let same_event = local.loaded && local.event.as_deref() == Some(host.event.as_str());
    (!same_event).then(|| host.clone())
}

/// The library call that loads a cut and its audio, as the scripts' own preloads do; an empty
/// audio name passes the number the library reads as "no audio".
pub fn cut_load_call(phase: u16, cut: &SceneCut) -> StoryCall {
    let audio = if cut.audio.is_empty() {
        StoryCallArg::Number(EVENT_CUT_LOAD_NO_AUDIO_NUMBER.to_bits())
    } else {
        StoryCallArg::Text(cut.audio.as_bytes().to_vec())
    };
    StoryCall {
        phase,
        pack: 0,
        function: EVENT_CUT_LOAD_FUNCTION.to_owned(),
        args: vec![StoryCallArg::Text(cut.event.as_bytes().to_vec()), audio],
    }
}

/// Whether `call`, a cut load the scene follow handed to the relay queue, is stale now; none for
/// any other call, and for a load whose cut is still missing outside a scene.
pub fn stale_cut_load(
    call: &StoryCall,
    local: &LocalCut,
    local_in_scene: bool,
) -> Option<StaleCutLoad> {
    if call.function != EVENT_CUT_LOAD_FUNCTION {
        return None;
    }
    if local_in_scene {
        return Some(StaleCutLoad::OwnSceneRuns);
    }
    let Some(StoryCallArg::Text(event)) = call.args.first() else {
        return None;
    };
    let loaded_here =
        local.loaded && local.event.as_deref().map(str::as_bytes) == Some(event.as_slice());
    loaded_here.then_some(StaleCutLoad::CutLoadedHere)
}
