use crate::rendering::block::{Block, BlockTexture};
use noise::{NoiseFn, Perlin};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

pub mod chunk;

const BASE_HEIGTH: f64 = 64.;
const HEIGTH_BIAS: f64 = 0.1;
const TERRAIN_SCALING_FACTOR: (f64, f64, f64) = (16., 16., 16.);

pub fn generate_terrain(start: (i32, i32, i32), end: (i32, i32, i32), seed: u32) -> HashSet<Block> {
    let blocks = Arc::new(Mutex::new(HashSet::new()));
    let perlin = Perlin::new(seed);

    for x in start.0..end.0 {
        let blocks = blocks.clone();
        for y in start.1..end.1 {
            for z in start.2..end.2 {
                let mut block = Block::new(
                    (x - start.0).try_into().unwrap(),
                    (y - start.1).try_into().unwrap(),
                    (z - start.2).try_into().unwrap(),
                );
                let add_block = check_block((x, y, z), perlin);

                if !add_block {
                    continue;
                }

                let block_left = check_block((x - 1, y, z), perlin);
                let block_right = check_block((x + 1, y, z), perlin);
                let block_under = check_block((x, y - 1, z), perlin);
                let block_above = check_block((x, y + 1, z), perlin);
                let block_before = check_block((x, y, z - 1), perlin);
                let block_after = check_block((x, y, z + 1), perlin);

                // If a block beside this block is empty, add that face to visible_faces
                if !block_left {
                    block.visible_faces ^= 0b100000
                }
                if !block_right {
                    block.visible_faces ^= 0b010000
                }
                if !block_under {
                    block.visible_faces ^= 0b001000
                }
                if !block_above {
                    block.visible_faces ^= 0b000100
                }
                if !block_before {
                    block.visible_faces ^= 0b000010
                }
                if !block_after {
                    block.visible_faces ^= 0b000001
                }

                block.texture = if !block_above {
                    BlockTexture::Moss
                } else if !check_block((x, y + 2, z), perlin) {
                    BlockTexture::Dirt
                } else {
                    BlockTexture::Stone
                };

                blocks.lock().unwrap().insert(block);
            }
        }
    }
    let blocks = blocks.lock().unwrap().clone();

    blocks
}

fn check_block(position: (i32, i32, i32), perlin: Perlin) -> bool {
    let (x, y, z) = position;
    let mut density = perlin.get([
        x as f64 / TERRAIN_SCALING_FACTOR.0,
        y as f64 / TERRAIN_SCALING_FACTOR.1,
        z as f64 / TERRAIN_SCALING_FACTOR.2,
    ]);

    if y > BASE_HEIGTH as i32 {
        density -= (y as f64 - BASE_HEIGTH) * HEIGTH_BIAS;
    }
    density > 0.
}
