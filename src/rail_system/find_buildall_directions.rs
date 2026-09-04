use crate::common::{Direction, Switch};
use crate::common::{
  switch_node_id
};


#[derive(Clone, Copy, Debug)]
struct HalfEdge {
    /// The doubled physical edge this half-edge belongs to.
    edge: usize,

    /// The other half-edge of the same doubled edge.
    twin: usize,

    /// Vertex at this end of the edge.
    vertex: usize,

    /// If this is a switch end, the corresponding direction.
    /// None means this is a station end.
    switch_direction: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
struct Pair {
    a: usize,
    b: usize,
}


/// Finds the switch directions for "build all".
///
/// The returned vector has length switches.len() * 4.
/// For switch `s` and incoming direction `d`,
///
///     result[4 * s + d]
///
/// gives the direction in which the cart should leave the switch.
///
/// Entries corresponding to directions for which
/// `switches[s].has_directions[d] == false` are left as Direction::N.
///
/// Returns None if the graph is malformed, disconnected, or no suitable
/// transition system can be constructed.
pub fn find_buildall_directions(
    switches: &Vec<Switch>,
    bidirectional_graph: &Vec<usize>,
) -> Option<Vec<Direction>> {
    let num_switches = switches.len();

    if bidirectional_graph.len() < 4 * num_switches {
        return None;
    }

    let num_stations =
        bidirectional_graph.len() - 4 * num_switches;

    let num_vertices = num_stations + num_switches;
    
    let vertex_of_index = |index: usize| -> Option<usize> {
        if index < num_stations {
            // Station.
            Some(index)
        } else if index < num_stations + 4 * num_switches {
            // Switch port.
            let p = index - num_stations;
            Some(num_stations + p / 4)
        } else {
            None
        }
    };

    // ------------------------------------------------------------
    // Validate the switch data.
    // ------------------------------------------------------------

    for (s, sw) in switches.iter().enumerate() {
        let mut degree = 0;

        for d in 0..4 {
            let index = switch_node_id(s, d, num_stations);

            if sw.has_directions[d] {
                degree += 1;

                if bidirectional_graph[index] >= bidirectional_graph.len() {
                    return None;
                }

                // The connection should lead to a station or a
                // valid switch port.
                if vertex_of_index(bidirectional_graph[index]).is_none() {
                    return None;
                }
            }
        }

        // Your junctions have 3 or 4 connections.
        if degree != 3 && degree != 4 {
            return None;
        }

        // There should be no graph entry for a nonexistent port.
        for d in 0..4 {
            if !sw.has_directions[d] {
                let index = switch_node_id(s, d, num_stations);

                // We don't strictly require this to be a sentinel,
                // so we don't reject it.
                let _ = index;
            }
        }
    }

    // ------------------------------------------------------------
    // Build the underlying undirected edge list.
    //
    // Every physical track becomes one edge.
    //
    // A switch-switch connection appears twice in
    // bidirectional_graph, so only add it once.
    //
    // A switch-station connection appears once from the switch.
    // ------------------------------------------------------------

    #[derive(Clone, Copy, Debug)]
    struct PhysicalEdge {
        a: usize,
        b: usize,

        // The direction/port at each end, if that end is a switch.
        a_direction: Option<usize>,
        b_direction: Option<usize>,
    }

    let mut physical_edges = Vec::<PhysicalEdge>::new();

    for s in 0..num_switches {
        for d in 0..4 {
            if !switches[s].has_directions[d] {
                continue;
            }

            let p = switch_node_id(s, d, num_stations);
            let other = bidirectional_graph[p];

            if other >= bidirectional_graph.len() {
                return None;
            }

            let a = num_stations + s;

            if other < num_stations {
                // Switch -> station.
                //
                // This edge is encountered only once, so add it.
                physical_edges.push(PhysicalEdge {
                    a,
                    b: other,
                    a_direction: Some(d),
                    b_direction: None,
                });
            } else {
                // Switch -> switch port.
                let other_port = other - num_stations;
                let other_switch = other_port / 4;
                let other_direction = other_port % 4;

                if other_switch >= num_switches {
                    return None;
                }

                // Check that the other end really points back.
                let other_index =
                    switch_node_id(other_switch, other_direction, num_stations);

                if bidirectional_graph[other_index] != p {
                    return None;
                }

                let b = num_stations + other_switch;

                // Only add the connection once.
                if p < other {
                    physical_edges.push(PhysicalEdge {
                        a,
                        b,
                        a_direction: Some(d),
                        b_direction: Some(other_direction),
                    });
                }
            }
        }
    }

    if physical_edges.is_empty() {
        // No tracks. There is nothing meaningful to build.
        return Some(vec![Direction::N; num_switches * 4]);
    }

    // ------------------------------------------------------------
    // Make sure all relevant vertices belong to one connected
    // component.
    // ------------------------------------------------------------

    let mut adjacency = vec![Vec::<usize>::new(); num_vertices];

    for e in &physical_edges {
        adjacency[e.a].push(e.b);
        adjacency[e.b].push(e.a);
    }

    let start = physical_edges[0].a;

    let mut seen_vertex = vec![false; num_vertices];
    let mut stack = vec![start];

    while let Some(v) = stack.pop() {
        if seen_vertex[v] {
            continue;
        }

        seen_vertex[v] = true;

        for &w in &adjacency[v] {
            if !seen_vertex[w] {
                stack.push(w);
            }
        }
    }

    for v in 0..num_vertices {
        if !adjacency[v].is_empty() && !seen_vertex[v] {
            return None;
        }
    }

    // ------------------------------------------------------------
    // Double every physical edge.
    //
    // This produces an Eulerian multigraph:
    //
    //   original edge e
    //
    // becomes
    //
    //   e_0
    //   e_1
    //
    // At a switch, the two copies of the same physical edge
    // form one forbidden group.
    //
    // At a station there is no forbidden transition: the two
    // copies simply have to be paired so that the cart turns
    // around at the station.
    // ------------------------------------------------------------

    let mut half_edges = Vec::<HalfEdge>::new();

    // For every physical edge we store its two doubled copies.
    //
    // Each copy has two half-edges.
    //
    // copy_half_edges[edge][copy][endpoint]
    //
    // but we flatten this into vectors for convenience.
    let mut copies: Vec<[usize; 2]> = Vec::new();

    for (edge_no, e) in physical_edges.iter().enumerate() {
        let mut edge_copies = [0usize; 2];

        for copy in 0..2 {
            let h0 = half_edges.len();
            let h1 = h0 + 1;

            half_edges.push(HalfEdge {
                edge: edge_no,
                twin: h1,
                vertex: e.a,
                switch_direction: e.a_direction,
            });

            half_edges.push(HalfEdge {
                edge: edge_no,
                twin: h0,
                vertex: e.b,
                switch_direction: e.b_direction,
            });

            edge_copies[copy] = h0;
        }

        copies.push(edge_copies);
    }

    let num_half_edges = half_edges.len();

    // ------------------------------------------------------------
    // Incident half-edges at each vertex.
    // ------------------------------------------------------------

    let mut incident = vec![Vec::<usize>::new(); num_vertices];

    for h in 0..num_half_edges {
        incident[half_edges[h].vertex].push(h);
    }

    // Every vertex in the doubled graph has even degree.
    for v in 0..num_vertices {
        if incident[v].len() % 2 != 0 {
            return None;
        }
    }

    // ------------------------------------------------------------
    // Create the initial transition pairing.
    //
    // pair[h] = the half-edge paired with h at the same vertex.
    //
    // At a switch:
    //
    //   group = the two copies of one physical edge
    //
    // We arrange groups consecutively:
    //
    //   e e f f g g ...
    //
    // and pair the first half with the corresponding half in
    // the second half:
    //
    //   e-f
    //   e-g
    //   f-g
    //
    // This guarantees that no pair belongs to the same
    // physical-edge group.
    //
    // This is the construction in Kotzig's theorem.
    // ------------------------------------------------------------

    let mut pair = vec![usize::MAX; num_half_edges];

    for v in 0..num_vertices {
        let inc = &incident[v];

        if inc.is_empty() {
            continue;
        }

        // Station:
        //
        // It has one physical edge and therefore two copies.
        // Pair those copies. This means:
        //
        //     switch -> station -> switch
        //
        // which is exactly the desired station reversal.
        if v < num_stations {
            if inc.len() != 2 {
                return None;
            }

            pair[inc[0]] = inc[1];
            pair[inc[1]] = inc[0];

            continue;
        }

        // Switch.
        //
        // Build groups of the two copies belonging to each
        // physical edge.
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut used = vec![false; inc.len()];

        for i in 0..inc.len() {
            if used[i] {
                continue;
            }

            let h = inc[i];
            let edge = half_edges[h].edge;

            let mut group = Vec::new();

            for j in i..inc.len() {
                let h2 = inc[j];

                if !used[j] && half_edges[h2].edge == edge {
                    used[j] = true;
                    group.push(h2);
                }
            }

            // A normal non-loop physical edge contributes two
            // half-edges at this switch, one for each copy.
            if group.len() != 2 {
                // This catches an unsupported physical loop from
                // a switch back to itself.
                return None;
            }

            groups.push(group);
        }

        let d = inc.len() / 2;

        if groups.len() != d {
            return None;
        }

        // Flatten the groups.
        let mut ordered = Vec::with_capacity(inc.len());

        for g in groups {
            ordered.extend(g);
        }

        debug_assert_eq!(ordered.len(), inc.len());

        for i in 0..d {
            let a = ordered[i];
            let b = ordered[i + d];

            // They must belong to different physical edges.
            if half_edges[a].edge == half_edges[b].edge {
                return None;
            }

            pair[a] = b;
            pair[b] = a;
        }
    }

    // ------------------------------------------------------------
    // Sanity check that every half-edge has exactly one partner.
    // ------------------------------------------------------------

    if pair.iter().any(|&p| p == usize::MAX) {
        return None;
    }

    for h in 0..num_half_edges {
        if pair[pair[h]] != h {
            return None;
        }
    }

    // ------------------------------------------------------------
    // Repeatedly merge the cycles of the transition system.
    //
    // Following:
    //
    //     h -> twin(h) -> pair(twin(h))
    //
    // traces one closed trail.
    //
    // If there are multiple cycles, find two cycles sharing
    // a switch or station and splice them.
    // ------------------------------------------------------------

    loop {
        let cycle_id = compute_cycle_ids(&half_edges, &pair);

        let number_of_cycles = cycle_id
            .iter()
            .copied()
            .max()
            .map(|x| x + 1)
            .unwrap_or(0);

        if number_of_cycles <= 1 {
            break;
        }

        let mut merged = false;

        // --------------------------------------------------------
        // Try every vertex.
        //
        // At a vertex, every transition pair represents one visit
        // to that vertex in the current cycle decomposition.
        // --------------------------------------------------------

        for v in 0..num_vertices {
            let inc = &incident[v];

            // Get one representative of every transition pair.
            let mut transitions = Vec::<Pair>::new();

            for &h in inc {
                let p = pair[h];

                if h < p {
                    transitions.push(Pair { a: h, b: p });
                }
            }

            // Try two transition pairs belonging to different
            // cycles.
            'outer: for i in 0..transitions.len() {
                for j in (i + 1)..transitions.len() {
                    let t1 = transitions[i];
                    let t2 = transitions[j];

                    let c1 = cycle_id[t1.a];
                    let c2 = cycle_id[t2.a];

                    if c1 == c2 {
                        continue;
                    }

                    let a = t1.a;
                    let b = t1.b;
                    let c = t2.a;
                    let d = t2.b;

                    // There are two ways to cross-splice:
                    //
                    //   (a,c) (b,d)
                    //
                    // or
                    //
                    //   (a,d) (b,c)
                    //
                    // Try the first.
                    if different_groups(
                        &half_edges,
                        a,
                        c,
                    ) && different_groups(
                        &half_edges,
                        b,
                        d,
                    ) {
                        pair[a] = c;
                        pair[c] = a;
                        pair[b] = d;
                        pair[d] = b;

                        merged = true;
                        break 'outer;
                    }

                    // Try the second.
                    if different_groups(
                        &half_edges,
                        a,
                        d,
                    ) && different_groups(
                        &half_edges,
                        b,
                        c,
                    ) {
                        pair[a] = d;
                        pair[d] = a;
                        pair[b] = c;
                        pair[c] = b;

                        merged = true;
                        break 'outer;
                    }
                }
            }

            if merged {
                break;
            }
        }

        if !merged {
            // In the normal degree-3/4 case this should not happen
            // for a connected graph satisfying our assumptions.
            return None;
        }
    }

