use crate::common::{Direction, Station};
use crate::common::{
  complete_function,
  create_and_write,
  create_and_writeln,
  realm_to_command_realm,
};

use crate::rail_functions::station_name::{
  break_up_station_name,
  make_abbreviated_station_name,
};


const SELECTION_DIVISOR: usize = 4;


fn get_subrange_len(range_len: usize) -> usize {
  range_len.div_ceil(SELECTION_DIVISOR)
}


fn write_start_function(stations: &Vec<Station>, out_path: &String) {
  let mut body = r#"clone ~ ~ ~ ~ ~ ~ ~ ~1 ~

data merge block ~ ~1 ~ {front_text: {messages: ['{"text":""}','{"text":""}','{"text":""}','{"text":""}']}}

$***/select/$(direction)/*1*_a"#.to_string();

  let num_stations = stations.len();

  if num_stations > SELECTION_DIVISOR {
    let upper_station_id = get_subrange_len(num_stations) - 1;

    body = body.replace("*1*", format!("s0_s{}", upper_station_id).as_str());
  } else {
    body = body.replace("*1*", "s0");
  }

  create_and_writeln(
    &format!("{}/select/_start.mcfunction",
             out_path,
    ),
    complete_function(body)
  );
}


fn make_select_sign_coords_str(direction: Direction, is_a: bool) -> String {
  match direction {
    Direction::N => {
      if is_a {"~2 ~1 ~"} else {"~2 ~ ~"}
    },
    Direction::S => {
      if is_a {"~-2 ~1 ~"} else {"~-2 ~ ~"}
    },
    Direction::W => {
      if is_a {"~ ~1 ~-2"} else {"~ ~ ~-2"}
    },
    Direction::E => {
      if is_a {"~ ~1 ~2"} else {"~ ~ ~2"}
    },
  }.to_string()
}


fn write_single_select_functions(
  stations: &Vec<Station>,
  station_id: usize,
  next_range: (usize, usize),
  direction: Direction,
  is_a: bool,
  out_path: &String
) {
  let direction_str = direction.to_str();

  let mut line_1 =
    r#"data merge block *1* {front_text: {messages: ['{"text":"*2*","color":"dark_blue"}','{"text":"*3*","clickEvent":{"action":"run_command","value":"***/x/station/summon/*4* {station_id:*5*}"},"color":"dark_blue"}','{"text":"*6*","color":"dark_blue"}','{"text":""}']}}

"#.to_string();

  let coords_1 = if is_a {"~ ~1 ~"} else {"~ ~ ~"};

  let (row_1, row_2, row_3) = break_up_station_name(&stations[station_id]);

  line_1 = line_1.replace("*1*", coords_1);
  line_1 = line_1.replace("*2*", row_1.as_str());
  line_1 = line_1.replace("*3*", row_2.as_str());
  line_1 = line_1.replace("*4*", direction_str);
  line_1 = line_1.replace("*5*", format!("{}", station_id).as_str());
  line_1 = line_1.replace("*6*", row_3.as_str());

  let mut line_2 =
    r#"data merge block *1* {front_text: {messages: ['{"text":""}','{"text":"Next Selection","clickEvent":{"action":"run_command","value":"***/select/*2*/*3*"},"color":"dark_blue"}','{"text":""}','{"text":""}']}}

"#.to_string();

  let coords_2 = if is_a {"~ ~ ~"} else {"~ ~-1 ~"};

  let (next_begin, next_end) = next_range;

  let range_str = if next_begin == next_end {
    format!("s{}_a", next_begin)
  } else {
    format!("s{}_s{}_a", next_begin, next_end)
  };
  
  line_2 = line_2.replace("*1*", coords_2);
  line_2 = line_2.replace("*2*", direction_str);
  line_2 = line_2.replace("*3*", range_str.as_str());

  let mut line_3 =
    r#"clone ~ ~ ~ ~ ~ ~ *1*

"#.to_string();

  let coords_3 = make_select_sign_coords_str(direction, is_a);

  line_3 = line_3.replace("*1*", coords_3.as_str());

  let mut line_4 =
    r#"data merge block *1* {front_text: {messages: ['{"text":""}','{"text":"Teleport","clickEvent":{"action":"run_command","value":"***/x/station/teleport/*2* {station_id:*3*}"},"color":"dark_blue"}','{"text":""}','{"text":""}']}}
"#.to_string();

  line_4 = line_4.replace("*1*", coords_3.as_str());
  line_4 = line_4.replace("*2*", direction_str);
  line_4 = line_4.replace("*3*", format!("{}", station_id).as_str());
  
  let body = format!("{line_1}{line_2}{line_3}{line_4}");

  create_and_write(
    &format!("{}/select/{}/s{}_{}.mcfunction",
             out_path,
             direction_str,
             station_id,
             if is_a {"a"} else {"b"}
    ),
    complete_function(body)
  );
}


