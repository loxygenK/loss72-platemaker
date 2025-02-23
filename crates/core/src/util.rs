use std::ops::Range;

pub fn get_slice_by_char(str: &str, range: Range<usize>) -> &str {
    let mut char_indicies = str.char_indices();

    let Some((start_pos, _)) = char_indicies.nth(range.start) else {
        return "";
    };

    let Some((end_pos, _)) = char_indicies.nth(range.end - 1) else {
        return &str[start_pos..];
    };

    &str[start_pos..end_pos]
}

#[cfg(test)]
mod tests {
    use super::get_slice_by_char;

    #[test]
    fn slices_correct_amount() {
        assert_eq!(get_slice_by_char("ABCDEFG", 0..3), "ABC");
        assert_eq!(get_slice_by_char("1234567890", 0..4), "1234");

        assert_eq!(get_slice_by_char("こんにちは世界", 0..6), "こんにちは世");
    }

    #[test]
    fn handles_overflow() {
        assert_eq!(get_slice_by_char("1234567890", 10..15), "");
        assert_eq!(get_slice_by_char("1234567890", 5..15), "67890");
    }
}
