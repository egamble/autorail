use crate::common::{Direction, Station, Switch};
use crate::common::switch_node_id;


/// Finds the switch directions for "BuildAll".
///
/// `bidirectional_graph` is a directed state graph.
///
/// Stations are graph states in [0, num_stations).
///
/// Switch ports are graph states after the station states:
///
///   num_stations + switch_no * 4 + direction
///
/// When the cart arrives at a switch through direction `in`:
///
///   cart arrives at switch `in`
///       -> choose a different existing direction `out`
///       -> bidirectional_graph[out] is the next state
///
/// The generated route satisfies:
///
///   * every station is visited
///   * every switch is visited
///   * no graph state is visited twice, except the final return
///     to the starting state
///   * no outgoing switch port is used more than once
///   * a switch never immediately reverses direction
///
/// Incoming switch-port states which are not part of the route are
/// filled with arbitrary legal directions. They do not affect the
/// BuildAll route because the route never enters those states.
pub fn find_buildall_directions(
  switches: &Vec<Switch>,
  bidirectional_graph: &Vec<usize>,
) -> Option<Vec<Direction>> {
  let num_switches = switches.len();

  if bidirectional_graph.len() < 4 * num_switches {
    println!(
      "Cannot find BuildAll directions: graph is too short."
    );
    return None;
  }

  let num_stations =
    bidirectional_graph.len() - 4 * num_switches;

  let graph_len = bidirectional_graph.len();

  if let Err(error) = validate_input_graph(
    switches,
    bidirectional_graph,
  ) {
    println!(
      "Cannot find BuildAll directions: {}",
      error
    );
    return None;
  }

  // A BuildAll route may start at a station or at an actual
  // switch port.
  let possible_starts =
    build_possible_starts(
      switches,
      num_stations,
    );

  // Try every possible starting state.
  //
  // We deliberately keep the starting state internal. The public
  // result is only the direction table, so the verifier below
  // independently searches for a valid route through that table.
  for start in possible_starts {
    let mut visited_states =
      vec![false; graph_len];

    let mut visited_stations =
      vec![false; num_stations];

    let mut visited_switches =
      vec![false; num_switches];

    // One entry for every switch/incoming-direction pair.
    //
    // The index is:
    //
    //   switch_no * 4 + incoming_direction
    //
    // Some entries may remain None if that incoming port was not
    // encountered by the successful route.
    let mut chosen_directions =
      vec![None::<usize>; num_switches * 4];

    // Absolute graph-state index of the outgoing switch port.
    //
    // This prevents two different incoming ports from consuming
    // the same outgoing port.
    let mut used_outgoing_ports =
      vec![false; graph_len];

    if let Some(directions) = search_buildall_route(
      start,
      start,
      switches,
      bidirectional_graph,
      num_stations,
      &mut visited_states,
      &mut visited_stations,
      &mut visited_switches,
      &mut chosen_directions,
      &mut used_outgoing_ports,
      0,
    ) {
      return Some(directions);
    }
  }

  println!("No BuildAll route found.");
  None
}

/// Build every state at which BuildAll may legally start.
fn build_possible_starts(
  switches: &[Switch],
  num_stations: usize,
) -> Vec<usize> {
  let mut starts = Vec::<usize>::new();

  // Stations.
  for station in 0..num_stations {
    starts.push(station);
  }

  // Actual switch ports.
  for switch_no in 0..switches.len() {
    for direction in 0..4 {
      if switches[switch_no].has_directions[direction] {
        starts.push(
          switch_node_id(
            switch_no,
            direction,
            num_stations,
          )
        );
      }
    }
  }

  starts
}

