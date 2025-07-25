use std::error::Error;

mod rendering;
mod terrain;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Running... ");
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let seed: u32 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u32;
    // let seed = seed & 255;

    let mut window_state = rendering::StateApplication::new(seed);
    event_loop.run_app(&mut window_state)?;
    Ok(())
}
