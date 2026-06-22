mod audio;
mod state;
mod sketch;
mod sketches;
#[cfg(feature = "osc")]
mod osc;

use state::LiveState;

fn main() {
    env_logger::init();
    nannou::app(LiveState::model).update(LiveState::update).run();
}
