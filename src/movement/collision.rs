use crate::rendering::block::{Block, BlockTexture};
use cgmath::{Point3, Vector3};
use std::collections::HashSet;

pub fn check_player_block(
    player_position: Point3<f32>,
    movement: Vector3<f32>,
    blocks: &HashSet<Block>,
) -> (bool, bool, bool) {
    let future_position = player_position + movement;
    let future_block_position = Point3 {
        x: future_position.x as u8,
        y: (future_position.y - 2.5) as u8,
        z: future_position.z as u8,
    };
    if !containts_any_block(future_block_position, blocks) {
        return (false, false, false);
    }

    (
        future_block_position.x != player_position.x as u8,
        future_block_position.y != player_position.y as u8,
        future_block_position.z != player_position.z as u8,
    )
}

fn containts_any_block(block_position: Point3<u8>, blocks: &HashSet<Block>) -> bool {
    let mut block = Block::new(block_position.x, block_position.y, block_position.z);
    block.texture = BlockTexture::Moss;
    if blocks.contains(&block) {
        return true;
    }
    block.texture = BlockTexture::Dirt;
    if blocks.contains(&block) {
        return true;
    }
    block.texture = BlockTexture::Stone;
    if blocks.contains(&block) {
        return true;
    }
    false
}
