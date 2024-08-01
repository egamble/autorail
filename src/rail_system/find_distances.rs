use std::collections::{HashMap};

use crate::common::{
  Block,
  BlockCoords,
  Direction,
  RailData,
  Station,
  Switch
};
use crate::common::{
  get_distance,
  set_distance,
  switch_node_id
};


pub fn find_distances(
  stations: &Vec<Station>,
  switches: &Vec<Switch>,
  rail_map: &HashMap<BlockCoords, Block>,
  ties_map: &HashMap<BlockCoords, (BlockCoords, Direction, Direction)>,
  weights_map: &HashMap<BlockCoords, i32>
) -> (Vec<i32>, Vec<BlockCoords>) {
  
  let num_stations = stations.len();
  let mut station_id_map: HashMap<BlockCoords, usize> = HashMap::new();
  for station_id in 0..num_stations {
    station_id_map.insert(stations[station_id].coords, station_id);
  }

  let num_switches = switches.len();
  let mut switch_id_map: HashMap<BlockCoords, usize> = HashMap::new();
  for switch_id in 0..num_switches {
    switch_id_map.insert(switches[switch_id].coords, switch_id);
  }

  let num_nodes = num_stations + 4 * num_switches;
  
  // Distances have type i32 instead of u32, because it's possible
  // to have negative distances when negative weights are used.
  let mut distances: Vec<i32> = vec![i32::MAX; num_nodes * num_nodes];

  let mut rail_system_coords: Vec<BlockCoords> = Vec::new();

  for from_station_id in 0..num_stations {
    let from_station = &stations[from_station_id];

    let (to_node_id, distance, mut rail_connection_coords) =
      find_connection(
        from_station.coords,
        from_station.direction,
        &station_id_map,
        &switch_id_map,
        rail_map,
        ties_map,
        weights_map,
        num_stations
      );

    rail_system_coords.append(&mut rail_connection_coords);

    // We subtract one from the distance for each end of the connection that is a switch node,
    // so that the distance to/from a switch node is less than the distance to/from the other
    // switch nodes on the same switch.
    // In this case the starting node is a station node, so we subtract either 0 or 1.
    let subtraction_distance = if is_switch_node(to_node_id, num_stations) {1} else {0};

    // Link the "from" station node to the node that was found to be connected to it,
    // with the found distance minus the subtraction distance.
    set_distance(&mut distances, num_nodes, from_station_id, to_node_id, distance - subtraction_distance);

    // Link the "from" station node to itself with zero distance.
    set_distance(&mut distances, num_nodes, from_station_id, from_station_id, 0);
  }

  for from_switch_id in 0..num_switches {
    let from_switch = &switches[from_switch_id];

    for from_direction_index in 0..4 { // NSWE
      if from_switch.has_directions[from_direction_index] {
        
        let (to_node_id, distance, mut rail_connection_coords) =
          find_connection(
            from_switch.coords,
            Direction::from_usize(from_direction_index),
            &station_id_map,
            &switch_id_map,
            rail_map,
            ties_map,
            weights_map,
            num_stations
          );

        rail_system_coords.append(&mut rail_connection_coords);

        // We subtract one from the distance for each end of the connection that is a switch node,
        // so that the distance to/from a switch node is less than the distance to/from the other
        // switch nodes on the same switch.
        // In this case the "from" node is a switch node, so we subtract either 1 or 2.
        let subtraction_distance = if is_switch_node(to_node_id, num_stations) {2} else {1};

        // Link the "from" switch node to the node that was found to be connected to it,
        // with the found distance minus the subtraction distance.
        let from_switch_node_id = switch_node_id(from_switch_id, from_direction_index, num_stations);
        set_distance(&mut distances, num_nodes, from_switch_node_id, to_node_id, distance - subtraction_distance);

        // Link the "from" switch node to the other switch nodes on the same switch with distance 2,
        // to compensate for the subtractions for starting or ending at a switch node.
        for to_direction_index in 0..4 { // NSWE
          if to_direction_index != from_direction_index {
            if from_switch.has_directions[to_direction_index] {

              let to_switch_node_id = switch_node_id(from_switch_id, to_direction_index, num_stations);
              set_distance(&mut distances, num_nodes, from_switch_node_id, to_switch_node_id, 2);
            }
          }
        }

        // Link the "from" switch node to itself with zero distance.
        set_distance(&mut distances, num_nodes, from_switch_node_id, from_switch_node_id, 0);
      }
    }
  }

  floyd_warshall(&mut distances, num_nodes);

  (distances, rail_system_coords)
}


