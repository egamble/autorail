use std::fs::{File, create_dir_all};
use std::path::Path;
use std::io::{BufReader, BufRead, BufWriter, Write};


pub const EMPTY: String = String::new();

pub const FUNCTION_PREFIX: &str = "function custom:rr";


macro_rules! exit {
  ( $($arg:tt),* ) => {
    eprintln!("");
    eprintln!( $($arg),* );
    std::process::exit(1);
  };
}


#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Realm {
  Overworld,
  Nether,
  End,
}


pub type BlockCoords = (i32, i32, i32, Realm);

pub type ChunkCoords = (i32, i32, Realm);

pub type RegionCoords = (i32, i32, Realm);


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
  N = 0,
  S = 1,
  W = 2,
  E = 3,
}

impl Direction {
  pub fn from_usize(value: usize) -> Direction {
    match value {
      0 => Direction::N,
      1 => Direction::S,
      2 => Direction::W,
      3 => Direction::E,
      _ => panic!("Can't convert to Direction: {}", value),
    }
  }

  pub fn opposite_direction(&self) -> Direction {
    match &self {
      Direction::N => Direction::S,
      Direction::S => Direction::N,
      Direction::W => Direction::E,
      Direction::E => Direction::W,
    }
  }

  pub fn direction_from_str(direction_str: &str) -> Direction {
    match direction_str.to_lowercase().as_str() {
      "n" => Direction::N,
      "s" => Direction::S,
      "w" => Direction::W,
      "e" => Direction::E,
      _ => panic!("Error parsing direction {:?}", direction_str),
    }
  }

  pub fn to_str(&self) -> &str {
    match &self {
      Direction::N => "n",
      Direction::S => "s",
      Direction::W => "w",
      Direction::E => "e",
    }
  }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BlockID {
  UnpoweredRail = 66,
  PoweredRail = 27,
  DetectorRail = 28,
  OakWallSign = 68,
  SpruceWallSign = 168,
  BirchWallSign = 268,
  JungleWallSign = 368,
  AcaciaWallSign = 468,
  DarkOakWallSign = 568,
  MangroveWallSign = 668,
  BambooWallSign = 768,
  CrimsonWallSign = 868,
  WarpedWallSign = 968,
}

impl BlockID {
  pub fn is_rail_id(&self) -> bool {
    [
      BlockID::UnpoweredRail,
      BlockID::PoweredRail,
      BlockID::DetectorRail,
    ].contains(self)
  }

  pub fn is_sign_id(&self) -> bool {
    [
      BlockID::OakWallSign,
      BlockID::SpruceWallSign,
      BlockID::BirchWallSign,
      BlockID::JungleWallSign,
      BlockID::AcaciaWallSign,
      BlockID::DarkOakWallSign,
      BlockID::MangroveWallSign,
      BlockID::BambooWallSign,
      BlockID::CrimsonWallSign,
      BlockID::WarpedWallSign,
    ].contains(self)
  }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RailData {
  NS = 0, // North / South
  EW = 1, // East / West
  AE = 2, // Ascending East
  AW = 3, // Ascending West
  AN = 4, // Ascending North
  AS = 5, // Ascending South
  SE = 6, // South / East
  SW = 7, // South / West
  NW = 8, // North / West
  NE = 9, // North / East
}

impl RailData {
  pub fn is_straight(&self) -> bool {
    [
      RailData::NS,
      RailData::EW,
      RailData::AE,
      RailData::AW,
      RailData::AN,
      RailData::AS,
    ].contains(&self)
  }

  pub fn is_curved(&self) -> bool {
    [
      RailData::SE,
      RailData::SW,
      RailData::NW,
      RailData::NE,
    ].contains(&self)
  }

  pub fn is_east_west(&self) -> bool {
    [
      RailData::EW,
      RailData::AE,
      RailData::AW,
    ].contains(&self)
  }

  pub fn is_north_south(&self) -> bool {
    [
      RailData::NS,
      RailData::AN,
      RailData::AS,
    ].contains(&self)
  }

  pub fn to_str(&self) -> &str {
    match &self {
      RailData::NS => "ns",
      RailData::EW => "ew",
      RailData::AE => "ae",
      RailData::AW => "aw",
      RailData::AN => "an",
      RailData::AS => "as",
      RailData::SE => "se",
      RailData::SW => "sw",
      RailData::NW => "nw",
      RailData::NE => "ne",
    }
  }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SignData {
  U = 0,
  D = 1,
  N = 2,
  S = 3,
  W = 4,
  E = 5,
}

impl SignData {
  pub fn to_direction(&self) -> Direction {
    match &self {
      SignData::N => Direction::N,
      SignData::S => Direction::S,
      SignData::W => Direction::W,
      SignData::E => Direction::E,
      _ => panic!("Can't convert to Direction: {:?}", &self),
    }
  }
}


#[derive(Debug, Clone)]
pub struct Block {
  pub coords: BlockCoords,
  pub id: BlockID,
  pub rail_data: RailData,
  pub sign_data: SignData,
  pub sign_text: String
}


impl Block {
  pub fn is_rail(&self) -> bool {
    (&self.id).is_rail_id()
  }

