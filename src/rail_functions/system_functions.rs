use crate::common::{Station, StationSign, Switch};

use crate::rail_functions::station_functions::{write_station_functions};
use crate::rail_functions::sign_functions::{write_sign_functions};
use crate::rail_functions::switch_functions::{write_switch_functions};


pub fn write_system_functions(
  stations: &Vec<Station>,
  station_signs: &Vec<StationSign>,
  switches: &Vec<Switch>,
  distances: &Vec<i32>,
  out_path: &String
) {
  write_station_functions(stations, out_path);

  write_sign_functions(
    station_signs,
    stations,
    distances,
    out_path
  );

  write_switch_functions(
    switches,
    stations.len(),
    distances,
    out_path
  );
}