fn is_switch_node(node_id: usize, num_stations: usize) -> bool {
  node_id >= num_stations
}


fn floyd_warshall(distances: &mut Vec<i32>, num_nodes: usize) {
  // The Floyd-Warshall algorithm is used to fill in the shortest distances for nodes
  // that are not directly connected.
  // https://en.wikipedia.org/wiki/Floyd%E2%80%93Warshall_algorithm

  for k in 0..num_nodes {
    for i in 0..num_nodes {
      for j in 0..num_nodes {
        let ij_dist = get_distance(distances, num_nodes, i, j);
        let ik_dist = get_distance(distances, num_nodes, i, k);
        let kj_dist = get_distance(distances, num_nodes, k, j);


        if ik_dist < i32::MAX && kj_dist < i32::MAX {
          let sum_dist = ik_dist + kj_dist;
          if sum_dist < ij_dist {
            set_distance(distances, num_nodes, i, j, sum_dist);
          }
        }
      }
    }
  }
}


fn find_connection(
  start_coords: BlockCoords,
  start_direction: Direction,
  station_id_map: &HashMap<BlockCoords, usize>,
  switch_id_map: &HashMap<BlockCoords, usize>,
  rail_map: &HashMap<BlockCoords, Block>,
  ties_map: &HashMap<BlockCoords, (BlockCoords, Direction, Direction)>,
  weights_map: &HashMap<BlockCoords, i32>,
  num_stations: usize
) -> (usize, i32, Vec<BlockCoords>) {
  let mut coords = start_coords;
  let mut direction = start_direction;
  let mut distance: i32 = 0;

  let mut rail_connection_coords: Vec<BlockCoords> = vec![start_coords];

  loop {
    let prev_direction = direction;

    (coords, direction) = find_next_rail_block(
      coords,
      direction,
      rail_map,
      ties_map
    );

    rail_connection_coords.push(coords);

    distance += 1;
    if let Some(weight) = weights_map.get(&coords) {
      distance += weight;
    }

    if let Some(to_station_id) = station_id_map.get(&coords) {
      return (*to_station_id, distance, rail_connection_coords);
    }

    if let Some(to_switch_id) = switch_id_map.get(&coords) {
      
      // The previous direction is the direction going into the switch.
      // We use the previous direction as the switch direction to compute the "to" switch node ID
      // rather than the current direction, because the center rail block of the switch might have
      // changed the incoming direction.
      // We use the opposite direction (of the previous direction) to compute the "to" switch node ID,
      // because switch nodes' directions are labeled as if traveling away from the switch.
      
      let to_switch_node_id = switch_node_id(*to_switch_id, prev_direction.opposite_direction() as usize, num_stations);
      return (to_switch_node_id, distance, rail_connection_coords);
    }
  }
}