/// Recursive backtracking search.
///
/// The search constructs one simple closed BuildAll route.
///
/// A graph state may only occur once in the route, except that
/// `start` may occur once more as the final destination.
///
/// `used_outgoing_ports` is separate from `visited_states` because
/// two different incoming switch ports may otherwise select the
/// same outgoing physical port.
fn search_buildall_route(
  start: usize,
  current: usize,
  switches: &[Switch],
  bidirectional_graph: &[usize],
  num_stations: usize,
  visited_states: &mut Vec<bool>,
  visited_stations: &mut Vec<bool>,
  visited_switches: &mut Vec<bool>,
  chosen_directions: &mut Vec<Option<usize>>,
  used_outgoing_ports: &mut Vec<bool>,
  depth: usize,
) -> Option<Vec<Direction>> {
  let num_switches = switches.len();
  let graph_len = bidirectional_graph.len();

  if depth > graph_len {
    return None;
  }

  // ------------------------------------------------------------
  // Already visited state.
  // ------------------------------------------------------------

  if visited_states[current] {
    // The only legal repeated state is the starting state,
    // reached after at least one transition.
    if current != start || depth == 0 {
      return None;
    }

    if all_vertices_visited(
      visited_stations,
      visited_switches,
    ) {
      return build_direction_vector(
        switches,
        chosen_directions,
      );
    }

    return None;
  }

  visited_states[current] = true;

  // ------------------------------------------------------------
  // Station
  // ------------------------------------------------------------

  if current < num_stations {
    let station = current;

    let was_station_visited =
      visited_stations[station];

    visited_stations[station] = true;

    let next = bidirectional_graph[current];

    if next >= graph_len {
      visited_states[current] = false;
      visited_stations[station] =
        was_station_visited;
      return None;
    }

    if next == start {
      if all_vertices_visited(
        visited_stations,
        visited_switches,
      ) {
        return build_direction_vector(
          switches,
          chosen_directions,
        );
      }
    } else if !visited_states[next] {
      if let Some(result) =
        search_buildall_route(
          start,
          next,
          switches,
          bidirectional_graph,
          num_stations,
          visited_states,
          visited_stations,
          visited_switches,
          chosen_directions,
          used_outgoing_ports,
          depth + 1,
        )
      {
        return Some(result);
      }
    }

    visited_states[current] = false;
    visited_stations[station] =
      was_station_visited;

    return None;
  }

  // ------------------------------------------------------------
  // Switch port
  // ------------------------------------------------------------

  let relative =
    current - num_stations;

  let switch_no =
    relative / 4;

  let incoming_direction =
    relative % 4;

  if switch_no >= num_switches {
    visited_states[current] = false;
    return None;
  }

  if !switches[switch_no]
    .has_directions[incoming_direction]
  {
    visited_states[current] = false;
    return None;
  }

  let was_switch_visited =
    visited_switches[switch_no];

  visited_switches[switch_no] = true;

  let direction_slot =
    switch_no * 4 + incoming_direction;

  // ------------------------------------------------------------
  // This incoming port has already been assigned.
  // ------------------------------------------------------------

  if let Some(outgoing_direction) =
    chosen_directions[direction_slot]
  {
    let outgoing_port =
      switch_node_id(
        switch_no,
        outgoing_direction,
        num_stations,
      );

    if used_outgoing_ports[outgoing_port] {
      visited_states[current] = false;
      visited_switches[switch_no] =
        was_switch_visited;
      return None;
    }

    let next =
      bidirectional_graph[outgoing_port];

    if next >= graph_len {
      visited_states[current] = false;
      visited_switches[switch_no] =
        was_switch_visited;
      return None;
    }

    used_outgoing_ports[outgoing_port] = true;

    if next == start {
      if all_vertices_visited(
        visited_stations,
        visited_switches,
      ) {
        return build_direction_vector(
          switches,
          chosen_directions,
        );
      }
    } else if !visited_states[next] {
      if let Some(result) =
        search_buildall_route(
          start,
          next,
          switches,
          bidirectional_graph,
          num_stations,
          visited_states,
          visited_stations,
          visited_switches,
          chosen_directions,
          used_outgoing_ports,
          depth + 1,
        )
      {
        return Some(result);
      }
    }

    used_outgoing_ports[outgoing_port] =
      false;

    visited_states[current] = false;
    visited_switches[switch_no] =
      was_switch_visited;

    return None;
  }

  // ------------------------------------------------------------
  // Try every legal outgoing direction.
  //
  // Prefer transitions which enter a previously unvisited
  // station/switch.
  // ------------------------------------------------------------

  for prefer_unvisited in [true, false] {
    for outgoing_direction in 0..4 {
      // No immediate U-turn at a switch.
      if outgoing_direction == incoming_direction {
        continue;
      }

      // Outgoing direction must physically exist.
      if !switches[switch_no]
        .has_directions[outgoing_direction]
      {
        continue;
      }

      let outgoing_port =
        switch_node_id(
          switch_no,
          outgoing_direction,
          num_stations,
        );

      // This physical outgoing port has already been consumed
      // by another transition in this candidate route.
      if used_outgoing_ports[outgoing_port] {
        continue;
      }

      let next =
        bidirectional_graph[outgoing_port];

      if next >= graph_len {
        continue;
      }

      // A state cannot be revisited, except for the starting
      // state as the final destination.
      if next != start
        && visited_states[next]
      {
        continue;
      }

      let next_is_new_vertex =
        is_new_vertex(
          next,
          num_stations,
          visited_stations,
          visited_switches,
        );

      if prefer_unvisited
        && !next_is_new_vertex
      {
        continue;
      }

      if !prefer_unvisited
        && next_is_new_vertex
      {
        continue;
      }

      // Choose this transition.
      chosen_directions[direction_slot] =
        Some(outgoing_direction);

      used_outgoing_ports[outgoing_port] =
        true;

      let result =
        if next == start {
          if all_vertices_visited(
            visited_stations,
            visited_switches,
          ) {
            build_direction_vector(
              switches,
              chosen_directions,
            )
          } else {
            None
          }
        } else {
          search_buildall_route(
            start,
            next,
            switches,
            bidirectional_graph,
            num_stations,
            visited_states,
            visited_stations,
            visited_switches,
            chosen_directions,
            used_outgoing_ports,
            depth + 1,
          )
        };

      if let Some(result) = result {
        return Some(result);
      }

      // Backtrack both pieces of state.
      used_outgoing_ports[outgoing_port] =
        false;

      chosen_directions[direction_slot] =
        None;
    }
  }

  // ------------------------------------------------------------
  // No transition worked from this state.
  // ------------------------------------------------------------

  visited_states[current] = false;
  visited_switches[switch_no] =
    was_switch_visited;

  None
}

