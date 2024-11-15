use std::fmt::Display;
use std::fmt::Write;

#[derive(Default, Debug)]
pub struct CodePreview {
    preview: String,
    line_number_start: usize,
    line_number_end: usize,
    caret_line_number: usize,
    caret_offset: usize,
}

impl CodePreview {
    pub fn new_at(
        data: &str,
        mut line: usize,
        mut column: usize,
        lines_above: usize,
        lines_below: usize,
        zero_indexed: bool
    ) -> CodePreview {
        if zero_indexed {
            line -= 1;
            column -= 1;
        }
        let line_offset: usize = data.split("\n").take(line).map(|l| l.len() + 1).sum();
        let caret_offset = line_offset + column;
        CodePreview::new(data, caret_offset, 1, lines_above, lines_below, zero_indexed)
    }

    pub fn new(
        data: &str,
        mut caret_offset: usize,
        mut caret_len: usize,
        lines_above: usize,
        lines_below: usize,
        zero_indexed: bool
    ) -> CodePreview {
        if data.is_empty() {
            return CodePreview::default();
        }
        if caret_offset >= data.len() {
            caret_offset = data.len() - 1;
        }
        if caret_len + caret_offset > data.len() {
            caret_len = (data.len() - caret_offset).max(1);
        }
        let lines: Vec<_> = data.split("\n").collect();

        let mut caret_line_number = lines.len() - 1;
        let mut caret_line_start_idx = 0;

        for (i, line) in lines.iter().enumerate() {
            let line_len = line.len() + 1;
            let caret_line_end = caret_line_start_idx + line_len;
            if caret_line_end > caret_offset {
                caret_line_number = i;
                break;
            }
            caret_line_start_idx += line_len;
        }

        if caret_offset != caret_offset {
            caret_line_start_idx = 0;
        }

        let preview_start_idx = if lines_above > caret_line_number {
            0
        } else {
            caret_line_number - lines_above
        };

        let preview_lines = lines[preview_start_idx..].iter().enumerate();

        let mut preview_builder = Vec::new();
        let mut remaining_caret_length = caret_len;

        let mut line_offset = lines[..preview_start_idx]
            .iter()
            .fold(0, |acc, l| acc + l.len() + 1);
        let indexing_offset = if zero_indexed { 0 } else { 1 };

        let mut lines_left = lines_above - 1 + lines_below - 1 + 1;
        let mut total_lines = 0;
        for (i, line) in preview_lines {
            let line_len = line.len() + 1;
            preview_builder.push(format!("{:>5} | {}", preview_start_idx + i + indexing_offset, line));
            let line_idx = i + preview_start_idx;
            if line_idx >= caret_line_number && remaining_caret_length != 0 {
                if line_idx == caret_line_number {
                    let caret_indentation = caret_offset - line_offset;
                    let used_caret_len = (line_len - caret_indentation)
                        .min(remaining_caret_length)
                        .max(1);
                    let caret: String = (0..used_caret_len).map(|_| "^".to_string()).collect();
                    let indentation: String =
                        (0..caret_indentation).map(|_| " ".to_string()).collect();

                    if remaining_caret_length >= used_caret_len {
                        remaining_caret_length -= used_caret_len;
                    } else {
                        remaining_caret_length = 0;
                    }

                    preview_builder.push(format!("      | {}{}", indentation, caret));
                } else {
                    let used_caret_len = line_len.min(remaining_caret_length);
                    let caret: String = (0..used_caret_len).map(|_| "^".to_string()).collect();
                    remaining_caret_length -= used_caret_len;
                    preview_builder.push(format!("      | {}", caret));
                }
            } else {
                if lines_left == 0 {
                    break;
                }
                lines_left -= 1;
            }

            total_lines += 1;
            line_offset += line_len;
        }

        CodePreview {
            preview: preview_builder.join("\n"),
            line_number_start: preview_start_idx,
            line_number_end: preview_start_idx + total_lines,
            caret_line_number: caret_line_number,
            caret_offset: caret_offset - caret_line_start_idx,
        }
    }

    pub fn line_number_start(&self) -> usize {
        self.line_number_start
    }
    pub fn line_number_end(&self) -> usize {
        self.line_number_end
    }
    pub fn caret_line_number(&self) -> usize {
        self.caret_line_number
    }
    pub fn caret_offset(&self) -> usize {
        self.caret_offset
    }

