use glam::IVec2;
use miette::miette;
use nom::{
    character::complete::{anychar, line_ending},
    combinator::{all_consuming, map_opt},
    multi::{many0, separated_list0},
    IResult,
};
use nom_locate::{position, LocatedSpan};
use pathfinding::prelude::count_paths;
use std::{
    collections::{HashMap, HashSet},
    iter::repeat,
};

type Span<'a> = LocatedSpan<&'a str>;
type Grid = HashMap<IVec2, u32>;

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, grid) = parse(Span::new(input))
        .map_err(|err| miette!("Parse error: {}", err))?;

    let count: usize = grid
        .iter()
        .filter(|&(_pos, height)| *height == 0)
        .map(|(pos, _height)| {
            (pos, trailhead_search(&grid, pos))
        })
        .flat_map(|(start, ends)| {
            repeat(*start).zip(ends.into_iter())
        })
        .map(|(start, end)| {
            trailhead_rate(&grid, &start, &end)
        })
        .sum();

    Ok(count.to_string())
}

fn trailhead_rate(
    grid: &Grid,
    start: &IVec2,
    end: &IVec2,
) -> usize {
    count_paths(
        *start,
        |current_pos| trail_successors(grid, current_pos),
        |current_pos| *current_pos == *end,
    )
}

fn trailhead_search(
    grid: &Grid,
    start: &IVec2,
) -> Vec<IVec2> {
    let mut visited = HashSet::new();
    let mut to_visit = HashSet::from([*start]);

    while !to_visit.is_empty() {
        to_visit = to_visit
            .iter()
            .flat_map(|current_pos| {
                visited.insert(*current_pos);

                trail_successors(grid, current_pos)
            })
            .collect();
    }

    visited
        .into_iter()
        .filter(|pos| {
            grid.get(pos).is_some_and(|height| *height == 9)
        })
        .collect()
}

fn trail_successors<'a>(
    grid: &'a Grid,
    pos: &IVec2,
) -> impl Iterator<Item = IVec2> + use<'a> {
    let pos = *pos;
    let current_height =
        grid.get(&pos).unwrap_or(&u32::MAX);

    [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y]
        .into_iter()
        .map(move |direction| pos + direction)
        .filter(|next_step| {
            grid.get(next_step).is_some_and(|height| {
                *height == *current_height + 1
            })
        })
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
        assert_eq!("81", process(input)?);
        Ok(())
    }
}
