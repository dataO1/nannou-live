// nannou-live — audio-reactive live visuals with scene switching.
// Architecture: Model-View-Controller with sketch trait system.
// Inspired by Xtal's sketch runtime pattern.

mod state;
mod audio;
mod sketch;
mod sketches;

use state::LiveState;

fn main() {
    env_logger::init();
    nannou::app(LiveState::model).update(LiveState::update).run();
}
