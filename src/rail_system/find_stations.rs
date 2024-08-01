use std::collections::{HashMap};

use crate::common::{
  Block,
  BlockID,
  BlockCoords,
  SignData,
  Station,
};


fn push_station(stations: &mut Vec<Station>, rail_block: &Block, name_sign: &Block) {
  stations.push(
    Station {
      coords: rail_block.coords,
      name: name_sign.sign_text.to_string(),
      direction: name_sign.sign_data.to_direction(),
    }
  )
}


pub fn find_stations(blocks: &Vec<Block>, rail_map: &HashMap<BlockCoords, Block>, sign_map: &HashMap<BlockCoords, Block>) -> Vec<Station> {
  let mut stations: Vec<Station> = Vec::new();
  
  for block in blocks {
    if !block.is_rail() {
      continue;
    }

    let (x, y, z, realm) = block.coords;

    if  block.id == BlockID::UnpoweredRail &&
      block.is_straight_rail() {
        if let Some(name_sign) = sign_map.get(&(x, y + 2, z, realm)) {
          if let Some(launch_sign) = sign_map.get(&(x, y + 1, z, realm)) {

            if let Some(side_sign_1) = sign_map.get(&(x + 1, y + 1, z, realm)) {
              if let Some(side_sign_2) = sign_map.get(&(x - 1, y + 1, z, realm)) {

                if block.is_north_south_rail() &&
                  name_sign.sign_data == launch_sign.sign_data &&
                  name_sign.sign_data == side_sign_1.sign_data &&
                  name_sign.sign_data == side_sign_2.sign_data {

                    if name_sign.sign_data == SignData::N {
                      if let Some(detector_rail) = rail_map.get(&(x, y, z - 1, realm)) {
                        if detector_rail.id == BlockID::DetectorRail {
                          push_station(&mut stations, block, name_sign);
                          continue;
                        }
                      }
                    }

                    if name_sign.sign_data == SignData::S {
                      if let Some(detector_rail) = rail_map.get(&(x, y, z + 1, realm)) {
                        if detector_rail.id == BlockID::DetectorRail {
                          push_station(&mut stations, block, name_sign);
                          continue;
                        }
                      }
                    }
                 }
              }
            }
            
            if let Some(side_sign_1) = sign_map.get(&(x, y + 1, z + 1, realm)) {
              if let Some(side_sign_2) = sign_map.get(&(x, y + 1, z - 1, realm)) {

                if block.is_east_west_rail() &&
                  name_sign.sign_data == launch_sign.sign_data &&
                  name_sign.sign_data == side_sign_1.sign_data &&
                  name_sign.sign_data == side_sign_2.sign_data {

                    if name_sign.sign_data == SignData::W {
                      if let Some(detector_rail) = rail_map.get(&(x - 1, y, z, realm)) {
                        if detector_rail.id == BlockID::DetectorRail {
                          push_station(&mut stations, block, name_sign);
                          continue;
                        }
                      }
                    }

                    if name_sign.sign_data == SignData::E {
                      if let Some(detector_rail) = rail_map.get(&(x + 1, y, z, realm)) {
                        if detector_rail.id == BlockID::DetectorRail {
                          push_station(&mut stations, block, name_sign);
                          continue;
                        }
                      }
                    }
                  }
              }
            }
          }
        }
      }
  }

  stations.sort_by(|a, b| {
    let name_a = a.name.to_lowercase();
    let name_b = b.name.to_lowercase();

    name_a.cmp(&name_b)
  });

  stations
}
