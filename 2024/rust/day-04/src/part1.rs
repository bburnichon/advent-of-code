use glam::IVec2;
use std::collections::HashMap;

const DIRECTIONS: [IVec2; 8] = [
    IVec2::new(1, 0),
    IVec2::new(1, 1),
    IVec2::new(0, 1),
    IVec2::new(-1, 1),
    IVec2::new(-1, 0),
    IVec2::new(-1, -1),
    IVec2::new(0, -1),
    IVec2::new(1, -1),
];

#[tracing::instrument]
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

    let mas = ['M', 'A', 'S'];

    let sum = grid
        .iter()
        .filter(|&(_position, char)| *char == 'X')
        .fold(0, |acc, (position, _char)| {
            acc + DIRECTIONS
                .iter()
                .filter(|&direction| {
                    let mut position = *position;

                    mas.iter().all(|letter| {
                        position += direction;
                        if let Some(char) =
                            grid.get(&position)
                        {
                            return *char == *letter;
                        }

                        false
                    })
                })
                .count()
        });

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
        assert_eq!("18", process(input)?);
        Ok(())
    }
}
