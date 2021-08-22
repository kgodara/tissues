use unicode_segmentation::UnicodeSegmentation;

const ELLIPSIS_LEN: usize = 2;

// Accepts:
//     content - string 
// Returns:
//     Some(String) with the last ELLIPSIS_LEN chars set to ellipsis
//     None if String grapheme_len < ELLIPSIS_LEN
pub fn set_str_end_as_ellipsis(content: &str) -> Option<String> {

    // debug!("set_str_end_as_ellipsis - content: {:?}", content);

    let grapheme_len: usize = content
        .graphemes(true)
        .count();

    if grapheme_len < ELLIPSIS_LEN {
        // debug!("set_str_end_as_ellipsis - content, result_str: {:?}, None", content);
        return None;
    }

    let mut result_str: String = "".to_owned();

    for (idx, g) in content.graphemes(true).enumerate() {
        // are we in the range of chars to be replaced by ellipsis
        // debug!("set_str_end_as_ellipsis - {:?} - ({:?}+1) <= {:?}", grapheme_len, idx, ELLIPSIS_LEN);
        if grapheme_len - (idx+1) < ELLIPSIS_LEN {
            result_str.push('.');
        } else {
            result_str.push_str(g);
        }
    }

    // debug!("set_str_end_as_ellipsis - content, result_str: {:?}, {:?}", content, result_str);

    Some(result_str)

}