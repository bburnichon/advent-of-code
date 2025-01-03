use glam::IVec2;
use miette::miette;
use nom::{
    bytes::complete::tag,
    character::{complete, complete::line_ending},
    combinator::{all_consuming, map, opt},
    multi::separated_list0,
    sequence::{separated_pair, terminated},
    IResult,
};
use pathfinding::prelude::astar;
use std::collections::HashSet;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, falling_bytes) =
        parse(input).map_err(|err| {
            miette::miette!("Parse error: {}", err)
        })?;

    let grid_size = if cfg!(test) {
        IVec2::splat(6)
    } else {
        IVec2::splat(70)
    };

    let simulation_length =
        if cfg!(test) { 12 } else { 1024 };

    let corrupted_memory: HashSet<IVec2> = falling_bytes
        .iter()
        .take(simulation_length)
        .cloned()
        .collect();

    let result = astar(
        &IVec2::ZERO,
        |pos| {
            [
                pos + IVec2::NEG_Y,
                pos + IVec2::NEG_X,
                pos + IVec2::Y,
                pos + IVec2::X,
            ]
            .into_iter()
            .filter_map(|pos| {
                if ((0..=grid_size.x).contains(&pos.x)
                    && (0..=grid_size.y).contains(&pos.y))
                    && !corrupted_memory.contains(&pos)
                {
                    Some((pos, 1))
                } else {
                    None
                }
            })
        },
        |pos| (grid_size - pos).abs().element_sum(),
        |pos| *pos == grid_size,
    )
    .ok_or_else(|| miette!("No path found"))?;

    Ok(result.1.to_string())
}

fn parse(input: &str) -> IResult<&str, Vec<IVec2>> {
    all_consuming(terminated(
        separated_list0(
            line_ending,
            map(
                separated_pair(
                    complete::i32,
                    tag(","),
                    complete::i32,
                ),
                |(x, y)| IVec2::new(x, y),
            ),
        ),
        opt(line_ending),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        "5,4
4,2
4,5
3,0
2,1
6,3
2,4
1,5
0,6
3,3
2,6
5,1
1,2
5,5
2,5
6,5
1,4
0,4
6,4
1,1
6,1
1,0
0,5
1,6
2,0",
        "22"
    )]
    fn test_process(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }
}
