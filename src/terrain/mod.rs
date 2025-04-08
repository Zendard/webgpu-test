use crate::rendering::block::Block;
use noise::{NoiseFn, Perlin};
use std::collections::{HashMap, HashSet};

const BASE_HEIGTH: f64 = 64.;
const HEIGTH_BIAS: f64 = 1.;
const TERRAIN_SCALING_FACTOR: f64 = 4.;
const EDGE_DENSITY: f64 = 1.;

pub fn generate_terrain(
    start: (i32, i32, i32),
    end: (i32, i32, i32),
    seed: u32,
) -> (HashSet<Block>, HashSet<Block>) {
    println!("Generating terrain...");
    let size = ((end.0 - start.0) * (end.1 - start.1) * (end.2 - start.2)).unsigned_abs();
    let mut densities = HashMap::with_capacity(size as usize);
    let mut edge_blocks = HashSet::new();
    let perlin = Perlin::new(seed);

    for x in start.0..end.0 {
        for y in start.1..end.1 {
            for z in start.2..end.2 {
                let block = Block::new(x, y, z);
                let size = (
                    (end.0 - start.0).abs() as f64 / TERRAIN_SCALING_FACTOR,
                    (end.1 - start.1).abs() as f64 / TERRAIN_SCALING_FACTOR,
                    (end.2 - start.2).abs() as f64 / TERRAIN_SCALING_FACTOR,
                );
                let mut density =
                    perlin.get([x as f64 / size.0, y as f64 / size.1, z as f64 / size.2]);

                if block.position.1 > BASE_HEIGTH as i32 {
                    density -= (block.position.1 as f64 - BASE_HEIGTH) * HEIGTH_BIAS;
                }

                densities.insert(block, density);

                if density < EDGE_DENSITY && density > 0. {
                    edge_blocks.insert(block);
                }
            }
        }
    }

    let blocks: HashSet<Block> = densities
        .iter()
        .filter_map(
            |(block, density)| {
                if *density > 0. {
                    Some(*block)
                } else {
                    None
                }
            },
        )
        .collect();

    println!(
        "Done, blocks: {}, edge blocks: {}",
        blocks.len(),
        edge_blocks.len()
    );
    (blocks, edge_blocks)
}
