use std::collections::{HashMap};

use crate::common::{
  Block,
  Station,
  StationSign
};
use crate::common::{
  block_coords_distance,
  find_nearest_station_id
};


fn nearest_station_ids_matrix(stations: &Vec<Station>) -> Vec<Vec<(usize, f64)>> {
  let mut matrix: Vec<Vec<(usize, f64)>> = Vec::new();

  for station in stations {
    let mut nearest_station_ids: Vec<(usize, f64)> = Vec::new();

    for (nearest_station_id, nearest_station) in stations.iter().enumerate() {
      let distance = block_coords_distance(station.coords, nearest_station.coords);
      
      nearest_station_ids.push((nearest_station_id, distance));
    }

    nearest_station_ids.sort_by(|a, b| {
      let (_, dist_a) = a;
      let (_, dist_b) = b;

      dist_a.partial_cmp(dist_b).unwrap()
    });
    
    matrix.push(nearest_station_ids);
  }

  matrix
}


fn station_sign_nearest_num(sign_text: &String) -> usize {
  for token in sign_text.split(" ") {
    if let Some(possible_num) = token.to_lowercase().strip_prefix("n") {
      match possible_num.parse::<usize>() {
        Ok(n) => return n,
        Err(_) => continue,
      }
    }
  }

  0
}


pub fn find_station_signs(blocks: &Vec<Block>, stations: &Vec<Station>) -> Vec<StationSign> {
  let mut station_name_to_id: HashMap<&String, usize> = HashMap::new();

  for (station_id, station) in stations.iter().enumerate() {
    station_name_to_id.insert(&station.name, station_id);
  }

  let nearest_station_ids_matrix = nearest_station_ids_matrix(stations);

  let mut station_signs: Vec<StationSign> = Vec::new();

  for block in blocks {
    if block.is_sign() {
      if let Some((nearest_station_id, _)) = find_nearest_station_id(block.coords, stations) {
        let mut station_sign = StationSign {
          coords: block.coords,
          belongs_to_station_id: nearest_station_id,
          refers_to_station_id: 0,
          nearest_num: 0,
          distance: 0.0
        };

        if let Some(station_id) = station_name_to_id.get(&block.sign_text) {
          station_sign.refers_to_station_id = *station_id;
          station_signs.push(station_sign);
          continue;
        }

        station_sign.nearest_num = station_sign_nearest_num(&block.sign_text);

        if station_sign.nearest_num > 0 {
          (
            station_sign.refers_to_station_id,
            station_sign.distance
          ) =
            nearest_station_ids_matrix
            [station_sign.belongs_to_station_id]
            [station_sign.nearest_num];
          
          // If the "belongs to" and "refers to" stations are in different realms,
          // the distance will be f64::INFINITY and we won't keep the station sign.
          if station_sign.distance != f64::INFINITY {
            station_signs.push(station_sign);
          }
        }
      }
    }
  }

  station_signs
}
