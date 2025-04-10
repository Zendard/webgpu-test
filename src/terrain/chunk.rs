use crate::rendering::block::Block;

pub const CHUNK_SIZE: u32 = 32;

#[derive(Debug)]
pub struct Chunk {
    #[allow(unused)]
    pub position: (i32, i32),
    pub blocks: Vec<Block>,
}

impl Chunk {
    pub fn new(position: (i32, i32), seed: u32) -> Self {
        let start = (
            position.0 * CHUNK_SIZE as i32,
            0,
            position.1 * CHUNK_SIZE as i32,
        );
        let end = (
            (position.0 + 1) * CHUNK_SIZE as i32,
            128,
            (position.1 + 1) * CHUNK_SIZE as i32,
        );

        let blocks = super::generate_terrain(start, end, seed)
            .iter()
            .copied()
            .collect();

        Self { position, blocks }
    }
}
