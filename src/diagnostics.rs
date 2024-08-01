use std::collections::{HashMap, HashSet};

use crate::common::{
  Block,
  BlockCoords,
  ChunkCoords,
  Direction,
  Station,
  StationSign,
  Switch
};
use crate::common::{
  bool_to_int,
  create_writer,
  writeln_out,
  get_num_nodes,
  get_distance,
  realm_to_out_string,
  find_nearest_station_id
};


pub fn write_stations(stations: &Vec<Station>, out_path: &String) {
  let mut writer = create_writer(out_path);

  for station in stations {
    let (x, y, z, realm) = station.coords;

    let out_string = format!("{}\t{}\t{}\t{}\t{}\t{}",
                             x, y, z,
                             realm_to_out_string(realm),
                             station.direction as usize,
                             station.name
    );
    writeln_out(&mut writer, out_path, out_string);
  }
}


pub fn write_station_signs(stations: &Vec<Station>, station_signs: &Vec<StationSign>, out_path: &String) {
  let mut writer = create_writer(out_path);

  for station_sign in station_signs {
    let (x, y, z, realm) = station_sign.coords;

    let refers_to_station_id = station_sign.refers_to_station_id;
    let refers_to_station_name = &stations[refers_to_station_id].name;
        
    let out_string = format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.2}",
                             x, y, z,
                             realm_to_out_string(realm),
                             station_sign.belongs_to_station_id,
                             refers_to_station_id,
                             refers_to_station_name,
                             station_sign.nearest_num,
                             station_sign.distance
    );
    writeln_out(&mut writer, out_path, out_string);
  }
}


pub fn write_switches(switches: &Vec<Switch>, out_path: &String) {
  let mut writer = create_writer(out_path);

  for switch in switches {
    let (x, y, z, realm) = switch.coords;

    let out_string = format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                             x, y, z,
                             realm_to_out_string(realm),
                             bool_to_int(switch.has_directions[Direction::N as usize]),
                             bool_to_int(switch.has_directions[Direction::S as usize]),
                             bool_to_int(switch.has_directions[Direction::W as usize]),
                             bool_to_int(switch.has_directions[Direction::E as usize])
    );
    writeln_out(&mut writer, out_path, out_string);
  }
}


pub fn write_distances(distances: &Vec<i32>, out_path: &String) {
  let num_nodes = get_num_nodes(distances);

  let mut writer = create_writer(out_path);

  for i in 0..num_nodes {
    let mut row_string: String = "[".to_string();

    for j in 0..num_nodes {
      let distance =  get_distance(distances, num_nodes, i, j);

      row_string.push_str(
        &format!("{}{}",
                 if j > 0 {", "} else {""},
                 if distance == i32::MAX {"∞".to_string()} else {distance.to_string()}
        )
      );
    }

    row_string.push_str("]");

    writeln_out(&mut writer, out_path, row_string);
  }
}


pub fn write_rail_blocks(
  rail_system_coords: &Vec<BlockCoords>,
  rail_map: &HashMap<BlockCoords, Block>,
  out_path: &String
) {
  // dedupe rail system coords
  let mut coords_set: HashSet<BlockCoords> = HashSet::new();

  for coords in rail_system_coords {
    if !coords_set.contains(coords) {
      coords_set.insert(*coords);
    }
  }

  let mut writer = create_writer(out_path);

  for coords in coords_set {
    if let Some(rail_block) = rail_map.get(&coords) {

      let (x, y, z, realm) = rail_block.coords;
      
      let out_string = format!("{}\t{}\t{}\t{}\t{}\t{}",
                               x, y, z,
                               realm_to_out_string(realm),
                               rail_block.id as u32,
                               rail_block.rail_data as u32
      );
      writeln_out(&mut writer, out_path, out_string);
    }
  }
}


pub fn write_switches_nearest_station(
  switches: &Vec<Switch>,
  stations: &Vec<Station>,
  out_path: &String
) {
  let mut switches_nearest_station: Vec<(usize, usize, f64)> = Vec::new();

  for (switch_id, switch) in switches.iter().enumerate() {
    if let Some((nearest_station_id, nearest_distance)) = find_nearest_station_id(switch.coords, stations) {
      
      switches_nearest_station.push((switch_id, nearest_station_id, nearest_distance));
    }
  }

  switches_nearest_station.sort_by(|a, b| {
    let (_, _, nearest_distance_a) = a;
    let (_, _, nearest_distance_b) = b;

    nearest_distance_b.partial_cmp(nearest_distance_a).unwrap()
  });

  let mut writer = create_writer(out_path);

  for (switch_id, nearest_station_id, nearest_distance) in switches_nearest_station {
    let switch = &switches[switch_id];
    let nearest_station = &stations[nearest_station_id];

    let (x, y, z, realm) = switch.coords;
      
    let out_string = format!("{}\t{}\t{}\t{}\t{}\t{}\t{:.2}\t{}",
                             x, y, z,
                             realm_to_out_string(realm),
                             switch_id,
                             nearest_station_id,
                             nearest_distance,
                             nearest_station.name
    );
    writeln_out(&mut writer, out_path, out_string);
  }
}


pub fn write_chunks(
  chunks: &Vec<(ChunkCoords, usize)>,
  out_path: &String
) {
  let mut writer = create_writer(out_path);

  for (chunk_coords, num_blocks) in chunks {

    let (x, z, realm) = chunk_coords;
      
    let out_string = format!("{}\t{}\t{}\t{}",
                             x, z,
                             realm_to_out_string(*realm),
                             num_blocks
    );
    writeln_out(&mut writer, out_path, out_string);
  }
}
