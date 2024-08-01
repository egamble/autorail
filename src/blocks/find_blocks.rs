use std::collections::{HashSet, HashMap};

use crate::common::{
  RegionCoords,
  ChunkCoords,
  BlockCoords,
  Direction,
  Region,
  Block,
  BlockID,
};
use crate::common::{block_coords_to_chunk_coords};

use crate::blocks::find_chunk_blocks::find_chunk_blocks;


pub fn find_blocks(
  starting_chunk_coords: ChunkCoords,
  world_dir: &String,
  ties_map: &HashMap<BlockCoords, (BlockCoords, Direction, Direction)>
) -> (Vec<Block>, Vec<(ChunkCoords, usize)>) {
  let mut all_blocks: Vec<Block> = Vec::new();

  let mut chunks: Vec<(ChunkCoords, usize)> = Vec::new();

  let mut chunk_coords_to_process = vec![starting_chunk_coords];

  let mut processed_chunk_coords: HashSet<ChunkCoords> = HashSet::new();

  let mut chunk_num: u32 = 0;

  let mut region_cache: HashMap<RegionCoords, Region> = HashMap::new();
  
  while let Some(chunk_coords) = chunk_coords_to_process.pop() {
    if !processed_chunk_coords.contains(&chunk_coords) {
      processed_chunk_coords.insert(chunk_coords);
      
      chunk_num += 1;

      let mut blocks = find_chunk_blocks(
        chunk_coords,
        chunk_num,
        world_dir,
        &mut region_cache
      );

      if blocks.len() > 0 {
        chunks.push((chunk_coords, blocks.len()));
      }

      let mut more_chunk_coords_to_process = find_more_chunk_coords_to_process(&blocks, &ties_map);
      chunk_coords_to_process.append(&mut more_chunk_coords_to_process);
    
      all_blocks.append(&mut blocks);
    }
  }
  
  (all_blocks, chunks)
}


fn is_block_at_chunk_edge(block: &Block) -> (bool, bool, bool, bool) {
  let (x, _, z, _) = block.coords;

  // Booleans indicating that the block is on the (north, south, west, east) edge of a chunk.
  (z.rem_euclid(16) == 0, z.rem_euclid(16) == 15, x.rem_euclid(16) == 0, x.rem_euclid(16) == 15)
}


fn find_more_chunk_coords_to_process(
  blocks: &Vec<Block>,
  ties_map: &HashMap<BlockCoords, (BlockCoords, Direction, Direction)>
) -> Vec<ChunkCoords> {

  let mut more_chunk_coords_to_process: Vec<ChunkCoords> = Vec::new();

  let mut sign_block_coords: HashSet<BlockCoords> = HashSet::new();
  
  for block in blocks {
    if block.is_sign() {
      sign_block_coords.insert(block.coords);
    }
  }

  for block in blocks {
    if !block.is_rail() {
      continue;
    }

    let (block_x, block_y, block_z, realm) = block.coords;

    // Add the adjacent chunk when a rail block is at the corresponding edge of the chunk,
    // and is either a straight rail oriented perpendicularly to the edge, or
    // is any curved rail. The adjacent chunk is added when any curved rail block is at
    // the edge of the chunk, even if the curved rail is not oriented toward the edge,
    // because the curved rail could be part of a switch and thus might switch toward
    // the edge.

    let (chunk_x, chunk_z, _) = block_coords_to_chunk_coords(block.coords);

    let (n, s, w, e) = is_block_at_chunk_edge(&block);

    if n && (block.is_curved_rail() || block.is_north_south_rail()) {
      more_chunk_coords_to_process.push((chunk_x, chunk_z - 1, realm));
    }
    if s && (block.is_curved_rail() || block.is_north_south_rail()) {
      more_chunk_coords_to_process.push((chunk_x, chunk_z + 1, realm));
    }
    if w && (block.is_curved_rail() || block.is_east_west_rail()) {
      more_chunk_coords_to_process.push((chunk_x - 1, chunk_z, realm));
    }
    if e && (block.is_curved_rail() || block.is_east_west_rail()) {
      more_chunk_coords_to_process.push((chunk_x + 1, chunk_z, realm));
    }

    // If the coordinates of a rail block match the "from" coordinates of a
    // tie in the ties file, add the chunk that contains the "to" coordinates.

    if let Some((to_block_coords, _, _)) = ties_map.get(&block.coords) {
      more_chunk_coords_to_process.push(block_coords_to_chunk_coords(*to_block_coords));
    }

    // If the rail block is straight and unpowered, check if there are wall sign blocks
    // immediately above the rail block and also immediately above that. If so, there's
    // a good chance that the rail block is the center of a station, and consequently we
    // will add several of the chunks around the current chunk, to make sure we have
    // all of the wall signs that are associated with the station. We'll add two rings
    // of chunks around the current chunk, except for the four chunks at the corners of
    // the outer ring. That's 5 X 5 chunks, minus the current chunk and minus the four
    // outer corners, so 20 more chunks.

    if  block.id == BlockID::UnpoweredRail &&
      block.is_straight_rail() &&
      sign_block_coords.contains(&(block_x, block_y + 1, block_z, realm)) && // sign immediately above the rail block
      sign_block_coords.contains(&(block_x, block_y + 2, block_z, realm)) {  // another sign above that one
        for chunk_x_offset in -2i32..=2 {
          for chunk_z_offset in -2i32..=2 {
            if !((chunk_x_offset.abs() == 2 && // exclude corner chunks
                  chunk_z_offset.abs() == 2) ||
                 (chunk_x_offset == 0 && // exclude current chunk
                  chunk_z_offset == 0)) {
              more_chunk_coords_to_process.push(
                (
                  chunk_x + chunk_x_offset,
                  chunk_z + chunk_z_offset,
                  realm,
                )
              );
            }
          }
        }
      }
  }

  more_chunk_coords_to_process
}
