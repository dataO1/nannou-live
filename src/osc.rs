// OSC placeholder — receives scene switch and parameter commands
// from DJ software. Wire up rosc when you're ready.

#[derive(Debug, Clone)]
pub enum OscCommand {
    Scene(String),
    Param { index: usize, value: f32 },
    Beat,
}

/// Spawn an OSC listener thread. Returns a receiver for OscCommands.
/// Placeholder: returns a dummy channel that never sends.
#[cfg(feature = "osc")]
pub fn spawn_listener() -> std::sync::mpsc::Receiver<OscCommand> {
    let (_tx, rx) = std::sync::mpsc::channel();
    // TODO: spawn thread with rosc::UdpSocket, parse OSC addresses:
    //   /nannou/scene <string>  → OscCommand::Scene
    //   /nannou/param <i> <f>   → OscCommand::Param
    //   /nannou/beat            → OscCommand::Beat
    log::info!("OSC listener placeholder — rosc not yet wired");
    rx
}
