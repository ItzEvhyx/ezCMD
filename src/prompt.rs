use crate::matcher::Hit;
use std::io::{self, BufRead, Write};

/// Interactive chooser when a query produces ambiguous matches.
/// Displays choices 1..=hits.len() and reads user input.
/// Returns Ok(Some(index)) if valid choice made, or Ok(None) if aborted/EOF/invalid after 3 attempts.
pub fn choose<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    hits: &[Hit],
) -> io::Result<Option<usize>> {
    if hits.is_empty() {
        return Ok(None);
    }

    for (i, hit) in hits.iter().enumerate() {
        writeln!(out, "  {}. {}", i + 1, hit.section.title)?;
    }

    let range_str = (1..=hits.len())
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("/");
    let prompt_str = format!("Choose [{}]: ", range_str);

    let mut attempts = 0;
    while attempts < 3 {
        write!(out, "{}", prompt_str)?;
        out.flush()?;

        let mut line = String::new();
        let bytes_read = input.read_line(&mut line)?;
        if bytes_read == 0 {
            // EOF reached
            return Ok(None);
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            // Enter alone selects nothing and re-prompts
            attempts += 1;
            continue;
        }

        if let Ok(choice) = trimmed.parse::<usize>() {
            if choice >= 1 && choice <= hits.len() {
                return Ok(Some(choice - 1));
            }
        }

        attempts += 1;
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::all_tools;
    use std::io::Cursor;

    fn dummy_hits() -> Vec<Hit<'static>> {
        let tool = all_tools().next().expect("seed tool");
        let mut sections = tool.sections();
        let sec1 = sections.next().expect("first section");
        let sec2 = sections.next().expect("second section");
        vec![
            Hit {
                tool,
                section: sec1,
                score: 0.9,
                section_idx: 0,
            },
            Hit {
                tool,
                section: sec2,
                score: 0.85,
                section_idx: 1,
            },
        ]
    }

    #[test]
    fn test_choose_valid_first_try() {
        let hits = dummy_hits();
        let mut input = Cursor::new(b"1\n");
        let mut output = Vec::new();

        let choice = choose(&mut input, &mut output, &hits).expect("choose");
        assert_eq!(choice, Some(0));

        let out_str = String::from_utf8(output).expect("utf8");
        assert!(out_str.contains("  1. "));
        assert!(out_str.contains("  2. "));
        assert!(out_str.contains("Choose [1/2]: "));
    }

    #[test]
    fn test_choose_invalid_then_valid() {
        let hits = dummy_hits();
        let mut input = Cursor::new(b"invalid\n2\n");
        let mut output = Vec::new();

        let choice = choose(&mut input, &mut output, &hits).expect("choose");
        assert_eq!(choice, Some(1));
    }

    #[test]
    fn test_choose_three_invalid_attempts() {
        let hits = dummy_hits();
        let mut input = Cursor::new(b"foo\nbar\nbaz\n");
        let mut output = Vec::new();

        let choice = choose(&mut input, &mut output, &hits).expect("choose");
        assert_eq!(choice, None);
    }

    #[test]
    fn test_choose_enter_alone_reprompts() {
        let hits = dummy_hits();
        let mut input = Cursor::new(b"\n\n\n");
        let mut output = Vec::new();

        let choice = choose(&mut input, &mut output, &hits).expect("choose");
        assert_eq!(choice, None);
    }

    #[test]
    fn test_choose_eof() {
        let hits = dummy_hits();
        let mut input = Cursor::new(b"");
        let mut output = Vec::new();

        let choice = choose(&mut input, &mut output, &hits).expect("choose");
        assert_eq!(choice, None);
    }
}
