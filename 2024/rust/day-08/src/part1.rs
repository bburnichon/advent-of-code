use glam::IVec2;
use itertools::Itertools;
use std::collections::HashMap;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let mut grid_size = IVec2::default();

    let antennas = input
        .lines()
        .enumerate()
        .flat_map(|(y, line)| {
            grid_size.y += 1;
            grid_size.x = line.len() as i32;

            line.chars().enumerate().filter_map(
                move |(x, ch)| {
                    if ch != '.' {
                        Some((
                            ch,
                            IVec2::new(x as i32, y as i32),
                        ))
                    } else {
                        None
                    }
                },
            )
        })
        .fold(
            HashMap::new(),
            |mut acc: HashMap<_, Vec<_>>,
             (ch, position)| {
                acc.entry(ch).or_default().push(position);
                acc
            },
        );

    let antinodes: usize = antennas
        .iter()
        .flat_map(|(_ch, positions)| {
            positions
                .iter()
                .combinations(2)
                .flat_map(|pos| {
                    let diff = pos[1] - pos[0];
                    [pos[0] - diff, pos[1] + diff]
                })
                .filter(|position| {
                    position.cmpge(IVec2::ZERO).all()
                        && position.cmplt(grid_size).all()
                })
        })
        .unique()
        .count();

    // 353: too high
    Ok(antinodes.to_string())
}

#[allow(dead_code)]
fn display_grid(
    grid_size: IVec2,
    antennas: &HashMap<char, Vec<IVec2>>,
    isolated_antinodes: &[IVec2],
) {
    let mut lines =
        Vec::with_capacity(grid_size.y as usize);
    for _ in 0..grid_size.y {
        lines.push(vec!['.'; grid_size.x as usize]);
    }

    for (ch, pos) in antennas {
        for position in pos {
            lines[position.y as usize]
                [position.x as usize] = *ch;
        }
    }
    for position in isolated_antinodes {
        lines[position.y as usize][position.x as usize] =
            '#';
    }

    println!("grid size: {grid_size}");
    lines.iter().for_each(|line| {
        println!("{}", line.iter().collect::<String>());
    });
    println!("\n\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "............
........0...
.....0......
.......0....
....0.......
......A.....
............
............
........A...
.........A..
............
............";
        assert_eq!("14", process(input)?);
        Ok(())
    }
}