// Returns tuples of (<new rail data> <new direction> <new y offset>) to check.
fn blocks_to_check(current_rail_data: RailData, current_direction: Direction) ->
  Option<Vec<(RailData, Direction, i32)>> {

    let regular_north = vec![
      (RailData::NS, Direction::N, 0),
      (RailData::AN, Direction::N, 0),
      (RailData::AS, Direction::N, -1),
      (RailData::SE, Direction::E, 0),
      (RailData::SW, Direction::W, 0),
      (RailData::NW, Direction::N, 0),
      (RailData::NE, Direction::N, 0),
    ];
    
    let regular_south = vec![
      (RailData::NS, Direction::S, 0),
      (RailData::AN, Direction::S, -1),
      (RailData::AS, Direction::S, 0),
      (RailData::SE, Direction::S, 0),
      (RailData::SW, Direction::S, 0),
      (RailData::NW, Direction::W, 0),
      (RailData::NE, Direction::E, 0),
    ];
    
    
    let regular_west = vec![
      (RailData::EW, Direction::W, 0),
      (RailData::AE, Direction::W, -1),
      (RailData::AW, Direction::W, 0),
      (RailData::SE, Direction::S, 0),
      (RailData::SW, Direction::W, 0),
      (RailData::NW, Direction::W, 0),
      (RailData::NE, Direction::N, 0),
    ];
    
    let regular_east = vec![
      (RailData::EW, Direction::E, 0),
      (RailData::AE, Direction::E, 0),
      (RailData::AW, Direction::E, -1),
      (RailData::SE, Direction::E, 0),
      (RailData::SW, Direction::S, 0),
      (RailData::NW, Direction::N, 0),
      (RailData::NE, Direction::E, 0),
    ];
    
    if current_rail_data.is_curved() {
      // We ignore the shape of the curved rail block, because we may be leaving
      // a switch curved rail, where the current direction is unrelated to the shape.
      return Some(
        match current_direction {
          Direction::N => regular_north,
          Direction::S => regular_south,
          Direction::W => regular_west,
          Direction::E => regular_east
        }
      );
    }

    let blocks_to_check = match (current_rail_data, current_direction) {
  
      // straight, non-ascending rail

      (RailData::NS, Direction::N) => regular_north,
      (RailData::NS, Direction::S) => regular_south,
      
      (RailData::EW, Direction::W) => regular_west,
      (RailData::EW, Direction::E) => regular_east,

      // ascending rail

      (RailData::AE, Direction::W) => regular_west,
      (RailData::AW, Direction::E) => regular_east,
      (RailData::AN, Direction::S) => regular_south,
      (RailData::AS, Direction::N) => regular_north,

      (RailData::AE, Direction::E) => vec![
        (RailData::EW, Direction::E, 1),
        (RailData::AE, Direction::E, 1),
        (RailData::SE, Direction::E, 1),
        (RailData::SW, Direction::S, 1),
        (RailData::NW, Direction::N, 1),
        (RailData::NE, Direction::E, 1),
      ],

      (RailData::AW, Direction::W) => vec![
        (RailData::EW, Direction::W, 1),
        (RailData::AW, Direction::W, 1),
        (RailData::SE, Direction::S, 1),
        (RailData::SW, Direction::W, 1),
        (RailData::NW, Direction::W, 1),
        (RailData::NE, Direction::N, 1),
      ],

      (RailData::AN, Direction::N) => vec![
        (RailData::NS, Direction::N, 1),
        (RailData::AN, Direction::N, 1),
        (RailData::SE, Direction::E, 1),
        (RailData::SW, Direction::W, 1),
        (RailData::NW, Direction::N, 1),
        (RailData::NE, Direction::N, 1),
      ],

      (RailData::AS, Direction::S) => vec![
        (RailData::NS, Direction::S, 1),
        (RailData::AS, Direction::S, 1),
        (RailData::SE, Direction::S, 1),
        (RailData::SW, Direction::S, 1),
        (RailData::NW, Direction::W, 1),
        (RailData::NE, Direction::E, 1),
      ],

      _ => {
        return None;
      },
    };

    Some(blocks_to_check)
}


fn find_next_rail_block(
  coords: BlockCoords,
  direction: Direction,
  rail_map: &HashMap<BlockCoords, Block>,
  ties_map: &HashMap<BlockCoords, (BlockCoords, Direction, Direction)>
) -> (BlockCoords, Direction) {

  if let Some((tie_to_coords, tie_from_direction, tie_to_direction)) = ties_map.get(&coords) {
    if direction == *tie_from_direction {
      return (*tie_to_coords, *tie_to_direction);
    }
  }

  if let Some(current_rail_block) = rail_map.get(&coords) {
    let current_rail_data = current_rail_block.rail_data;

    if let Some(blocks_to_check) = blocks_to_check(current_rail_data, direction) {

      let (x, y, z, realm) = coords;

      let (new_x, new_z) = match direction {
        Direction::N => (x, z - 1),
        Direction::S => (x, z + 1),
        Direction::W => (x - 1, z),
        Direction::E => (x + 1, z),
      };
      
      for block_to_check in blocks_to_check {
        let (new_rail_data, new_direction, y_offset) = block_to_check;
        //
        let new_coords = (new_x, y + y_offset, new_z, realm);
        
        if let Some(new_rail_block) = rail_map.get(&new_coords) {
          if new_rail_block.is_rail() && new_rail_block.rail_data == new_rail_data {
            return (new_coords, new_direction);
          }
        }
      }
    } else {
      panic!("Unexpected rail data {:?} and direction {:?} at {:?}", current_rail_data, direction, coords);
    }
  }

  exit!("Couldn't find a rail connection from: {:?}", coords);
}