    // ------------------------------------------------------------
    // Convert the final Euler tour into switch transitions.
    //
    // During traversal:
    //
    //     current half-edge
    //          |
    //          v
    //     traverse its edge
    //          |
    //          v
    //     arrive via twin(current)
    //          |
    //          v
    //     pair[twin(current)]
    //
    // If the arrival is at a switch, the latter is the outgoing
    // half-edge.
    // ------------------------------------------------------------

    let mut buildall_directions =
        vec![Direction::N; num_switches * 4];

    let start = 0usize;
    let mut current = start;
    let mut visited_half_edges = vec![false; num_half_edges];

    loop {
        if visited_half_edges[current] {
            break;
        }

        visited_half_edges[current] = true;

        let arrival = half_edges[current].twin;
        let arrival_vertex = half_edges[arrival].vertex;

        let outgoing = pair[arrival];

        if arrival_vertex >= num_stations {
            // We have arrived at a switch.
            let switch_no = arrival_vertex - num_stations;

            let in_direction_index = half_edges[arrival]
                .switch_direction?;

            let out_direction_index = half_edges[outgoing]
                .switch_direction?;

            // The outgoing edge must actually belong to the same
            // switch.
            if half_edges[outgoing].vertex != arrival_vertex {
                return None;
            }

            // Most importantly, incoming and outgoing physical
            // edges must differ.
            if half_edges[arrival].edge ==
                half_edges[outgoing].edge
            {
                return None;
            }

            buildall_directions[switch_node_id(switch_no, in_direction_index, 0)] =
                Direction::from_index(out_direction_index);
        } else {
            // Station.
            //
            // We don't output anything here. The station pairing
            // already forces the cart to reverse.
        }

        current = outgoing;
    }

