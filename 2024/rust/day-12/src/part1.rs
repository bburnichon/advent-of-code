use miette::miette;
use petgraph::{
    adj::NodeIndex, algo::condensation,
    graphmap::UnGraphMap, visit::IntoNodeReferences,
};
use std::collections::HashMap;

const DIRECTIONS: [(i32, i32); 4] =
    [(0, 1), (1, 0), (0, -1), (-1, 0)];

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let grid = parse(input)
        .map_err(|err| miette!("Parse error: {}", err))?;

    let mut graph = UnGraphMap::new();

    for (pos, c) in grid.iter() {
        graph.add_node(*pos);

        for dir in DIRECTIONS.iter() {
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
            // When on perimeter, there are fewer neighbors
            let perimeter = nodes
                .iter()
                .map(|node| {
                    4 - graph.neighbors(*node).count()
                })
                .sum::<usize>();

            area * perimeter
        })
        .sum::<usize>();

    Ok(price.to_string())
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
        "140"
    )]
    #[case(
        "OOOOO
OXOXO
OOOOO
OXOXO
OOOOO",
        "772"
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
        "1930"
    )]
    fn test_process(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }
}
