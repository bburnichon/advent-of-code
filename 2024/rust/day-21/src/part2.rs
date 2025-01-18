use glam::IVec2;
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::OnceLock,
};

#[tracing::instrument(skip(input))]
pub fn process(input: &str) -> miette::Result<String> {
    let codes = parse(input).map_err(|err| {
        miette::miette!("Parse error: {}", err)
    })?;

    let bots = if cfg!(test) { 2 } else { 25 };

    let path_cache = &mut HashMap::new();
    let length_cache = &mut HashMap::new();
    let mut state = State {
        path_cache,
        length_cache,
    };

    let result = codes
        .iter()
        .map(|&code| state.code_complexity(code, bots))
        .sum::<usize>();

    Ok(result.to_string())
}

struct State<'a> {
    path_cache: &'a mut HashMap<
        (char, char),
        (usize, HashSet<String>),
    >,
    length_cache:
        &'a mut HashMap<(char, char, usize), usize>,
}

fn code_val(code: &str) -> usize {
    let first_non_digit = code
        .find(|ch: char| !ch.is_digit(10))
        .unwrap_or(code.len());

    code[..first_non_digit].parse::<usize>().unwrap_or(0)
}

impl<'a> State<'a> {
    fn code_complexity(
        &mut self,
        code: &str,
        depth: usize,
    ) -> usize {
        self.shortest_seq_len(code, depth) * code_val(code)
    }
}

#[derive(Debug, Clone, Copy)]
enum Keypad {
    Numpad,
    Dirpad,
}

static NUMPAD_POSITIONS: OnceLock<(
    HashMap<char, IVec2>,
    HashMap<IVec2, char>,
)> = OnceLock::new();
static DIRPAD_POSITIONS: OnceLock<(
    HashMap<char, IVec2>,
    HashMap<IVec2, char>,
)> = OnceLock::new();