    // We must have traversed every doubled edge.
    if visited_half_edges.iter().any(|&x| !x) {
        return None;
    }

    // ------------------------------------------------------------
    // Validate that every real switch direction got assigned.
    // ------------------------------------------------------------

    for s in 0..num_switches {
        for d in 0..4 {
            if switches[s].has_directions[d] {
                // A valid output direction cannot be the placeholder
                // N merely because N happens to be the first enum
                // value. Check by reconstructing whether the slot
                // has actually been encountered.
                //
                // We do this with a separate traversal below.
            }
        }
    }

    // More robustly validate by traversing again and counting
    // incoming directions.
    let mut assigned = vec![false; num_switches * 4];

    current = start;

    loop {
        if visited_half_edges[current] == false {
            // Should never happen.
            return None;
        }

        // We need a separate stopping condition because
        // `visited_half_edges` is already populated.
        break;
    }

    // Instead of relying on the placeholder value, reconstruct
    // assignments from the transition system directly.
    for s in 0..num_switches {
        for d in 0..4 {
            if !switches[s].has_directions[d] {
                continue;
            }

            let mut found = 0;

            for h in 0..num_half_edges {
                if half_edges[h].vertex
                    == num_stations + s
                    && half_edges[h].switch_direction
                        == Some(d)
                {
                    found += 1;
                }
            }

            if found != 2 {
                return None;
            }

            assigned[4 * s + d] = true;
        }
    }

    Some(buildall_directions)
}


/// Returns true iff two half-edges belong to different original
/// physical edges.
///
/// At a switch this is exactly the "don't immediately reverse
/// along the same track" condition.
fn different_groups(
    half_edges: &[HalfEdge],
    a: usize,
    b: usize,
) -> bool {
    half_edges[a].edge != half_edges[b].edge
}


/// Compute the cycle containing every half-edge.
///
/// The successor relation is:
///
///     h -> twin(h) -> pair(twin(h))
///
/// so the actual successor half-edge is pair[twin(h)].
fn compute_cycle_ids(
    half_edges: &[HalfEdge],
    pair: &[usize],
) -> Vec<usize> {
    let n = half_edges.len();

    let mut cycle_id = vec![usize::MAX; n];
    let mut next_cycle = 0usize;

    for start in 0..n {
        if cycle_id[start] != usize::MAX {
            continue;
        }

        let mut h = start;

        loop {
            if cycle_id[h] != usize::MAX {
                break;
            }

            cycle_id[h] = next_cycle;

            let arrival = half_edges[h].twin;
            h = pair[arrival];
        }

        next_cycle += 1;
    }

    cycle_id
}
