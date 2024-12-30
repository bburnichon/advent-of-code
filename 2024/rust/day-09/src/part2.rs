use itertools::Itertools;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
struct File {
    length: usize,
    file_id: usize,
}

impl File {
    fn checksum(&self, offset: usize) -> usize {
        self.file_id
            * self.length
            * (2 * offset + self.length - 1)
            / 2
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let lengths = input
        .trim_end()
        .chars()
        .map(|ch| {
            ch.to_digit(10).unwrap_or_else(|| {
                // clippy does not like expect inside
                // closure
                panic!("'{}' is not a digit", ch)
            })
        })
        .collect::<Vec<_>>();

    let mut offset = 0;
    let mut files = lengths.iter().enumerate().fold(
        Vec::new(),
        |mut files, (i, len)| {
            if i % 2 == 0 {
                files.push((
                    offset,
                    File {
                        length: *len as usize,
                        file_id: i / 2,
                    },
                ));
            }
            offset += *len as usize;

            files
        },
    );

    let mut map = BTreeMap::from_iter(
        files
            .iter()
            .map(|(offset, file)| (*offset, file.length)),
    );

    for (offset, file) in files.iter_mut().rev() {
        if let Some(space) = map
            .iter()
            .tuple_windows()
            .find_map(|(left, right)| {
                let space_offset = *left.0 + *left.1;
                let space_len = *right.0 - space_offset;

                if space_offset < *offset
                    && space_len >= file.length
                {
                    Some(space_offset)
                } else {
                    None
                }
            })
        {
            map.remove(offset);
            *offset = space;
            map.insert(space, file.length);
        }
    }

    let checksum = files
        .iter()
        .map(|(offset, file)| file.checksum(*offset))
        .sum::<usize>();

    Ok(checksum.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "2333133121414131402";
        assert_eq!("2858", process(input)?);
        Ok(())
    }
}
