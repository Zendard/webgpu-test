use std::error::Error;

use rendering::Instance;

mod physics;
mod rendering;
mod texture;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    let mut window_state = rendering::StateApplication::default();
    event_loop.run_app(&mut window_state)?;
    let mut state = window_state.state.unwrap();
    // state.add_instance(Instance::new(0.0, 0.0, 0.0, None));
    state.add_instance(Instance::new(1.0, 1.0, 0.0, None));
    // state.add_instance(Instance::new(2.0, 2.0, 0.0, None));
    Ok(())
}
