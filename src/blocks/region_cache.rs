use std::io::Read;
use std::collections::HashMap;

use crate::common::{RegionCoords, Region, Realm};
use crate::common::{open_file};


const MAX_CACHED_REGIONS: usize = 10;


fn read_region_data(region_coords: RegionCoords, world_dir: &String) -> Option<Vec<u8>> {
  let (x, z, realm) = region_coords;

  let realm_dir = match realm {
    Realm::Overworld => "",
    Realm::Nether => "/DIM-1",
    Realm::End => "/DIM1",
  };
  
  let region_path = format!("{}{}/region/r.{}.{}.mca", world_dir, realm_dir, x, z);

  println!("{:?}", region_coords);
  
  let mut file = open_file(&region_path);
  let mut data = Vec::new();

  file.read_to_end(&mut data).ok();
  
  Some(data)
}


fn maybe_shrink_region_cache(region_cache: &mut HashMap<RegionCoords, Region>) {
  if region_cache.len() <= MAX_CACHED_REGIONS {
    return
  }

  let mut oldest_chunk_num = u32::MAX;
  let mut oldest_region_coords: Option<RegionCoords> = None;

  for (region_coords, region) in region_cache.iter() {
    if region.last_chunk_num < oldest_chunk_num {
      oldest_chunk_num = region.last_chunk_num;
      oldest_region_coords = Some(region_coords.clone());
    }
  }

  if let Some(region_coords) = oldest_region_coords {
    region_cache.remove(&region_coords);
  }
}


pub fn update_region_cache(
  region_coords: RegionCoords,
  chunk_num: u32,
  world_dir: &String,
  region_cache: &mut HashMap<RegionCoords, Region>
) {
  if let Some(region) = region_cache.get_mut(&region_coords) {
    region.last_chunk_num = chunk_num;
    return;
  }

  let data = read_region_data(region_coords, world_dir);
  if data == None {
    return;
  }

  let region = Region {
    coords: region_coords,
    data: data.unwrap(),
    last_chunk_num: chunk_num
  };

  region_cache.insert(region_coords, region);

  maybe_shrink_region_cache(region_cache);
}
