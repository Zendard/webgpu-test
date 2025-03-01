use pollster::FutureExt;
use std::error::Error;

mod particle;
mod texture;
mod wgpu;

fn main() -> Result<(), Box<dyn Error>> {
    crate::wgpu::create_window().block_on()?;

    Ok(())
}