fn to_position(
    positions: &[&str],
) -> (
    HashMap<char, IVec2>,
    HashMap<IVec2, char>,
) {
    let by_char = positions
        .into_iter()
        .enumerate()
        .flat_map(|(row, s)| {
            s.chars().enumerate().filter_map(
                move |(column, ch)| {
                    ch.ne(&' ').then_some((
                        ch,
                        IVec2::new(
                            column as i32,
                            row as i32,
                        ),
                    ))
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let by_pos = by_char
        .iter()
        .map(|(ch, pos)| (*pos, *ch))
        .collect();

    (by_char, by_pos)
}

impl Keypad {
    fn keypad_buttons(&self) -> &[&str] {
        match self {
            Keypad::Numpad => &["789", "456", "123", " 0A"],
            Keypad::Dirpad => &[" ^A", "<v>"],
        }
    }

    fn init_positions(
        &self,
    ) -> &(
        HashMap<char, IVec2>,
        HashMap<IVec2, char>,
    ) {
        match self {
            Keypad::Numpad => {
                NUMPAD_POSITIONS.get_or_init(|| {
                    to_position(self.keypad_buttons())
                })
            }
            Keypad::Dirpad => {
                DIRPAD_POSITIONS.get_or_init(|| {
                    to_position(self.keypad_buttons())
                })
            }
        }
    }

    fn to_position(&self, ch: &char) -> Option<IVec2> {
        let (by_char, _) = self.init_positions();

        by_char.get(ch).copied()
    }

    fn to_char(&self, pos: &IVec2) -> Option<char> {
        let (_, by_pos) = self.init_positions();

        by_pos.get(pos).copied()
    }

    fn shortest_paths(
        &self,
        cache: &mut HashMap<
            (char, char),
            (usize, HashSet<String>),
        >,
        path: (char, char),
    ) -> (usize, HashSet<String>) {
        cache
            .entry(path)
            .or_insert_with(|| {
                // Best way to go from a key to itself is
                // just to press activate
                if path.0 == path.1 {
                    return (
                        1,
                        HashSet::from(["A".into()]),
                    );
                }

                let mut shortest_path_len = usize::MAX;
                let mut paths = HashSet::new();
                let mut queue = VecDeque::new();
                queue.push_back((
                    self.to_position(&path.0)
                        .expect("invalid pushed position"),
                    String::new(),
                ));
                'bfs: while let Some((pos, moves)) =
                    queue.pop_front()
                {
                    for (dir, key) in [
                        (pos + IVec2::NEG_Y, '^'),
                        (pos + IVec2::Y, 'v'),
                        (pos + IVec2::X, '>'),
                        (pos + IVec2::NEG_X, '<'),
                    ] {
                        if let Some(new_ch) =
                            self.to_char(&dir)
                        {
                            if new_ch == path.1 {
                                if shortest_path_len
                                    < moves.len() + 2
                                {
                                    break 'bfs;
                                }
                                shortest_path_len =
                                    moves.len() + 2;
                                paths.insert(format!(
                                    "{}{}A",
                                    moves, key
                                ));
                            } else {
                                queue.push_back((
                                    dir,
                                    format!(
                                        "{}{}",
                                        moves, key
                                    ),
                                ))
                            }
                        };
                    }
                }

                (shortest_path_len, paths)
            })
            .clone()
    }

    fn to_shortest_seqs(
        &self,
        cache: &mut HashMap<
            (char, char),
            (usize, HashSet<String>),
        >,
        code: &str,
    ) -> HashSet<String> {
        ["A", code]
            .into_iter()
            .flat_map(|s| s.chars())
            .tuple_windows::<(char, char)>()
            .map(|(from, to)| {
                self.shortest_paths(cache, (from, to))
                    .1
                    .into_iter()
                    .collect::<Vec<String>>()
            })
            .multi_cartesian_product()
            .map(|path| path.join(""))
            .collect()
    }

    fn shortest_path_len(
        &self,
        path_cache: &mut HashMap<
            (char, char),
            (usize, HashSet<String>),
        >,
        length_cache: &mut HashMap<
            (char, char, usize),
            usize,
        >,
        path: (char, char),
        depth: usize,
    ) -> usize {
        if let Some(length) =
            length_cache.get(&(path.0, path.1, depth))
        {
            return *length;
        }

        if depth == 1 {
            return self.shortest_paths(path_cache, path).0;
        }

        let min_path = self
            .shortest_paths(path_cache, path)
            .1
            .iter()
            .map(|sub_path| {
                self.shortest_seq_len(
                    path_cache,
                    length_cache,
                    sub_path,
                    depth - 1,
                )
            })
            .min()
            .unwrap_or(usize::MAX);

        length_cache
            .insert((path.0, path.1, depth), min_path);
        min_path
    }

    fn shortest_seq_len(
        &self,
        path_cache: &mut HashMap<
            (char, char),
            (usize, HashSet<String>),
        >,
        length_cache: &mut HashMap<
            (char, char, usize),
            usize,
        >,
        code: &str,
        depth: usize,
    ) -> usize {
        ["A", code]
            .into_iter()
            .flat_map(|s| s.chars())
            .tuple_windows::<(char, char)>()
            .map(|(from, to)| {
                self.shortest_path_len(
                    path_cache,
                    length_cache,
                    (from, to),
                    depth,
                )
            })
            .sum::<usize>()
    }
}

impl State<'_> {
    fn shortest_seq_len(
        &mut self,
        code: &str,
        depth: usize,
    ) -> usize {
        let numpad = Keypad::Numpad;
        let seqs =
            numpad.to_shortest_seqs(self.path_cache, code);

        let dirpad = Keypad::Dirpad;
        seqs.iter()
            .map(|code| {
                dirpad.shortest_seq_len(
                    self.path_cache,
                    self.length_cache,
                    code,
                    depth,
                )
            })
            .min()
            .unwrap_or(usize::MAX)
    }
}

fn parse(input: &str) -> Result<Vec<&str>, &'static str> {
    Ok(input.lines().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("029A", 29)]
    #[case("980A", 980)]
    #[case("179A", 179)]
    #[case("456A", 456)]
    #[case("379A", 379)]
    fn test_code_val(
        #[case] input: &str,
        #[case] expected: usize,
    ) -> miette::Result<()> {
        assert_eq!(expected, code_val(input));
        Ok(())
    }

    #[rstest]
    #[case("029A", 68)]
    #[case("980A", 60)]
    #[case("179A", 68)]
    #[case("456A", 64)]
    #[case("379A", 64)]
    fn test_code_shortest_len(
        #[case] input: &str,
        #[case] expected: usize,
    ) -> miette::Result<()> {
        let mut state = State {
            path_cache: &mut HashMap::new(),
            length_cache: &mut HashMap::new(),
        };
        assert_eq!(
            expected,
            state.shortest_seq_len(input, 2)
        );
        Ok(())
    }

    #[rstest]
    #[case("029A", 68*29)]
    #[case("980A", 60*980)]
    #[case("179A", 68*179)]
    #[case("456A", 64*456)]
    #[case("379A", 64*379)]
    #[case(
        "029A
980A
179A
456A
379A",
        126384
    )]
    fn test_process(
        #[case] input: &str,
        #[case] expected: u32,
    ) -> miette::Result<()> {
        assert_eq!(
            expected.to_string().as_str(),
            process(input)?
        );
        Ok(())
    }
}
