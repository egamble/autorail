use crate::common::{Realm, Station};
use crate::common::{
  complete_function,
  create_and_write,
  create_and_writeln,
  realm_to_command_realm,
  EMPTY,
};

use crate::rail_functions::station_name::{break_up_station_name};


fn build_station_body(
  station: &Station,
  station_id: usize,
  build_all_route: &Vec<usize>,
) -> String {
  let mut body = r#"execute positioned *1* run ***/x/station/build/*2* {next_station_id:*6*}

data merge block *3* {front_text: {has_glowing_text: 1b, messages: [{"text":"*4*","color":"blue"},{"text":"*5*","color":"blue","click_event":{"action":"run_command","command":"***/x/station/name_sign {next_station_id:*6*}"}},{"text":"*7*","color":"blue"},{"text":"","color":"blue"}]}}"#.to_string();

  let (x, y, z, _) = station.coords;

  let (row_1, row_2, row_3) = break_up_station_name(station);

  let next_station_id = build_all_route[station_id];

  body = body.replace("*1*", format!("{} {} {}", x, y, z).as_str());
  body = body.replace("*2*", station.direction.to_str());
  body = body.replace("*3*", format!("{} {} {}", x, y + 2, z).as_str());
  body = body.replace("*4*", row_1.as_str());
  body = body.replace("*5*", row_2.as_str());
  body = body.replace("*6*", format!("{}", next_station_id).as_str());
  body = body.replace("*7*", row_3.as_str());

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
  build_all_route: &Vec<usize>,
  out_path: &String
) {
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
