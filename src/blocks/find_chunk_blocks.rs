use std::collections::HashMap;

use zune_inflate::DeflateDecoder;

use crate::common::{ChunkCoords, Block, RegionCoords, Region};
use crate::common::{
  chunk_coords_to_region_coords,
  read_three_byte_size,
  read_four_byte_size
};

use crate::blocks::region_cache::update_region_cache;

use crate::blocks::nbt_blocks::find_chunk_nbt_blocks;


pub fn find_chunk_blocks(
  chunk_coords: ChunkCoords,
  chunk_num: u32,
  world_dir: &String,
  region_cache: &mut HashMap<RegionCoords, Region>
) -> Vec<Block> {
  let region_coords = chunk_coords_to_region_coords(chunk_coords);

  update_region_cache(
    region_coords,
    chunk_num,
    world_dir,
    region_cache
  );

  match region_cache.get(&region_coords) {
    Some(region) => {
      if let Some(chunk_nbt) = extract_chunk_nbt(chunk_coords, &region.data) {
        find_chunk_nbt_blocks(chunk_coords, chunk_nbt)
      } else {
        vec![]
      }
    },
    None => {
      vec![]
    }
  }    
}


fn extract_chunk_nbt(chunk_coords: ChunkCoords, region_data: &Vec<u8>) -> Option<Vec<u8>> {
  let (chunk_x, chunk_z, _) = chunk_coords;
  let loc_offset = 4 * (chunk_x.rem_euclid(32) + 32 * chunk_z.rem_euclid(32)) as usize;

  let chunk_offset = 4096 * read_three_byte_size(region_data, loc_offset);

  let chunk_length = read_four_byte_size(region_data, chunk_offset);

  let region_data_start = chunk_offset + 5;

  if chunk_offset != 0 {
    let compressed_chunk_nbt = &region_data[region_data_start..region_data_start + chunk_length];
    let mut decoder = DeflateDecoder::new(compressed_chunk_nbt);
    let chunk_nbt = decoder.decode_zlib().unwrap();
    return Some(chunk_nbt.to_vec());
  }

  None
}
