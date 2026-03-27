use crate::common::{Direction, Switch};
use crate::common::{
  switch_node_id
};
use rand::Rng;


pub fn find_buildall_directions(
  _bidirectional_graph: &Vec<usize>,
  switches: &Vec<Switch>
) -> Vec<Direction> {
  let num_switches = switches.len();

  let mut buildall_directions = vec![Direction::N; num_switches * 4];

  for switch_id in 0..num_switches {
    let switch = &switches[switch_id];

    for from_direction_index in 0..4 { // NSWE
      if switch.has_directions[from_direction_index] {
        loop {
          let to_direction_index = rand::thread_rng().gen_range(0..4);
          if switch.has_directions[to_direction_index] && to_direction_index != from_direction_index {
            buildall_directions[switch_node_id(switch_id, from_direction_index, 0)] = Direction::from_usize(to_direction_index);
            break;
          }
        }
      }
    }
  }

  buildall_directions
}
