#[macro_use]
mod common;
mod diagnostics;
mod in_files;
mod blocks;
mod rail_system;
mod rail_functions;

use std::collections::{HashMap};

use crate::common::{
  Block,
  BlockCoords,
  Direction,
  Realm};
use crate::common::{
  coord_from_str,
  block_coords_to_chunk_coords
};

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

  let (
    block_x_str,
    block_z_str,
    world_dir,
    functions_out_path,
    diagnostics_out_path_option,
    ties_path_option,
    weights_path_option,
  ) = parse_args(&args);

  let starting_chunk_coords = block_coords_to_chunk_coords((
    coord_from_str(block_x_str.as_str()),
    0,
    coord_from_str(block_z_str.as_str()),
    Realm::Overworld,
  ));

  
  let ties_map: HashMap<BlockCoords, (BlockCoords, Direction, Direction)> =
    if let Some(ties_path) = ties_path_option {
      println!("\nReading from ties file {:?}", ties_path);
      build_ties_map(&ties_path)
    } else {
      HashMap::new()
    };

  let weights_map: HashMap<BlockCoords, i32> =
    if let Some(weights_path) = weights_path_option {
      println!("Reading from weights file {:?}", weights_path);
      build_weights_map(&weights_path)
    } else {
      HashMap::new()
    };


  // find all potentially relevant blocks

  println!("\nReading from regions:");
  let (blocks, chunks) = find_blocks(starting_chunk_coords, &world_dir, &ties_map);
  
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
  write_fixed_functions(&functions_out_path);

  println!("Writing system functions");
  write_system_functions(
    &stations,
    &station_signs,
    &switches,
    &distances,
    &functions_out_path
  );
  
  println!("Writing select functions");
  write_select_functions(&stations, &functions_out_path);


  // write diagnostics

  if let Some(diagnostics_out_path) = diagnostics_out_path_option {
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
  };
}


const STARTING_PARAM_INDEX: usize = 3;

fn param_from_args(args: &Vec<String>, param_prefix: &str) -> Option<String> {
  let args_len = args.len();

  for (i, param) in args.iter().enumerate() {
    if i < STARTING_PARAM_INDEX {
      continue;
    }
    
    if let Some(stripped_param) = param.strip_prefix(param_prefix) {
      if stripped_param == "" {
        if i < args_len - 1 {
          return Some(args[i + 1].to_string());
        }
        return None;
      }
      return Some(stripped_param.to_string());
    }
  }

  None
}


const MIN_REQUIRED_ARGS: usize = 5;

fn parse_args(args: &Vec<String>) -> (
  String, // block_x_str
  String, // block_z_str
  String, // world_dir
  String, // functions_out_path
  Option<String>, // diagnostics_out_path_option,
  Option<String>, // ties_path_option
  Option<String>  // weights_path_option
) {
  if args.len() >= MIN_REQUIRED_ARGS {

    let block_x_str = &args[1];
    let block_z_str = &args[2];
    
    if let Some(world_dir) = param_from_args(args, "-i") {
      if let Some(functions_out_path) = param_from_args(args, "-o") {

        let diagnostics_out_path_option = param_from_args(args, "-d");
        let ties_path_option = param_from_args(args, "-t");
        let weights_path_option = param_from_args(args, "-w");

        return (
          block_x_str.to_string(),
          block_z_str.to_string(),
          world_dir,
          functions_out_path,
          diagnostics_out_path_option,
          ties_path_option,
          weights_path_option
        );
      }
    }
  }

  let command = &args[0];
  exit!("Usage: {} <block_x> <block_z> -i <world_dir> -o <functions_out_path> [-d <diagnostics_out_path>] [-t <ties_path>] [-w <weights_path>]", command);
}
