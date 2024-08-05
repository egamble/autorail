use crate::common::{Direction, RailData, Switch};
use crate::common::{
  complete_function,
  create_and_write,
  realm_to_command_realm,
  get_num_nodes,
  get_distance,
  switch_node_id,
  EMPTY,
};


fn get_switch_rail_data(
  switch: &Switch,
  from_direction: Direction,
  to_direction: Direction
) -> RailData {
  match from_direction {
    Direction::N => {
      match to_direction {
        Direction::W => RailData::NW,
        Direction::E => RailData::NE,
        _ => if switch.has_directions[Direction::W as usize] {RailData::SW} else {RailData::SE}
      }
    },
    Direction::S => {
      match to_direction {
        Direction::W => RailData::SW,
        Direction::E => RailData::SE,
        _ => if switch.has_directions[Direction::W as usize] {RailData::NW} else {RailData::NE}
      }
    },
    Direction::W => {
      match to_direction {
        Direction::N => RailData::NW,
        Direction::S => RailData::SW,
        _ => if switch.has_directions[Direction::N as usize] {RailData::NE} else {RailData::SE}
      }
    },
    Direction::E => {
      match to_direction {
        Direction::N => RailData::NE,
        Direction::S => RailData::SE,
        _ => if switch.has_directions[Direction::N as usize] {RailData::NW} else {RailData::SW}
      }
    },
  }
}


fn switch_body(
  switch: &Switch,
  switch_id: usize,
  from_direction: Direction,
  num_stations: usize,
  distances: &Vec<i32>,
  num_nodes: usize
) -> String {
  let mut shortest_directions: Vec<Direction> = Vec::new();
  let mut num_shortest_directions: Vec<i32> = vec![0; 4];

  for station_id in 0..num_stations {
    // Set the default value of shortest_direction to the value of from_direction
    // so that if there is no available path to the station, we can determine
    // that later, since from_direction is not a valid direction to exit the switch.
    let mut shortest_direction = from_direction;
    let mut shortest_distance = i32::MAX;

    for to_direction_index in 0..4 {
      let to_direction = Direction::from_usize(to_direction_index);
      
      if to_direction != from_direction &&
        switch.has_directions[to_direction_index] {
          let distance = get_distance(
            distances,
            num_nodes,
            switch_node_id(switch_id, to_direction_index, num_stations),
            station_id
          );

          if distance < shortest_distance {
            shortest_distance = distance;
            shortest_direction = to_direction;
          }
        }
    }

    shortest_directions.push(shortest_direction);
    num_shortest_directions[shortest_direction as usize] += 1;
  }

  let mut max_num_shortest_directions = -1;
  let mut max_to_direction = from_direction;
  
  for to_direction_index in 0..4 {
    let to_direction = Direction::from_usize(to_direction_index);
      
    if to_direction != from_direction &&
      switch.has_directions[to_direction_index] &&
      num_shortest_directions[to_direction_index] > max_num_shortest_directions {
        max_num_shortest_directions = num_shortest_directions[to_direction_index];
        max_to_direction = to_direction;
      }
  }

  if max_to_direction == from_direction {
    panic!("No connection to stations from switch {} coming from direction {}", switch_id, from_direction.to_str());
  }

  let max_switch_rail_data = get_switch_rail_data(
    switch,
    from_direction,
    max_to_direction
  );
  
  let mut body = format!("***/x/switch/set_{}_{}

",
                         from_direction.to_str(),
                         max_switch_rail_data.to_str()
  );

  for (station_id, to_direction) in shortest_directions.iter().enumerate() {
    if *to_direction != from_direction && *to_direction != max_to_direction {

      let switch_rail_data = get_switch_rail_data(
        switch,
        from_direction,
        *to_direction
      );

      let mut body_line = r#"execute if entity @e[type=minecart,name="*1*",distance=..2.5] run ***/x/switch/*2*
"#.to_string();

      body_line = body_line.replace("*1*", format!("S{}", station_id).as_str());
      body_line = body_line.replace("*2*",
                                    format!("set_{}_{}",
                                            from_direction.to_str(),
                                            switch_rail_data.to_str()
                                    ).as_str()
      );

      body.push_str(body_line.as_str());
    };
  }
  
  body
}


fn add_build_switches_body(
  switch: &Switch,
  switch_id: usize,
  direction: Direction
) -> String {
  let mut body = r#"execute in *1* run setblock *2* air
execute in *1* run setblock *2* command_block[facing=down]{Command:"***/switches/*3*"}
"#.to_string();

  let (x, y, z, realm) = switch.coords;

  let mut x_offset = 0;
  let mut z_offset = 0;

  match direction {
    Direction::N => {
      z_offset = -2;
    },
    Direction::S => {
      z_offset = 2;
    },
    Direction::W => {
      x_offset = -2;
    },
    Direction::E => {
      x_offset = 2;
    },
  }

  body = body.replace("*1*", realm_to_command_realm(realm).as_str());
  body = body.replace("*2*", format!("{} {} {}", x + x_offset, y - 2, z + z_offset).as_str());
  body = body.replace("*3*", format!("sw{}_{}", switch_id, direction.to_str()).as_str());
  
  body
}


pub fn write_switch_functions(
  switches: &Vec<Switch>,
  num_stations: usize,
  distances: &Vec<i32>,
  out_path: &String
) {
  let num_nodes = get_num_nodes(distances);

  let mut build_switches_body: String = EMPTY;

  for (switch_id, switch) in switches.iter().enumerate() {

    for direction_index in 0..4 {
      if switch.has_directions[direction_index] {
        let direction = Direction::from_usize(direction_index);

        build_switches_body.push_str(
          add_build_switches_body(
            switch,
            switch_id,
            direction
          ).as_str()
        );

        create_and_write(
          &format!("{}/switches/sw{}_{}.mcfunction",
                   out_path,
                   switch_id,
                   direction.to_str()
          ),
          complete_function(
            switch_body(switch,
                        switch_id,
                        direction,
                        num_stations,
                        distances,
                        num_nodes
            )
          )
        );
      }
    }
  }

  create_and_write(
    &format!("{}/switches/_build.mcfunction",
             out_path
    ),
    complete_function(build_switches_body)
  );
}
