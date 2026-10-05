use crate::common::{Station, Switch};


pub fn find_build_all_route(
  stations: &Vec<Station>,
  switches: &Vec<Switch>,
  distances: &Vec<i32>,
) -> Vec<usize> {
  let num_stations = stations.len();
  let num_switches = switches.len();
  let num_nodes = num_stations + 4 * num_switches;

  // Return the distance from station a to station b.
  //
  // The station IDs occupy the first num_stations entries of
  // the distance matrix.
  let distance = |a: usize, b: usize| -> i64 {
    distances[a * num_nodes + b].max(1) as i64
  };

  // Calculate the total cost of a complete loop.
  let route_cost = |route: &[usize]| -> i64 {
    let mut cost = 0i64;

    for i in 0..route.len() {
      let from = route[i];
      let to = route[(i + 1) % route.len()];
      cost += distance(from, to);
    }

    cost
  };

  // Construct a tour using nearest-neighbor, starting at `start`.
  fn nearest_neighbor(
    num_stations: usize,
    distance: &dyn Fn(usize, usize) -> i64,
    start: usize,
  ) -> Vec<usize> {
    let mut route = Vec::with_capacity(num_stations);
    let mut used = vec![false; num_stations];

    let mut current = start;
    route.push(current);
    used[current] = true;

    for _ in 1..num_stations {
      let mut best_station = None;
      let mut best_distance = i64::MAX;

      for candidate in 0..num_stations {
        if used[candidate] {
          continue;
        }

        let d = distance(current, candidate);

        if d < best_distance {
          best_distance = d;
          best_station = Some(candidate);
        }
      }

      let next = best_station.unwrap();
      route.push(next);
      used[next] = true;
      current = next;
    }

    route
  }

  // Asymmetric 2-opt.
  //
  // For a route:
  //
  //   ... A -> B ... C -> D ...
  //
  // reversing B..C changes the edges to:
  //
  //   ... A -> C ... B -> D ...
  //
  // Unlike the symmetric TSP, we cannot assume that the
  // internal edges have the same cost after reversal, so
  // the complete affected section is evaluated.
  fn two_opt(
    route: &mut Vec<usize>,
    distance: &dyn Fn(usize, usize) -> i64,
  ) {
    let n = route.len();

    if n < 4 {
      return;
    }

    loop {
      let mut improved = false;
      let mut best_delta = 0i64;
      let mut best_i = 0usize;
      let mut best_j = 0usize;

      for i in 0..n - 1 {
        for j in i + 2..n {
          // Don't reverse the entire circular route.
          if i == 0 && j == n - 1 {
            continue;
          }

          let mut old_cost = 0i64;
          let mut new_cost = 0i64;

          // Edge entering the reversed section.
          old_cost += distance(
            route[i],
            route[i + 1],
          );

          new_cost += distance(
            route[i],
            route[j],
          );

          // Internal edges of the reversed section.
          for k in i + 1..j {
            old_cost += distance(
              route[k],
              route[k + 1],
            );

            new_cost += distance(
              route[k + 1],
              route[k],
            );
          }

          // Edge leaving the reversed section.
          old_cost += distance(
            route[j],
            route[(j + 1) % n],
          );

          new_cost += distance(
            route[i + 1],
            route[(j + 1) % n],
          );

          let delta = new_cost - old_cost;

          if delta < best_delta {
            best_delta = delta;
            best_i = i;
            best_j = j;
            improved = true;
          }
        }
      }

      if !improved {
        break;
      }

      route[best_i + 1..=best_j].reverse();
    }
  }

  let mut best_route = Vec::new();
  let mut best_cost = i64::MAX;

  let mut shortest_cost = i64::MAX;
  let mut longest_cost = i64::MIN;

  // Try every station as the starting point. This is cheap enough
  // for the intended O(n^3) complexity and greatly improves the
  // nearest-neighbor result.
  for start in 0..num_stations {
    let mut route = nearest_neighbor(
      num_stations,
      &distance,
      start,
    );

    two_opt(
      &mut route,
      &distance,
    );

    let cost = route_cost(&route);

    if cost < shortest_cost {
      shortest_cost = cost;
    }

    if cost > longest_cost {
      longest_cost = cost;
    }

    if cost < best_cost {
      best_cost = cost;
      best_route = route;
    }
  }

  println!("Shortest build all route found: {}", shortest_cost);
  println!("Longest build all route found:  {}", longest_cost);
  
  // Convert the cyclic ordering into the requested representation:
  //
  //   build_all_route[from_station] = destination_station
  //
  let mut build_all_route = vec![0usize; num_stations];

  for i in 0..num_stations {
    let from = best_route[i];
    let to = best_route[(i + 1) % num_stations];

    build_all_route[from] = to;
  }

  build_all_route
}
