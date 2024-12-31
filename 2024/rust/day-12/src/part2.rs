use itertools::Itertools;
use miette::miette;
use petgraph::{
    adj::NodeIndex, algo::condensation,
    graphmap::UnGraphMap, visit::IntoNodeReferences,
};
use std::collections::HashMap;

// Order is important with usage of
// circular_tuple_windows
const DIRECTIONS_CW: [(i32, i32); 4] =
    [(0, 1), (1, 0), (0, -1), (-1, 0)];

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let grid = parse(input)
        .map_err(|err| miette!("Parse error: {}", err))?;

    let mut graph = UnGraphMap::new();

    for (pos, c) in grid.iter() {
        graph.add_node(*pos);

        for dir in DIRECTIONS_CW.iter() {
            let neighbor = (pos.0 + dir.0, pos.1 + dir.1);
            if grid
                .get(&neighbor)
                .is_some_and(|field| *field == *c)
            {
                graph.add_edge(*pos, neighbor, ());
            }
        }
    }

    let grapes = condensation(
        graph.clone().into_graph::<NodeIndex>(),
        false,
    );

    let price = grapes
        .node_references()
        .map(|(_node_index, nodes)| {
            let area = nodes.len();
            let group_id = nodes
                .first()
                .map(|node| grid.get(node).unwrap_or(&' '))
                .expect("empty group");

            // Now, we should count the edges, but in our
            // case, it is the same thing as the number of
            // corners
            let corners = nodes
                .iter()
                .map(|node| {
                    compute_corners_seen_by_node(
                        &grid, node, group_id,
                    )
                })
                .sum::<usize>();

            area * corners
        })
        .sum::<usize>();

    Ok(price.to_string())
}

fn compute_corners_seen_by_node(
    grid: &HashMap<(i32, i32), char>,
    node: &(i32, i32),
    group_id: &char,
) -> usize {
    let mut count = 0;

    for (dir0, dir1) in
        DIRECTIONS_CW.iter().circular_tuple_windows()
    {
        // dir0 East  North West  South
        // dir1 North West  South East
        // |  1|2  |  1
        // | +|+-+ | +-+
        // |  N-0  |  N|0
        // | + + + |   +
        let in_group =
            |n: &(i32, i32), d: &(i32, i32)| -> bool {
                grid.get(&(n.0 + d.0, n.1 + d.1))
                    .is_some_and(|gid| *gid == *group_id)
            };
        let diag_not_in_group = |n: &(i32, i32)| -> bool {
            grid.get(&(
                n.0 + dir0.0 + dir1.0,
                n.1 + dir0.1 + dir1.1,
            ))
            .is_some_and(|gid| *gid != *group_id)
        };

        let node0 = in_group(node, dir0);
        let node1 = in_group(node, dir1);
        if node0 && node1 && diag_not_in_group(node) {
            count += 1;
        } else if !node0 && !node1 {
            count += 1;
        }
    }

    count
}

fn parse(
    input: &str,
) -> miette::Result<HashMap<(i32, i32), char>> {
    let grid = input
        .lines()
        .enumerate()
        .flat_map(|(y, line)| {
            line.chars().enumerate().map(move |(x, ch)| {
                ((x as i32, y as i32), ch)
            })
        })
        .collect::<HashMap<_, _>>();

    Ok(grid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        "AAAA
BBCD
BBCC
EEEC",
        "80"
    )]
    #[case(
        "OOOOO
OXOXO
OOOOO
OXOXO
OOOOO",
        "436"
    )]
    #[case(
        "EEEEE
EXXXX
EEEEE
EXXXX
EEEEE",
        "236"
    )]
    #[case(
        "AAAAAA
AAABBA
AAABBA
ABBAAA
ABBAAA
AAAAAA",
        "368"
    )]
    #[case(
        "RRRRIICCFF
RRRRIICCCF
VVRRRCCFFF
VVRCCCJFFF
VVVVCJJCFE
VVIVCCJJEE
VVIIICJJEE
MIIIIIJJEE
MIIISIJEEE
MMMISSJEEE",
        "1206"
    )]
    fn test_process(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }
}
