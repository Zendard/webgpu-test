use std::error::Error;

use pollster::FutureExt;

mod window;

fn main() -> Result<(), Box<dyn Error>> {
    crate::window::create_window().block_on()?;

    Ok(())
}
