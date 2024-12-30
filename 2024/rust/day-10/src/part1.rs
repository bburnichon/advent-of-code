use glam::IVec2;
use miette::miette;
use nom::{
    character::complete::{anychar, line_ending},
    combinator::{all_consuming, map_opt},
    multi::{many0, separated_list0},
    IResult,
};
use nom_locate::{position, LocatedSpan};
use std::collections::{HashMap, HashSet};

type Span<'a> = LocatedSpan<&'a str>;
type Grid = HashMap<IVec2, u32>;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, grid) = parse(Span::new(input))
        .map_err(|err| miette!("Parse error: {}", err))?;

    let count: usize = grid
        .iter()
        .filter(|&(_pos, height)| *height == 0)
        .map(|(pos, _height)| trailhead_score(&grid, pos))
        .sum();

    Ok(count.to_string())
}

fn trailhead_score(grid: &Grid, start: &IVec2) -> usize {
    let mut visited = HashSet::new();
    let mut to_visit = HashSet::from([*start]);

    while !to_visit.is_empty() {
        to_visit = to_visit
            .iter()
            .flat_map(|current_pos| {
                let current_height = grid
                    .get(current_pos)
                    .unwrap_or(&u32::MAX);
                visited.insert(*current_pos);

                [
                    IVec2::X,
                    IVec2::Y,
                    IVec2::NEG_X,
                    IVec2::NEG_Y,
                ]
                .iter()
                .map(|direction| *current_pos + *direction)
                .filter(|next_step| {
                    !to_visit.contains(next_step)
                        && grid.get(next_step).is_some_and(
                            |height| {
                                *height
                                    == *current_height + 1
                            },
                        )
                })
            })
            .collect();
    }

    visited
        .iter()
        .filter(|pos| {
            grid.get(*pos)
                .is_some_and(|height| *height == 9)
        })
        .count()
}

fn parse(input: Span) -> IResult<Span, Grid> {
    let (input, lines) = all_consuming(separated_list0(
        line_ending,
        many0(pos_height),
    ))(input)?;

    let map =
        lines.iter().flatten().copied().collect::<Grid>();

    Ok((input, map))
}

fn pos_height(input: Span) -> IResult<Span, (IVec2, u32)> {
    let (input, position) = position(input)?;
    let line = position.location_line() as i32 - 1;
    let column = position.get_column() as i32 - 1;
    let (input, height) =
        map_opt(anychar, |c| c.to_digit(10))(input)?;

    Ok((
        input,
        (IVec2::new(column, line), height),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "89010123
78121874
87430965
96549874
45678903
32019012
01329801
10456732";
        assert_eq!("36", process(input)?);
        Ok(())
    }
}
