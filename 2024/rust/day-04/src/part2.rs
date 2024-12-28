use glam::IVec2;
use std::{collections::HashMap, iter::zip};

const DIRECTIONS_SLASH: [IVec2; 2] =
    [IVec2::new(1, -1), IVec2::new(-1, 1)];
const DIRECTIONS_BACKSLASH: [IVec2; 2] =
    [IVec2::new(1, 1), IVec2::new(-1, -1)];
const MAS: [char; 2] = ['M', 'S'];

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let grid = input
        .lines()
        .enumerate()
        .flat_map(|(y, line)| {
            line.chars().enumerate().map(move |(x, ch)| {
                (IVec2::new(x as i32, y as i32), ch)
            })
        })
        .collect::<HashMap<_, _>>();

    let sum = grid
        .iter()
        .filter(|&(_position, char)| *char == 'A')
        .filter(|&(position, _char)| {
            let check =
                |(direction, mas): (&IVec2, &char)| {
                    grid.get(&(position + direction))
                        .is_some_and(|actual| {
                            *actual == *mas
                        })
                };

            (zip(DIRECTIONS_SLASH.iter(), MAS.iter())
                .all(check)
                || zip(
                    DIRECTIONS_SLASH.iter(),
                    MAS.iter().rev(),
                )
                .all(check))
                && (zip(
                    DIRECTIONS_BACKSLASH.iter(),
                    MAS.iter(),
                )
                .all(check)
                    || zip(
                        DIRECTIONS_BACKSLASH.iter(),
                        MAS.iter().rev(),
                    )
                    .all(check))
        })
        .count();

    Ok(sum.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX
";
        assert_eq!("9", process(input)?);
        Ok(())
    }
}