    pub fn showcase(s: String) -> String {
        s.split('\n')
            .enumerate()
            .map(|(nr, line)| format!("{nr:>5} | {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn diff_string(left: &str, right: &str, zero_indexed: bool) -> String {
        let char_iter = left.chars().zip(right.chars()).enumerate();
        let mut caret_pos = None;
        for (i, (l, r)) in char_iter {
            if l != r {
                caret_pos = Some(i);
                break;
            }
        }

        if caret_pos.is_none() && left.len() != right.len() {
            caret_pos = Some(left.len().min(right.len()));
        }

        let mut str_builder = String::new();
        if let Some(caret_offset) = caret_pos {
            let _ = writeln!(&mut str_builder, "Left:");
            let preview = CodePreview::new(left, caret_offset, 1, 2, 2, zero_indexed);
            let _ = writeln!(&mut str_builder, "{}", preview.preview);
            let _ = writeln!(&mut str_builder);
            let _ = writeln!(&mut str_builder, "Right:");
            let preview = CodePreview::new(right, caret_offset, 1, 2, 2, zero_indexed);
            let _ = writeln!(&mut str_builder, "{}", preview.preview);
        } else {
            let _ = writeln!(&mut str_builder, "Found no difference");
            let _ = writeln!(&mut str_builder);
        }

        str_builder
    }
}

impl Display for CodePreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.preview)
    }
}

#[test]
fn preview_test_wide_offset_1() {
    let preview = CodePreview::new("a\na\ns", 2, 2, 2, 2, false);
    assert_eq!(
        preview.preview,
        "    0 | a
    1 | a
      | ^^
    2 | s"
    );
}

#[test]
fn preview_test_wide_offset_2() {
    let preview = CodePreview::new("a\na\ns", 0, 2, 2, 2, false);
    assert_eq!(
        preview.preview,
        "    0 | a
      | ^^
    1 | a
    2 | s"
    );
}

#[test]
fn preview_test_len_1() {
    let preview = CodePreview::new("a\na\ns", 2, 4, 2, 2, false);
    assert_eq!(
        preview.preview,
        "    0 | a
    1 | a
      | ^^
    2 | s
      | ^"
    );
}

#[test]
fn preview_test_offset_1() {
    let preview = CodePreview::new("a\na\ns", 0, 1, 2, 2, false);
    assert_eq!(
        preview.preview,
        "    0 | a
      | ^
    1 | a
    2 | s"
    );
}

#[test]
fn preview_test_offset_2() {
    let preview = CodePreview::new("a\na\ns", 1, 1, 2, 2, false);
    assert_eq!(
        preview.preview,
        "    0 | a
      |  ^
    1 | a
    2 | s"
    );
}

#[test]
fn preview_test_offset_3() {
    let preview = CodePreview::new("a\na\ns", 2, 1, 2, 2, false);
    assert_eq!(
        preview.preview,
        "    0 | a
    1 | a
      | ^
    2 | s"
    );
}

#[test]
fn preview_test_offset_4() {
    let preview = CodePreview::new("a\na\ns", 3, 1, 2, 2, false);
    assert_eq!(
        preview.preview,
        "    0 | a
    1 | a
      |  ^
    2 | s"
    );
}

#[test]
fn preview_test_offset_5() {
    let preview = CodePreview::new("a\na\ns", 4, 1, 2, 2, false);
    assert_eq!(
        preview.preview,
        "    0 | a
    1 | a
    2 | s
      | ^"
    );
}

#[test]
fn preview_test_empty_0() {
    let preview = CodePreview::new("\n\n", 0, 1, 2, 2, false);
    assert_eq!(
        preview.preview,
        "    0 | 
      | ^
    1 | 
    2 | "
    );
}

#[test]
fn preview_test_empty_1() {
    let preview = CodePreview::new("\n\n", 1, 1, 2, 2, false);
    assert_eq!(
        preview.preview,
        "    0 | 
    1 | 
      | ^
    2 | "
    );
}

#[test]
fn preview_test_empty_2() {
    let preview = CodePreview::new("\n\n", 500, 1, 2, 2, false);
    assert_eq!(
        preview.preview,
        "    0 | 
    1 | 
      | ^
    2 | "
    );
}

#[test]
fn preview_test_preview_size() {
    let preview = CodePreview::new("0\n1\n2\n3\n4\n5\n6\n7\n8\n9", 8, 1, 2, 2, false);
    assert_eq!(
        preview.preview,
        "    2 | 2
    3 | 3
    4 | 4
      | ^
    5 | 5
    6 | 6"
    );
}

#[test]
fn preview_test_preview_size_below() {
    let preview = CodePreview::new("0\n1\n2\n3\n4\n5\n6\n7\n8\n9", 8, 1, 2, 4, false);
    assert_eq!(
        preview.preview,
        "    2 | 2
    3 | 3
    4 | 4
      | ^
    5 | 5
    6 | 6
    7 | 7
    8 | 8"
    );
}

#[test]
fn preview_test_preview_size_above() {
    let preview = CodePreview::new("0\n1\n2\n3\n4\n5\n6\n7\n8\n9", 8, 1, 4, 2, false);
    assert_eq!(
        preview.preview,
        "    0 | 0
    1 | 1
    2 | 2
    3 | 3
    4 | 4
      | ^
    5 | 5
    6 | 6"
    );
}
