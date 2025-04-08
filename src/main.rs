use std::error::Error;

mod movement;
mod rendering;
mod terrain;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let terrain = terrain::generate_terrain((-10, 0, -10), (10, 100, 10), 674737572);

    let mut window_state = rendering::StateApplication::new(terrain);
    event_loop.run_app(&mut window_state)?;
    Ok(())
}
