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

    let mut position =
        position.expect("position should be defined");
    let mut direction = IVec2::NEG_Y; // Starting going north
    let mut visited = HashSet::new();

    while (0..=map_size.x).contains(&position.x)
        && (0..=map_size.y).contains(&position.y)
    {
        visited.insert(position);

        while objects.contains(&(position + direction)) {
            direction = direction.perp();
        }

        position += direction;
    }

    Ok(visited.len().to_string())
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
        assert_eq!("41", process(input)?);
        Ok(())
    }
}
