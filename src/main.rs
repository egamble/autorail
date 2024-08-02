#[macro_use]
mod common;
mod diagnostics;
mod in_files;
mod blocks;
mod rail_system;
mod rail_functions;

use std::collections::{HashMap};

use crate::common::{Block, BlockCoords, Realm};
use crate::common::{coord_from_str, block_coords_to_chunk_coords};

use crate::diagnostics::{
  write_stations,
  write_station_signs,
  write_switches,
  write_switches_nearest_station,
  write_distances,
  write_rail_blocks,
  write_chunks
};

use crate::in_files::{
  build_ties_map,
  build_weights_map
};

use crate::blocks::find_blocks::{find_blocks};

use crate::rail_system::find_stations::{find_stations};
use crate::rail_system::find_station_signs::{find_station_signs};
use crate::rail_system::find_switches::{find_switches};
use crate::rail_system::find_distances::{find_distances};

use crate::rail_functions::fixed_functions::{write_fixed_functions};
use crate::rail_functions::system_functions::{write_system_functions};
use crate::rail_functions::select_functions::{write_select_functions};

  
fn main() {
  let args: Vec<String> = std::env::args().collect();

  match &*args {
    [_,
     block_x_str,
     block_z_str,
     world_dir,
     functions_out_path,
     diagnostics_out_path,
     ties_path,
     weights_path,
    ] => {

      let starting_chunk_coords = block_coords_to_chunk_coords((
        coord_from_str(block_x_str),
        0,
        coord_from_str(block_z_str),
        Realm::Overworld,
      ));
      
      println!("\nReading from ties file {:?}", ties_path);
      let ties_map = build_ties_map(ties_path);

      println!("Reading from weights file {:?}", weights_path);
      let weights_map = build_weights_map(weights_path);


      // find all potentially relevant blocks

      println!("\nReading from regions:");
      let (blocks, chunks) = find_blocks(starting_chunk_coords, world_dir, &ties_map);

      let mut rail_map: HashMap<BlockCoords, Block> = HashMap::new();
      let mut sign_map: HashMap<BlockCoords, Block> = HashMap::new();

      for block in &blocks {
        if block.is_rail() {
          rail_map.insert(block.coords, block.clone());
        } else if block.is_sign() {
          sign_map.insert(block.coords, block.clone());
        }
      }


      // build rail system
      
      println!("\nFinding stations");
      let stations = find_stations(&blocks, &rail_map, &sign_map);

      println!("Finding station signs");
      let station_signs = find_station_signs(&blocks, &stations);

      println!("Finding switches");
      let switches = find_switches(&blocks, &rail_map);

      println!("Finding connections and shortest routes");
      let (distances, rail_system_coords) =
        find_distances(
          &stations,
          &switches,
          &rail_map,
          &ties_map,
          &weights_map
        );


      // write functions

      println!("\nWriting fixed functions");
      write_fixed_functions(functions_out_path);

      println!("Writing system functions");
      write_system_functions(
        &stations,
        &station_signs,
        &switches,
        &distances,
        functions_out_path
      );

      println!("Writing select functions");
      write_select_functions(&stations, functions_out_path);


      // write diagnostics

      println!("\nWriting diagnostics");

      write_stations(
        &stations,
        &format!("{diagnostics_out_path}/stations.tsv"));

      write_station_signs(
        &stations,
        &station_signs,
        &format!("{diagnostics_out_path}/station-signs.tsv"));

      write_switches(
        &switches,
        &format!("{diagnostics_out_path}/switches.tsv"));

      write_switches_nearest_station(
        &switches,
        &stations,
        &format!("{diagnostics_out_path}/switches-nearest-station.tsv"));

      write_distances(
        &distances,
        &format!("{diagnostics_out_path}/distances.dat"));

      write_rail_blocks(
        &rail_system_coords,
        &rail_map,
        &format!("{diagnostics_out_path}/rail-blocks.tsv"));

      write_chunks(
        &chunks,
        &format!("{diagnostics_out_path}/chunks.tsv"));
    },
    _ => {
      let command = &args[0];
      exit!("Usage: {} <block_x> <block_z> <world_dir> <functions_out_path> <diagnostics_out_path> <ties_path> <weights_path>" , command);
    }
  }
}
