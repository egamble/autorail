use crate::common::{Station};
use crate::common::{EMPTY};


pub fn break_up_station_name(station: &Station) -> (String, String, String) {
  let mut rows = [EMPTY; 3];
  let mut i = 0;
  
  for token in station.name.split(" ") {
    // If the row is empty, we know that the token will fit.
    // If the row is non-empty, check that the row plus the new token plus a separating space will fit.
    if rows[i].len() > 0 && rows[i].len() + token.len() + 1 > 16 {
      i += 1;
      if i > 2 {
        let coords = station.coords;
        exit!("Station name takes more than three lines at: {:?}", coords);
      }
    }
    if rows[i].len() > 0 {
      rows[i].push_str(" ");
    }
    rows[i].push_str(token);
  }

  // When there are only two rows, move the rows down so that the
  // station name starts on the second row, as that looks better.
  if rows[2].len() == 0 {
    rows[2] = rows[1].clone();
    rows[1] = rows[0].clone();
    rows[0] = EMPTY;
  }

  (
    rows[0].clone(),
    rows[1].clone(),
    rows[2].clone(),
  )
}

pub fn make_abbreviated_station_name(station: &Station) -> String {
  let station_name = &station.name;

  if station_name.len() <= 16 {
    return station_name.to_string();
  }

  let tokens: Vec<&str> = station_name.split(" ").collect();
  let (last, but_last) = tokens.split_last().unwrap();

  let mut abbreviated_name = "".to_string();

  for token in but_last {
    let first_char = token.chars().next().unwrap();

    abbreviated_name.push_str(format!("{} ", first_char).as_str());
  }

  abbreviated_name.push_str(last);

  abbreviated_name
}