/// Returns true if `state` belongs to a station or switch which
/// has not yet been visited.
fn is_new_vertex(
  state: usize,
  num_stations: usize,
  visited_stations: &[bool],
  visited_switches: &[bool],
) -> bool {
  if state < num_stations {
    return !visited_stations[state];
  }

  let relative =
    state - num_stations;

  let switch_no =
    relative / 4;

  if switch_no >= visited_switches.len() {
    return false;
  }

  !visited_switches[switch_no]
}

/// Returns true when every station and switch has been visited.
fn all_vertices_visited(
  visited_stations: &[bool],
  visited_switches: &[bool],
) -> bool {
  visited_stations.iter().all(|&x| x)
    && visited_switches.iter().all(|&x| x)
}

/// Convert the route's partial transition table into the public
/// direction vector.
///
/// Ports which were not encountered by the successful route get
/// an arbitrary legal direction. Those entries are deliberately
/// not required to participate in the BuildAll cycle.
fn build_direction_vector(
  switches: &[Switch],
  chosen_directions: &[Option<usize>],
) -> Option<Vec<Direction>> {
  let num_switches = switches.len();

  let mut result =
    vec![Direction::N; num_switches * 4];

  for switch_no in 0..num_switches {
    for incoming_direction in 0..4 {
      if !switches[switch_no]
        .has_directions[incoming_direction]
      {
        continue;
      }

      let slot =
        switch_no * 4 + incoming_direction;

      let outgoing_direction =
        if let Some(direction) =
          chosen_directions[slot]
        {
          direction
        } else {
          // This port was not part of the BuildAll route.
          //
          // Give it any legal non-reversing direction.
          let mut fallback = None;

          for direction in 0..4 {
            if direction != incoming_direction
              && switches[switch_no]
                .has_directions[direction]
            {
              fallback = Some(direction);
              break;
            }
          }

          match fallback {
            Some(direction) => direction,

            None => {
              println!(
                "No legal fallback direction for \
                 Switch {} {}.",
                switch_no + 1,
                Direction::from_index(
                  incoming_direction
                ).to_str()
              );

              return None;
            }
          }
        };

      result[slot] =
        Direction::from_index(
          outgoing_direction
        );
    }
  }

  Some(result)
}

