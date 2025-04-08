use crate::rendering::block::Block;

use super::Instance;
use std::collections::HashSet;

pub fn blocks_to_instances(
    blocks: &HashSet<Block>,
    edge_blocks: &mut HashSet<Block>,
) -> Vec<Instance> {
    println!("Culling blocks...");
    // let mut blocks_culled = Vec::new();
    // for block in edge_blocks.clone() {
    //     let (x, y, z) = block.position;
    //     let block_left = Block::new(x - 1, y, z);
    //     let block_right = Block::new(x + 1, y, z);
    //     let block_under = Block::new(x, y - 1, z);
    //     let block_above = Block::new(x, y + 1, z);
    //     let block_before = Block::new(x, y, z - 1);
    //     let block_after = Block::new(x, y, z + 1);
    //
    //     let existing_blocks: Vec<&Block> = blocks
    //         .iter()
    //         .filter(|block| {
    //             **block == block_left
    //                 || **block == block_right
    //                 || **block == block_under
    //                 || **block == block_above
    //                 || **block == block_before
    //                 || **block == block_after
    //         })
    //         .collect();
    //
    //     if existing_blocks.len() < 6 {
    //         blocks_culled.push(block.as_instances());
    //         edge_blocks.remove(&block);
    //     }
    // }
    println!("Done");
    // blocks_culled
    //     .iter()
    //     .flat_map(|instance| *instance)
    //     .collect()
    blocks.iter().flat_map(Block::as_instances).collect()
}
