use glam::IVec2;
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{line_ending, one_of},
    combinator::all_consuming,
    multi::{many0, separated_list0},
    sequence::{pair, terminated},
    IResult,
};
use nom_locate::{position, LocatedSpan};
use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
    iter::successors,
};

type Span<'a> = LocatedSpan<&'a str>;

const DIRECTIONS: [IVec2; 4] =
    [IVec2::NEG_Y, IVec2::NEG_X, IVec2::Y, IVec2::X];

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, map) =
        parse(Span::new(input)).map_err(|err| {
            miette::miette!("Parse error: {}", err)
        })?;

    let grid_size = map
        .last()
        .map(|(pos, _)| *pos)
        .expect("Grid is empty");
    let max_length =
        grid_size.distance_squared(IVec2::ZERO) as usize;

    let start = map
        .iter()
        .find_map(|(pos, ch)| {
            if *ch == 'S' {
                Some(*pos)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            miette::miette!("Cannot find 'S' in map")
        })?;
    let end = map
        .iter()
        .find_map(|(pos, ch)| {
            if *ch == 'E' {
                Some(*pos)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            miette::miette!("Cannot find 'E' in map")
        })?;

    let walls = map
        .iter()
        .filter_map(|(pos, ch)| {
            if *ch == '#' {
                Some(*pos)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();

    let mut visited = HashSet::new();
    let normal_path = successors(Some(start), |pos| {
        visited.insert(*pos);
        if pos == &end {
            return None;
        }

        DIRECTIONS.into_iter().find_map(|dir| {
            let next_pos = dir + *pos;
            if !walls.contains(&next_pos)
                && !visited.contains(&next_pos)
                && visited.len() < max_length
            {
                Some(next_pos)
            } else {
                None
            }
        })
    })
    .enumerate()
    .map(|(ix, pos)| (pos, ix))
    .collect::<HashMap<_, usize>>();

    if normal_path.is_empty()
        || normal_path.len() >= max_length
    {
        Err(miette::miette!(
            "Maximum length is {}",
            max_length
        ))?;
    }

    let result = normal_path.len() - 1;

    // All walls are connected, so there always exists
    // a path between two pos in same region
    // of the manhattan distance when one crossing is
    // allowed.
    let shortcuts = normal_path
        .iter()
        .tuple_combinations()
        .filter(|((&s, &s_score), (&e, &e_score))| {
            let shortcut_cost =
                (e - s).abs().element_sum() as usize;
            shortcut_cost <= 20
                && max(e_score, s_score)
                    >= min(e_score, s_score)
                        + shortcut_cost
                        + 100
        })
        .count();

    println!("Shortcuts: {:?}", shortcuts);

    Ok(result.to_string())
}

fn parse(input: Span) -> IResult<Span, Vec<(IVec2, char)>> {
    let (input, map) = all_consuming(separated_list0(
        line_ending,
        many0(terminated(
            pair(pos_parse, one_of("#SE")),
            many0(tag(".")),
        )),
    ))(input)?;

    Ok((
        input,
        map.into_iter().flatten().collect(),
    ))
}

fn pos_parse(input: Span) -> IResult<Span, IVec2> {
    let (input, position) = position(input)?;
    let pos = IVec2::new(
        position.location_line() as i32 - 1,
        position.get_column() as i32 - 1,
    );

    Ok((input, pos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        "###############
#...#...#.....#
#.#.#.#.#.###.#
#S#...#.#.#...#
#######.#.#.###
#######.#.#...#
#######.#.###.#
###..E#...#...#
###.#######.###
#...###...#...#
#.#####.#.###.#
#.#...#.#.#...#
#.#.#.#.#.#.###
#...#...#...###
###############",
        "84"
    )]
    fn test_process(
        #[case] input: &str,
        #[case] expected: &str,
    ) -> miette::Result<()> {
        assert_eq!(expected, process(input)?);
        Ok(())
    }
}