/// Validate the static graph before searching.
fn validate_input_graph(
  switches: &[Switch],
  bidirectional_graph: &[usize],
) -> Result<(), String> {
  let num_switches = switches.len();

  if bidirectional_graph.len()
    < 4 * num_switches
  {
    return Err(format!(
      "graph length {} is smaller than \
       4 * {} switches.",
      bidirectional_graph.len(),
      num_switches
    ));
  }

  let num_stations =
    bidirectional_graph.len()
      - 4 * num_switches;

  for (switch_no, switch) in
    switches.iter().enumerate()
  {
    let degree =
      switch.has_directions
        .iter()
        .filter(|&&exists| exists)
        .count();

    if degree != 3 && degree != 4 {
      return Err(format!(
        "Switch {} has degree {}, expected 3 or 4.",
        switch_no + 1,
        degree
      ));
    }

    for direction in 0..4 {
      if !switch.has_directions[direction] {
        continue;
      }

      let port =
        switch_node_id(
          switch_no,
          direction,
          num_stations,
        );

      if port >= bidirectional_graph.len() {
        return Err(format!(
          "Switch {} {} has invalid graph index {}.",
          switch_no + 1,
          Direction::from_index(direction).to_str(),
          port
        ));
      }

      let next =
        bidirectional_graph[port];

      if next >= bidirectional_graph.len() {
        return Err(format!(
          "Switch {} {} points to invalid \
           graph state {}.",
          switch_no + 1,
          Direction::from_index(direction).to_str(),
          next
        ));
      }
    }
  }

  // Stations must point to a switch port.
  for station in 0..num_stations {
    let next =
      bidirectional_graph[station];

    if next < num_stations {
      return Err(format!(
        "Station {} points to station {}.",
        station,
        next
      ));
    }

    if next >= bidirectional_graph.len() {
      return Err(format!(
        "Station {} points to invalid state {}.",
        station,
        next
      ));
    }
  }

  Ok(())
}


/// Verify the BuildAll direction table.
///
/// The verifier looks for a valid BuildAll route starting from
/// any legal state. It does NOT require every switch-port state
/// to be visited; only every station and every switch must be
/// visited.
///
/// This matches the invariants used by the generator.
pub fn verify_buildall_directions(
  switches: &[Switch],
  stations: &[Station],
  buildall_directions: &[Direction],
  bidirectional_graph: &[usize],
) -> Result<(), String> {
  let num_stations = stations.len();
  let num_switches = switches.len();

  let expected_directions_len =
    num_switches * 4;

  let expected_graph_len =
    num_stations + expected_directions_len;

  if buildall_directions.len()
    != expected_directions_len
  {
    return Err(format!(
      "BuildAll direction vector has length {}, \
       expected {}.",
      buildall_directions.len(),
      expected_directions_len
    ));
  }

  if bidirectional_graph.len()
    != expected_graph_len
  {
    return Err(format!(
      "Bidirectional graph has length {}, \
       expected {}.",
      bidirectional_graph.len(),
      expected_graph_len
    ));
  }

  validate_input_graph(
    switches,
    bidirectional_graph,
  )?;

  let possible_starts =
    build_possible_starts(
      switches,
      num_stations,
    );

  let mut last_error =
    String::from(
      "No starting state produced a valid BuildAll route."
    );

  for start in possible_starts {
    match verify_route_from_start(
      start,
      switches,
      buildall_directions,
      bidirectional_graph,
      num_stations,
    ) {
      Ok(route) => {
        println!(
          "BuildAll verification succeeded \
           starting at {}.",
          describe_state(
            start,
            num_stations,
          )
        );

        print_route(
          &route,
          num_stations,
        );

        return Ok(());
      }

      Err(error) => {
        last_error = format!(
          "Starting at {}: {}",
          describe_state(
            start,
            num_stations,
          ),
          error
        );
      }
    }
  }

  Err(format!(
    "BuildAll verification FAILED: {}",
    last_error
  ))
}

