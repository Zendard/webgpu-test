use crate::rendering::block::Block;
use noise::{NoiseFn, Perlin};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

pub mod chunk;

const BASE_HEIGTH: f64 = 64.;
const HEIGTH_BIAS: f64 = 0.1;
const TERRAIN_SCALING_FACTOR: f64 = 1.;

pub fn generate_terrain(start: (i32, i32, i32), end: (i32, i32, i32), seed: u32) -> HashSet<Block> {
    // println!("Generating terrain...");
    // let volume = ((end.0 - start.0) * (end.1 - start.1) * (end.2 - start.2)).unsigned_abs();
    let blocks = Arc::new(Mutex::new(HashSet::new()));
    let perlin = Perlin::new(seed);
    let size = (
        (end.0 - start.0).abs() as f64 / TERRAIN_SCALING_FACTOR,
        (end.1 - start.1).abs() as f64 / TERRAIN_SCALING_FACTOR,
        (end.2 - start.2).abs() as f64 / TERRAIN_SCALING_FACTOR,
    );

    // let mut handles = Vec::with_capacity((end.0 - start.0) as usize);
    for x in start.0..end.0 {
        let blocks = blocks.clone();
        // let handle = std::thread::spawn(move || {
        for y in start.1..end.1 {
            for z in start.2..end.2 {
                let mut block = Block::new(x, y, z);
                let add_block = check_block((x, y, z), perlin, size);

                if !add_block {
                    // return;
                    continue;
                }

                let block_left = check_block((x - 1, y, z), perlin, size);
                let block_right = check_block((x + 1, y, z), perlin, size);
                let block_under = check_block((x, y - 1, z), perlin, size);
                let block_above = check_block((x, y + 1, z), perlin, size);
                let block_before = check_block((x, y, z - 1), perlin, size);
                let block_after = check_block((x, y, z + 1), perlin, size);

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

                blocks.lock().unwrap().insert(block);
            }
        }
        // });
        // handles.push(handle);
    }
    // for handle in handles {
    // handle.join().unwrap();
    // }
    let blocks = blocks.lock().unwrap().clone();
    if blocks.len() >= crate::rendering::MAX_BLOCKS_IN_CHUNK as usize {
        println!("Oops! Blocks: {}", blocks.len());
    }

    blocks
}

fn check_block(position: (i32, i32, i32), perlin: Perlin, size: (f64, f64, f64)) -> bool {
    let (x, y, z) = position;
    let mut density = perlin.get([x as f64 / size.0, y as f64 / size.1, z as f64 / size.2]);

    if y > BASE_HEIGTH as i32 {
        density -= (y as f64 - BASE_HEIGTH) * HEIGTH_BIAS;
    }
    density > 0.
}