  pub fn is_sign(&self) -> bool {
    (&self.id).is_sign_id()
  }

  pub fn is_straight_rail(&self) -> bool {
    (&self.rail_data).is_straight()
  }

  pub fn is_curved_rail(&self) -> bool {
    (&self.rail_data).is_curved()
  }

  pub fn is_east_west_rail(&self) -> bool {
    (&self.rail_data).is_east_west()
  }

  pub fn is_north_south_rail(&self) -> bool {
    (&self.rail_data).is_north_south()
  }
}


pub fn block_name_to_id(name: &str) -> Option<BlockID> {
  let id = match name {
    "minecraft:rail"               => BlockID::UnpoweredRail,
    "minecraft:powered_rail"       => BlockID::PoweredRail,
    "minecraft:detector_rail"      => BlockID::DetectorRail,
    "minecraft:oak_wall_sign"      => BlockID::OakWallSign,
    "minecraft:spruce_wall_sign"   => BlockID::SpruceWallSign,
    "minecraft:birch_wall_sign"    => BlockID::BirchWallSign,
    "minecraft:jungle_wall_sign"   => BlockID::JungleWallSign,
    "minecraft:acacia_wall_sign"   => BlockID::AcaciaWallSign,
    "minecraft:dark_oak_wall_sign" => BlockID::DarkOakWallSign,
    "minecraft:mangrove_wall_sign" => BlockID::MangroveWallSign,
    "minecraft:bamboo_wall_sign"   => BlockID::BambooWallSign,
    "minecraft:crimson_wall_sign"  => BlockID::CrimsonWallSign,
    "minecraft:warped_wall_sign"   => BlockID::WarpedWallSign,
    _ => {
      return None;
    },
  };

  Some(id)
}


pub fn shape_to_rail_data(shape: &str) -> Option<RailData> {
  let rail_data = match shape {
    "north_south"     => RailData::NS,
    "east_west"       => RailData::EW,
    "ascending_east"  => RailData::AE,
    "ascending_west"  => RailData::AW,
    "ascending_north" => RailData::AN,
    "ascending_south" => RailData::AS,
    "south_east"      => RailData::SE,
    "south_west"      => RailData::SW,
    "north_west"      => RailData::NW,
    "north_east"      => RailData::NE,
    _                 => {
      return None;
    },
  };

  Some(rail_data)
}


pub fn facing_to_sign_data(facing: &str) -> Option<SignData> {
  let sign_data = match facing {
    "down"   => SignData::D,
    "up"     => SignData::U,
    "north"  => SignData::N,
    "south"  => SignData::S,
    "west"   => SignData::W,
    "east"   => SignData::E,
    _        => {
      return None;
    },
  };

  Some(sign_data)
}


#[derive(Clone)]
pub struct Station {
  pub coords: BlockCoords,
  pub name: String,
  pub direction: Direction
}


pub struct StationSign {
  pub coords: BlockCoords,
  pub belongs_to_station_id: usize,
  pub refers_to_station_id: usize,
  pub nearest_num: usize,
  pub distance: f64 // distance from "belongs to" station to "refers to" station
}


#[derive(Clone)]
pub struct Switch {
  pub coords: BlockCoords,
  pub has_directions: [bool; 4] // NSWE
}


pub fn block_coords_to_chunk_coords(block_coords: BlockCoords) -> ChunkCoords {
  let (x, _, z, realm) = block_coords;
  (
    x.div_euclid(16),
    z.div_euclid(16),
    realm,
  )
}


pub fn chunk_coords_to_region_coords(chunk_coords: ChunkCoords) -> RegionCoords {
  let (x, z, realm) = chunk_coords;
  (
    x.div_euclid(32),
    z.div_euclid(32),
    realm,
  )
}


pub fn i32_from_str(i32_str: &str, type_str: &str) -> i32 {
  if let Ok(n) = i32_str.parse::<i32>() {
    return n;
  }
  exit!("Error parsing {} {:?}", type_str, i32_str);
}


pub fn coord_from_str(coord_str: &str) -> i32 {
  i32_from_str(coord_str, "coordinate")
}


pub fn realm_from_str(realm_str: &str) -> Realm {
  let lower_realm_str = realm_str.to_lowercase();
  
  if lower_realm_str.contains("nether") {
    return Realm::Nether;
  };

  if lower_realm_str.contains("end") {
    return Realm::End;
  };

  Realm::Overworld
}


pub struct Region {
  pub coords: RegionCoords,
  pub data: Vec<u8>,
  pub last_chunk_num: u32
}


pub fn realm_to_out_string(realm: Realm) -> String {
  match realm {
    Realm::Overworld => "",
    Realm::Nether => "nether",
    Realm::End => "end",
  }.to_string()
}


pub fn realm_to_command_realm(realm: Realm) -> String {
  match realm {
    Realm::Overworld => "overworld",
    Realm::Nether => "the_nether",
    Realm::End => "the_end",
  }.to_string()
}


pub fn read_two_byte_size(data: &Vec<u8>, index: usize) -> usize {
  (
    (data[index + 0] as u16) << 8 |
    (data[index + 1] as u16) << 0
  ) as usize
}


pub fn read_three_byte_size(data: &Vec<u8>, index: usize) -> usize {
  (
    (data[index + 0] as i32) << 16 |
    (data[index + 1] as i32) << 8 |
    (data[index + 2] as i32) << 0
  ) as usize
}


pub fn read_four_byte_size(data: &Vec<u8>, index: usize) -> usize {
  (
    (data[index + 0] as i32) << 24 |
    (data[index + 1] as i32) << 16 |
    (data[index + 2] as i32) << 8 |
    (data[index + 3] as i32) << 0
  ) as usize
}


pub fn read_i32(data: &Vec<u8>) -> i32 {
    (data[0] as i32) << 24 |
    (data[1] as i32) << 16 |
    (data[2] as i32) << 8 |
    (data[3] as i32) << 0
}


fn sqr(x: i32) -> i32 {
  x * x
}


pub fn block_coords_distance(coords1: BlockCoords, coords2: BlockCoords) -> f64 {
  let (x1, y1, z1, realm1) = coords1;
  let (x2, y2, z2, realm2) = coords2;
  
  if realm1 != realm2 {
    return f64::INFINITY;
  }

  ((sqr(x2 - x1) + sqr(y2 - y1) + sqr(z2 - z1)) as f64).sqrt()
}


pub fn bool_to_int(v: bool) -> u32 {
  if v {
    1
  } else {
    0
  }
}


pub fn open_file(path: &String) -> File {
  let file = match File::open(path) {
    Ok(file) => file,
    Err(err) => {
      exit!("Can't open file {:?}: {}", path, err);
    }
  };

  file
}


pub fn create_reader(in_path: &String) -> impl BufRead {
  let in_file = open_file(in_path);
  BufReader::new(in_file)
}


pub fn create_writer(out_path: &String) -> impl Write {
  let out_path_obj = Path::new(out_path);
  let out_path_prefix = out_path_obj.parent().unwrap();

  match create_dir_all(out_path_prefix) {
    Ok(_) => {},
    Err(err) => {
      exit!("Error creating directories in {:?}: {}", out_path, err);
    }
  }

  let out_file = match File::create(out_path) {
    Ok(file) => file,
    Err(err) => {
      exit!("Error creating file {:?}: {}", out_path, err);
    }
  };

  BufWriter::new(out_file)
}


pub fn write_out(writer: &mut impl Write, out_path: &String, out_string: String) {
  if let Err(err) = write!(writer, "{out_string}") {
    exit!("Error writing to file {:?}: {}", out_path, err);
  }
}

pub fn writeln_out(writer: &mut impl Write, out_path: &String, out_string: String) {
  if let Err(err) = writeln!(writer, "{out_string}") {
    exit!("Error writing to file {:?}: {}", out_path, err);
  }
}


pub fn create_and_write(out_path: &String, out_body: String) {
  let mut writer = create_writer(&out_path);
  write_out(&mut writer, &out_path, out_body);
}

pub fn create_and_writeln(out_path: &String, out_body: String) {
  let mut writer = create_writer(&out_path);
  writeln_out(&mut writer, &out_path, out_body);
}


pub fn complete_function_str(function_body: &str) -> String {
  format!("# Generated by autorail. https://github.com/egamble/autorail

{}",
          function_body.replace("***", FUNCTION_PREFIX))
}

pub fn complete_function(function_body: String) -> String {
  complete_function_str(function_body.as_str())
}


pub fn get_num_nodes(distances: &Vec<i32>) -> usize {
  (distances.len() as f64).sqrt() as usize
}


pub fn get_distance(distances: &Vec<i32>, num_nodes: usize, i: usize, j: usize) -> i32 {
  distances[i * num_nodes + j]
}


pub fn set_distance(distances: &mut Vec<i32>, num_nodes: usize, i: usize, j: usize, distance: i32) {
  distances[i * num_nodes + j] = distance;
}


pub fn switch_node_id(switch_id: usize, direction_index: usize, num_stations: usize) -> usize {
  num_stations + 4 * switch_id + direction_index
}


pub fn find_nearest_station_id(coords: BlockCoords, stations: &Vec<Station>) -> Option<(usize, f64)> {
  let mut nearest_distance: f64 = f64::INFINITY;
  let mut nearest_station_id: usize = 0;

  for (station_id, station) in stations.iter().enumerate() {
    let distance = block_coords_distance(coords, station.coords);

    if distance < nearest_distance {
      nearest_distance = distance;
      nearest_station_id = station_id;
    }
  }

  if nearest_distance == f64::INFINITY {
    return None;
  }
  
  Some((nearest_station_id, nearest_distance))
}