/// Verify one deterministic BuildAll route.
fn verify_route_from_start(
  start: usize,
  switches: &[Switch],
  buildall_directions: &[Direction],
  bidirectional_graph: &[usize],
  num_stations: usize,
) -> Result<Vec<usize>, String> {
  let num_switches = switches.len();
  let total_states =
    bidirectional_graph.len();

  let mut visited_states =
    vec![false; total_states];

  let mut visited_stations =
    vec![false; num_stations];

  let mut visited_switches =
    vec![false; num_switches];

  // For every outgoing switch port, record the first route
  // state which consumed it.
  let mut used_outgoing_from:
    Vec<Option<usize>> =
      vec![None; total_states];

  let mut route =
    Vec::<usize>::new();

  let mut current = start;

  loop {
    // ----------------------------------------------------------
    // Repeated state.
    // ----------------------------------------------------------

    if visited_states[current] {
      if current == start {
        if all_vertices_visited(
          &visited_stations,
          &visited_switches,
        ) {
          return Ok(route);
        }

        return Err(format!(
          "returned to the start before \
           visiting every station and switch."
        ));
      }

      print_route(
        &route,
        num_stations,
      );

      return Err(format!(
        "entered already visited state {} \
         before returning to the start.",
        describe_state(
          current,
          num_stations,
        )
      ));
    }

    visited_states[current] = true;
    route.push(current);

    // ----------------------------------------------------------
    // Station.
    // ----------------------------------------------------------

    if current < num_stations {
      let station = current;

      visited_stations[station] = true;

      let next =
        bidirectional_graph[current];

      if next >= total_states {
        return Err(format!(
          "{} points to invalid state {}.",
          describe_state(
            current,
            num_stations,
          ),
          next
        ));
      }

      current = next;
      continue;
    }

    // ----------------------------------------------------------
    // Switch.
    // ----------------------------------------------------------

    let relative =
      current - num_stations;

    let switch_no =
      relative / 4;

    let incoming_direction_no =
      relative % 4;

    if switch_no >= num_switches {
      return Err(format!(
        "{} refers to invalid switch {}.",
        describe_state(
          current,
          num_stations,
        ),
        switch_no + 1
      ));
    }

    if !switches[switch_no]
      .has_directions[incoming_direction_no]
    {
      return Err(format!(
        "{} is a nonexistent switch port.",
        describe_state(
          current,
          num_stations,
        )
      ));
    }

    visited_switches[switch_no] =
      true;

    let incoming_direction =
      Direction::from_index(
        incoming_direction_no
      );

    let slot =
      switch_no * 4
        + incoming_direction_no;

    let outgoing_direction =
      buildall_directions[slot];

    // ----------------------------------------------------------
    // Incoming and outgoing must differ.
    // ----------------------------------------------------------

    if outgoing_direction
      == incoming_direction
    {
      return Err(format!(
        "at {} the outgoing direction \
         {:?} equals the incoming direction.",
        describe_state(
          current,
          num_stations,
        ),
        outgoing_direction
      ));
    }

    // ----------------------------------------------------------
    // Outgoing direction must exist.
    // ----------------------------------------------------------

    let outgoing_direction_no =
      outgoing_direction as usize;

    if !switches[switch_no]
      .has_directions[
        outgoing_direction_no
      ]
    {
      return Err(format!(
        "at {} selected nonexistent \
         outgoing direction {:?}.",
        describe_state(
          current,
          num_stations,
        ),
        outgoing_direction
      ));
    }

    let outgoing_port =
      switch_node_id(
        switch_no,
        outgoing_direction_no,
        num_stations,
      );

    if outgoing_port >= total_states {
      return Err(format!(
        "at {} outgoing port {} is invalid.",
        describe_state(
          current,
          num_stations,
        ),
        outgoing_port
      ));
    }

    // ----------------------------------------------------------
    // An outgoing switch port may only be consumed once.
    // ----------------------------------------------------------

    if let Some(previous_source) =
      used_outgoing_from[outgoing_port]
    {
      print_route(
        &route,
        num_stations,
      );

      return Err(format!(
        "at {} outgoing port {} was already \
         used when leaving {}.\n\
         Current transition would be: {} -> {}.",
        describe_state(
          current,
          num_stations,
        ),
        describe_state(
          outgoing_port,
          num_stations,
        ),
        describe_state(
          previous_source,
          num_stations,
        ),
        describe_state(
          current,
          num_stations,
        ),
        describe_state(
          bidirectional_graph[
            outgoing_port
          ],
          num_stations,
        ),
      ));
    }

    used_outgoing_from[outgoing_port] =
      Some(current);

    // ----------------------------------------------------------
    // Follow the selected track.
    // ----------------------------------------------------------

    let next =
      bidirectional_graph[outgoing_port];

    if next >= total_states {
      return Err(format!(
        "{} -> {} points to invalid \
         graph state {}.",
        describe_state(
          current,
          num_stations,
        ),
        describe_state(
          outgoing_port,
          num_stations,
        ),
        next
      ));
    }

    current = next;
  }
}

fn describe_state(
  state: usize,
  num_stations: usize,
) -> String {
  if state < num_stations {
    return format!(
      "Station {}",
      state + 1
    );
  }

  let relative =
    state - num_stations;

  let switch_no =
    relative / 4;

  let direction_no =
    relative % 4;

  format!(
    "Switch {} {}",
    switch_no + 1,
    Direction::from_index(
      direction_no
    ).to_str()
  )
}

/// Print a route for diagnostics.
fn print_route(
  route: &[usize],
  num_stations: usize,
) {
  println!();
  println!(
    "BuildAll route ({} states):",
    route.len()
  );
  println!(
    "----------------------------------------"
  );

  for (step, &state) in
    route.iter().enumerate()
  {
    println!(
      "{:4}: {}",
      step,
      describe_state(
        state,
        num_stations,
      )
    );
  }

  println!(
    "----------------------------------------"
  );
  println!();
}
