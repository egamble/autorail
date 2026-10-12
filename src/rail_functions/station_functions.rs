use crate::common::{Realm, Station, Switch, Direction};
use crate::common::{
  complete_function,
  create_and_write,
  create_and_writeln,
  realm_to_command_realm,
  get_switch_rail_data,
  get_distance,
  switch_node_id,
  EMPTY,
};

use crate::rail_functions::station_name::{break_up_station_name};


fn build_station_body(
  station: &Station,
  station_id: usize,
  num_stations: usize,
  switches: &Vec<Switch>,
  distances: &Vec<i32>,
  bidirectional_graph: &Vec<usize>,
  build_all_route: &Vec<usize>,
) -> String {
  let num_switches = switches.len();
  let num_nodes = num_stations + 4 * num_switches;

  let mut body = r#"execute positioned *1* run ***/x/station/build/*2* {next_station_id:*3*,first_switch_coords:*4*,first_switch_shape:*5*}

data merge block *6* {front_text:{allow_op_features:1b,has_glowing_text:1b,messages:[{"text":"*7*","color":"blue"},{"text":"*8*","color":"blue","click_event":{"action":"run_command","command":"***/x/station/name_sign {next_alphabetical_station_id:*9*}"}},{"text":"*10*","color":"blue"},{"text":"","color":"blue"}]}}"#.to_string();

  let (x, y, z, _) = station.coords;

  let (row_1, row_2, row_3) = break_up_station_name(station);

  let next_station_id = build_all_route[station_id];

  let mut next_alphabetical_station_id = station_id + 1;
  if next_alphabetical_station_id >= num_stations {
    next_alphabetical_station_id = 0;
  }

  let first_node_id = bidirectional_graph[station_id];
  if first_node_id < num_stations {
    exit!("Station {} is connected directly to station {}", station_id, first_node_id);
  }

  let first_switch_id = (first_node_id - num_stations) / 4;
  let first_switch = &switches[first_switch_id];
  let (switch_x, switch_y, switch_z, _) = first_switch.coords;

  let from_direction_index = (first_node_id - num_stations) % 4;
  let from_direction = Direction::from_index(from_direction_index);

  // Set the default value of shortest_direction to the value of from_direction
  // so that if there is no available path to the station, we can determine
  // that later, since from_direction is not a valid direction to exit the switch.
  let mut shortest_direction = from_direction;
  let mut shortest_distance = i32::MAX;

  for to_direction_index in 0..4 {
    let to_direction = Direction::from_index(to_direction_index);
      
    if to_direction != from_direction &&
      first_switch.has_directions[to_direction_index] {
        let distance = get_distance(
          distances,
          num_nodes,
          switch_node_id(first_switch_id, to_direction_index, num_stations),
          next_station_id
        );

        if distance < shortest_distance {
          shortest_distance = distance;
          shortest_direction = to_direction;
        }
      }
  }

  if shortest_direction == from_direction {
    exit!("No path from station {} to station {}", station_id, next_station_id);
  }

  let first_switch_rail_data = get_switch_rail_data(
    first_switch,
    from_direction,
    shortest_direction
  );
  
  body = body.replace("*1*", format!("{} {} {}", x, y, z).as_str());
  body = body.replace("*2*", station.direction.to_str());
  body = body.replace("*3*", format!("{}", next_station_id).as_str());
  body = body.replace("*4*", format!("'{} {} {}'", switch_x, switch_y, switch_z).as_str());
  body = body.replace("*5*", format!("{}", first_switch_rail_data.to_long_str()).as_str());
  body = body.replace("*6*", format!("{} {} {}", x, y + 2, z).as_str());
  body = body.replace("*7*", row_1.as_str());
  body = body.replace("*8*", row_2.as_str());
  body = body.replace("*9*", format!("{}", next_alphabetical_station_id).as_str());
  body = body.replace("*10*", row_3.as_str());

  body
}


fn add_build_stations_body(realm: Realm, station_id: usize) -> String {
  format!("execute in {} run ***/stations/build/s{}
",
          realm_to_command_realm(realm),
          station_id
  )
}


pub fn write_station_functions(
  stations: &Vec<Station>,
  switches: &Vec<Switch>,
  distances: &Vec<i32>,
  bidirectional_graph: &Vec<usize>,
  build_all_route: &Vec<usize>,
  out_path: &String
) {
  let num_stations = stations.len();
  
  let mut build_stations_body: String = EMPTY;

  for (station_id, station) in stations.iter().enumerate() {
    let (_, _, _, realm) = station.coords;

    build_stations_body.push_str(
      add_build_stations_body(realm, station_id).as_str()
    );


    create_and_writeln(
      &format!("{}/stations/build/s{}.mcfunction",
               out_path,
               station_id
      ),
      complete_function(
        build_station_body(
          station,
          station_id,
          num_stations,
          switches,
          distances,
          bidirectional_graph,
          build_all_route,
        )
      )
    );

  }

  create_and_write(
    &format!("{}/stations/_build.mcfunction",
             out_path
    ),
    complete_function(build_stations_body)
  );
}
