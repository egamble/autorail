use crate::common::{BlockCoords, Direction, Station, StationSign};
use crate::common::{
  block_coords_to_file_name,
  complete_function,
  create_and_write,
  create_and_writeln,
  realm_to_command_realm,
  get_num_nodes,
  get_distance,
  EMPTY,
};

use crate::rail_functions::station_name::{break_up_station_name};


fn station_sign_body(station_sign: &StationSign, stations: &Vec<Station>) -> String {
  let mut body = r#"***/x/station/quick_select {*1*,direction:*2*,select_fn:*3*}"#.to_string();

  let belongs_to_station = &stations[station_sign.belongs_to_station_id];
  let (x, y, z, _) = belongs_to_station.coords;

  let mut x_offset = 0;
  let mut z_offset = 0;

  match belongs_to_station.direction {
    Direction::N => {
      x_offset = -1;
    },
    Direction::S => {
      x_offset = 1;
    },
    Direction::W => {
      z_offset = 1;
    },
    Direction::E => {
      z_offset = -1;
    },
  }

  body = body.replace("*1*", format!("x:{},y:{},z:{}", x + x_offset, y + 1, z + z_offset).as_str());
  body = body.replace("*2*", belongs_to_station.direction.to_str());
  body = body.replace("*3*", format!("s{}_a", station_sign.refers_to_station_id).as_str());

  body
}


fn build_station_sign_body(
  station_sign: &StationSign,
  stations: &Vec<Station>,
  distances: &Vec<i32>,
  num_nodes: usize
) -> String {
  let mut body = r#"data merge block *1* {front_text: {messages: ['{"text":"*2*","color":"dark_blue"}','{"text":"*3*","color":"dark_blue","clickEvent":{"action":"run_command","value":"***/signs/*4*"}}','{"text":"*5*","color":"dark_blue"}','{"text":"*6*","color":"dark_blue"}']}}"#.to_string();
  
  let sign_coords = station_sign.coords;
  let (x, y, z, _) = sign_coords;

  let refers_to_station = &stations[station_sign.refers_to_station_id];
  let (row_1, row_2, row_3) = break_up_station_name(refers_to_station);

  let eucl_distance = station_sign.distance.round();
  let rail_distance = get_distance(
    distances,
    num_nodes,
    station_sign.belongs_to_station_id,
    station_sign.refers_to_station_id
  );

  let mut row_4 = EMPTY;
  if station_sign.nearest_num > 0 {
    row_4 = format!("N{} E{} R{}",
                    station_sign.nearest_num,
                    if eucl_distance == f64::INFINITY {"∞".to_string()} else {eucl_distance.to_string()},
                    if rail_distance == i32::MAX {"∞".to_string()} else {rail_distance.to_string()},
    )
  }

  body = body.replace("*1*", format!("{x} {y} {z}").as_str());
  body = body.replace("*2*", row_1.as_str());
  body = body.replace("*3*", row_2.as_str());
  body = body.replace("*4*", &block_coords_to_file_name(sign_coords));
  body = body.replace("*5*", row_3.as_str());
  body = body.replace("*6*", row_4.as_str());

  body
}


fn add_build_station_signs_body(sign_coords: BlockCoords) -> String {
  let (_, _, _, realm) = sign_coords;
  
  format!("execute in {} run ***/signs/build_{}
",
          realm_to_command_realm(realm),
          block_coords_to_file_name(sign_coords),
  )
}


// The reason sign functions use a sign's coordinates in the function name rather
// than a simple unique number, such as a sign's index within the station_signs vector,
// is so an existing sign will continue to run the correct sign function even when
// new signs are added and the build function hasn't yet been run near the existing sign.

pub fn write_sign_functions(
  station_signs: &Vec<StationSign>,
  stations: &Vec<Station>,
  distances: &Vec<i32>,
  out_path: &String
) {
  let num_nodes = get_num_nodes(distances);

  let mut build_station_signs_body: String = EMPTY;

  for station_sign in station_signs {

    let sign_coords = station_sign.coords;

    build_station_signs_body.push_str(
      add_build_station_signs_body(sign_coords).as_str()
    );

    create_and_writeln(
      &format!("{}/signs/{}.mcfunction",
               out_path,
               block_coords_to_file_name(sign_coords),
      ),
      complete_function(
        station_sign_body(station_sign, stations)
      )
    );

    create_and_writeln(
      &format!("{}/signs/build_{}.mcfunction",
               out_path,
               block_coords_to_file_name(sign_coords),
      ),
      complete_function(
        build_station_sign_body(
          station_sign,
          stations,
          distances,
          num_nodes
        )
      )
    );
  }

  create_and_write(
    &format!("{}/signs/_build.mcfunction",
             out_path
    ),
    complete_function(build_station_signs_body)
  );
}
