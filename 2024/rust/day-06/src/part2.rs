use glam::IVec2;
use miette::miette;
use nom::{
    character::complete::{line_ending, one_of},
    multi::{many1, separated_list1},
    IResult,
};
use nom_locate::LocatedSpan;
use std::collections::HashSet;

type Span<'a> = LocatedSpan<&'a str>;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, (map_size, objects, position)) =
        parse(Span::new(input)).map_err(|err| {
            miette!("Parse error: {}", err)
        })?;

    let start_position =
        position.expect("position should be defined");
    let mut position = start_position;
    let mut direction = IVec2::NEG_Y; // Starting going north
    let objects = HashSet::from_iter(objects.into_iter());

    let visited = check_path(
        map_size,
        &objects,
        &mut position,
        &mut direction,
    );

    let mut without_direction = visited
        .iter()
        .map(|(pos, _)| *pos)
        .collect::<HashSet<_>>();

    without_direction.remove(&start_position);

    let count = without_direction
        .iter()
        .filter(|tested_position| {
            let mut position = start_position;
            let mut direction = IVec2::NEG_Y;

            let mut objects_with_one_added =
                objects.clone();
            objects_with_one_added
                .insert(**tested_position);

            let _ = check_path(
                map_size,
                &objects_with_one_added,
                &mut position,
                &mut direction,
            );

            let last_position = position + direction;
            (0..=map_size.x).contains(&last_position.x)
                && (0..=map_size.y)
                    .contains(&last_position.y)
        })
        .count();

    // 1790 too high
    // 1687 too low
    Ok(count.to_string())
}

fn check_path(
    map_size: IVec2,
    objects: &HashSet<IVec2>,
    position: &mut IVec2,
    direction: &mut IVec2,
) -> HashSet<(IVec2, IVec2)> {
    let mut visited = HashSet::new();

    while (0..=map_size.x).contains(&position.x)
        && (0..=map_size.y).contains(&position.y)
        && !visited.contains(&(*position, *direction))
    {
        visited.insert((*position, *direction));

        while objects.contains(&(*position + *direction)) {
            *direction = direction.perp();
        }

        *position += *direction;
    }
    visited
}

fn parse(
    input: Span,
) -> IResult<Span, (IVec2, Vec<IVec2>, Option<IVec2>)> {
    let (input, objects) =
        separated_list1(line_ending, many1(token))(input)?;

    let size = objects
        .iter()
        .flatten()
        .last()
        .and_then(|&(last, _)| Some(last))
        .unwrap_or_default();

    let walls = objects
        .iter()
        .flatten()
        .filter_map(|&(position, tag)| {
            if tag == '#' {
                Some(position)
            } else {
                None
            }
        })
        .collect();

    let position = objects.iter().flatten().find_map(
        |&(position, tag)| {
            if tag == '^' {
                Some(position)
            } else {
                None
            }
        },
    );

    Ok((input, (size, walls, position)))
}

fn token(input: Span) -> IResult<Span, (IVec2, char)> {
    let location = IVec2::new(
        input.get_column() as i32 - 1,
        input.location_line() as i32 - 1,
    );

    let (input, token) = one_of(".^#")(input)?;

    Ok((input, (location, token)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
#.........
......#...";
        assert_eq!("6", process(input)?);
        Ok(())
    }
}
