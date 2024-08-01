use std::collections::{HashMap};

use crate::common::{
  Block,
  BlockID,
  BlockCoords,
  Direction,
  Switch
};


pub fn find_switches(blocks: &Vec<Block>, rail_map: &HashMap<BlockCoords, Block>) -> Vec<Switch> {
  let mut switches: Vec<Switch> = Vec::new();
  
  for block in blocks {
    if  block.id == BlockID::UnpoweredRail &&
      block.is_curved_rail() {
        let mut has_directions = [false; 4]; // NSWE
        let mut num_directions = 0;

        let (x, y, z, realm) = block.coords;

        if let Some(unpowered_rail) = rail_map.get(&(x, y, z - 1, realm)) {
          if let Some(detector_rail) = rail_map.get(&(x, y, z - 2, realm)) {
            if unpowered_rail.id == BlockID::UnpoweredRail &&
              detector_rail.id == BlockID::DetectorRail {
              has_directions[Direction::N as usize] = true;
              num_directions += 1;
            }
          }
        }

        if let Some(unpowered_rail) = rail_map.get(&(x, y, z + 1, realm)) {
          if let Some(detector_rail) = rail_map.get(&(x, y, z + 2, realm)) {
            if unpowered_rail.id == BlockID::UnpoweredRail &&
              detector_rail.id == BlockID::DetectorRail {
              has_directions[Direction::S as usize] = true;
              num_directions += 1;
            }
          }
        }

        if let Some(unpowered_rail) = rail_map.get(&(x - 1, y, z, realm)) {
          if let Some(detector_rail) = rail_map.get(&(x - 2, y, z, realm)) {
            if unpowered_rail.id == BlockID::UnpoweredRail &&
              detector_rail.id == BlockID::DetectorRail {
              has_directions[Direction::W as usize] = true;
              num_directions += 1;
            }
          }
        }

        if let Some(unpowered_rail) = rail_map.get(&(x + 1, y, z, realm)) {
          if let Some(detector_rail) = rail_map.get(&(x + 2, y, z, realm)) {
            if unpowered_rail.id == BlockID::UnpoweredRail &&
              detector_rail.id == BlockID::DetectorRail {
              has_directions[Direction::E as usize] = true;
              num_directions += 1;
            }
          }
        }

        if num_directions >= 3 {
          switches.push(
            Switch {
              coords: block.coords,
              has_directions: has_directions
            }
          )
        }
      }
  }

  switches
}
