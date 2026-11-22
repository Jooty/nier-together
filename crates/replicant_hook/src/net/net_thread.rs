use std::io;
use std::ops::Deref;
use std::time::Instant;

use nt_hook::net::LinkPeer;
use nt_net::Transport;
use replicant_game::readiness::PlayReads;
use replicant_game::scale_up_prompt::{OpenAsk, PressSample, PromptStep};
use replicant_proto::{GameReadiness, Message, PlayerSettings, SessionOptions};

use crate::net::replicant_net::ReplicantNet;

pub struct NetThread {
    inner: nt_hook::net::NetThread<ReplicantNet>,
}

impl Deref for NetThread {
    type Target = nt_hook::net::NetThread<ReplicantNet>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NetThread {
    pub fn spawn<T>(transport: T, link_port: u16) -> io::Result<Self>
    where
        T: Transport + 'static,
    {
        nt_hook::net::NetThread::spawn(
            ReplicantNet::default(),
            transport,
            LinkPeer::launcher_on_port(link_port),
        )
        .map(|inner| Self { inner })
    }

    pub fn take_launcher_settings(&self) -> Option<PlayerSettings> {
        self.inner.update(|shared| shared.pending_settings.take())
    }

    pub fn tell_launcher_settings(&self, settings: PlayerSettings) {
        self.inner.send_to_launcher(Message::Settings(settings));
    }

    pub fn tell_launcher_host_options(&self, options: SessionOptions) {
        self.inner.update(|shared| shared.options = options);
        self.inner
            .send_to_launcher(Message::SessionOptions(options));
    }

    pub fn tell_launcher_saved(&self, succeeded: bool) {
        self.inner
            .send_to_launcher(Message::GameSaved { succeeded });
    }

    pub fn step_scale_up_prompt(&self, now_ms: u64, input: PressSample) -> PromptStep {
        let step = self.inner.update(|shared| {
            let safe = shared.safe_to_take_a_joiner();
            shared.scale_up.step(safe, now_ms, input)
        });
        if let Some(answer) = step.answer {
            self.inner.send_to_launcher(Message::ScaleUpAnswer(answer));
        }
        step
    }

    pub fn shown_scale_up_ask(&self) -> Option<OpenAsk> {
        self.inner.update(|shared| shared.scale_up.shown().cloned())
    }

    pub fn scale_up_input_masked(&self) -> bool {
        self.inner
            .update(|shared| shared.scale_up.wants_input_masked())
    }

    pub fn report_readiness(
        &self,
        readiness: GameReadiness,
        reads: PlayReads,
        safe_for_a_joiner: bool,
    ) {
        self.inner.update(|shared| {
            shared.local_readiness = readiness;
            shared.local_reads = reads;
            shared.local_safe_for_a_joiner = safe_for_a_joiner;
            shared.local_readiness_at = Some(Instant::now());
        });
    }
}
