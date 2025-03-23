use std::error::Error;

mod rendering;
mod texture;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    let mut window_state = rendering::StateApplication::default();
    event_loop.run_app(&mut window_state)?;
    let mut state = window_state.state.unwrap();
    Ok(())
}