fn write_multiple_select_functions(
  stations: &Vec<Station>,
  range: (usize, usize),
  next_range: (usize, usize),
  direction: Direction,
  is_a: bool,
  out_path: &String
) {
  let direction_str = direction.to_str();

  let mut line_1 =
    r#"data merge block *1* {front_text: {messages: ['{"text":""}','{"text":"*2*","clickEvent":{"action":"run_command","value":"***/select/*3*/*4*"},"color":"dark_blue"}','{"text":"-","color":"dark_blue"}','{"text":"*5*","color":"dark_blue"}']}}

"#.to_string();

  let coords_1 = if is_a {"~ ~1 ~"} else {"~ ~ ~"};

  let (begin, end) = range;

  let station_name_1 = make_abbreviated_station_name(&stations[begin]);
  let station_name_2 = make_abbreviated_station_name(&stations[end]);

  let range_len = end - begin + 1;

  let subrange_str = if range_len > SELECTION_DIVISOR {
    let subrange_len = get_subrange_len(range_len);

    format!("s{}_s{}_b", begin, begin + subrange_len - 1)
  } else {
    format!("s{}_b", begin)
  };

  line_1 = line_1.replace("*1*", coords_1);
  line_1 = line_1.replace("*2*", station_name_1.as_str());
  line_1 = line_1.replace("*3*", direction_str);
  line_1 = line_1.replace("*4*", subrange_str.as_str());
  line_1 = line_1.replace("*5*", station_name_2.as_str());

  let mut line_2 =
    r#"data merge block *1* {front_text: {messages: ['{"text":""}','{"text":"Next Selection","clickEvent":{"action":"run_command","value":"***/select/*2*/*3*"},"color":"dark_blue"}','{"text":""}','{"text":""}']}}

"#.to_string();

  let coords_2 = if is_a {"~ ~ ~"} else {"~ ~-1 ~"};

  let (next_begin, next_end) = next_range;

  let range_str = if next_begin == next_end {
    format!("s{}_a", next_begin)
  } else {
    format!("s{}_s{}_a", next_begin, next_end)
  };
  
  line_2 = line_2.replace("*1*", coords_2);
  line_2 = line_2.replace("*2*", direction_str);
  line_2 = line_2.replace("*3*", range_str.as_str());

  let mut line_3 =
    r#"setblock *1* air
"#.to_string();

  let coords_3 = make_select_sign_coords_str(direction, is_a);
  
  line_3 = line_3.replace("*1*", coords_3.as_str());
  
  let body = format!("{line_1}{line_2}{line_3}");

  create_and_write(
    &format!("{}/select/{}/s{}_s{}_{}.mcfunction",
             out_path,
             direction_str,
             begin,
             end,
             if is_a {"a"} else {"b"}
    ),
    complete_function(body)
  );
}


fn write_select_range(
  stations: &Vec<Station>,
  range: (usize, usize),
  next_range: (usize, usize),
  out_path: &String
) {
  let (begin, end) = range;
  
  for direction_index in 0..4 {
    let direction = Direction::from_usize(direction_index);
      
    if begin == end {
      write_single_select_functions(stations, begin, next_range, direction, true, out_path);
      write_single_select_functions(stations, begin, next_range, direction, false, out_path);
    } else {
      write_multiple_select_functions(stations, range, next_range, direction, true, out_path);
      write_multiple_select_functions(stations, range, next_range, direction, false, out_path);
    }
  }

  if begin != end {
    write_select_ranges(stations, range, out_path);
  }
}


fn write_select_ranges(stations: &Vec<Station>, parent_range: (usize, usize), out_path: &String) {
  let (parent_begin, parent_end) = parent_range;

  let subrange_len = get_subrange_len(parent_end - parent_begin + 1);

  let mut subranges: Vec<(usize, usize)> = Vec::new();

  for i in 0..SELECTION_DIVISOR {
    let begin = parent_begin + i * subrange_len;

    if begin <= parent_end {
      let potential_end = begin + subrange_len - 1;
      let end =
        if potential_end <= parent_end {
          potential_end
        } else {
          parent_end
        };

      subranges.push((begin, end));
    }
  }
  
  let num_subranges = subranges.len();

  for i in 0..num_subranges - 1 {
    write_select_range(stations, subranges[i], subranges[i + 1], out_path);
  }

  write_select_range(stations, subranges[num_subranges - 1], subranges[0], out_path);
}


fn write_teleport_functions(stations: &Vec<Station>, out_path: &String) {
  for (station_id, station) in stations.iter().enumerate() {
    let (x, y, z, realm) = station.coords;
  
    let mut x_offset = 0;
    let mut z_offset = 0;

    match station.direction {
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

    let facing_degrees = match station.direction {
      Direction::N => 0,
      Direction::S => -180,
      Direction::W => -90,
      Direction::E => 90,
    };

    let body = format!("execute in {} run tp @p {} {} {} {} 0",
                        realm_to_command_realm(realm),
                        x + x_offset, y, z + z_offset,
                        facing_degrees
    );

    create_and_writeln(
      &format!("{}/x/teleport/s{}.mcfunction",
               out_path,
               station_id
      ),
      complete_function(body)
    );
  }
}

  
pub fn write_select_functions(stations: &Vec<Station>, out_path: &String) {

  write_start_function(stations, out_path);

  write_select_ranges(stations, (0, stations.len() - 1), out_path);

  write_teleport_functions(stations, out_path);
}
