use std::io::{BufRead};
use std::collections::{HashMap};

use crate::common::{
  BlockCoords,
  Direction
};
use crate::common::{
  i32_from_str,
  coord_from_str,
  realm_from_str,
  create_reader
};


pub fn build_ties_map(ties_path: &String) ->
  HashMap<BlockCoords, (BlockCoords, Direction, Direction)> {

  let mut ties_map: HashMap<BlockCoords, (BlockCoords, Direction, Direction)> = HashMap::new();

  let reader = create_reader(ties_path);

  for line_result in reader.lines() {
    if let Ok(line) = line_result {
      let split: Vec<&str> = line.split("\t").collect();

      match split[..] {
        [
          from_x_str,
          from_y_str,
          from_z_str,
          from_realm_str,
          from_direction_str,
          to_x_str,
          to_y_str,
          to_z_str,
          to_realm_str,
          to_direction_str
        ] => {
          let from_block_coords = (
            coord_from_str(from_x_str),
            coord_from_str(from_y_str),
            coord_from_str(from_z_str),
            realm_from_str(from_realm_str)
          );

          let to_block_coords = (
            coord_from_str(to_x_str),
            coord_from_str(to_y_str),
            coord_from_str(to_z_str),
            realm_from_str(to_realm_str)
          );

          let from_direction = Direction::direction_from_str(from_direction_str);
          let to_direction = Direction::direction_from_str(to_direction_str);
          
          ties_map.insert(from_block_coords, (to_block_coords, from_direction, to_direction));
        },
        _ => {
          exit!("Error reading from ties file {:?}, line: {}", ties_path, line);
        }
      }
    } else {
      exit!("Error reading line from ties file {:?}", ties_path);
    }
  }

  ties_map
}


pub fn build_weights_map(weights_path: &String) -> HashMap<BlockCoords, i32> {
  let mut weights_map: HashMap<BlockCoords, i32> = HashMap::new();

  let reader = create_reader(weights_path);

  for line_result in reader.lines() {
    if let Ok(line) = line_result {
      let split: Vec<&str> = line.split("\t").collect();

      match split[..] {
        [
          x_str,
          y_str,
          z_str,
          realm_str,
          weight_str
        ] => {
          let block_coords = (
            coord_from_str(x_str),
            coord_from_str(y_str),
            coord_from_str(z_str),
            realm_from_str(realm_str),
          );

          let weight = i32_from_str(weight_str, "weight");
          
          weights_map.insert(block_coords, weight);
        },
        _ => {
          exit!("Error reading from weights file {:?}, line: {}", weights_path, line);
        }
      }
    } else {
      exit!("Error reading line from weights file {:?}", weights_path);
    }
  }

  weights_map
}
