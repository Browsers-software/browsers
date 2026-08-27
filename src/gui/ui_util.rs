pub fn ellipsize(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }
    text[0..max_length - 1].to_string() + "…"
}

// updates a Slint list model in place instead of replacing it wholesale - swapping out the
// ModelRc itself makes Slint recreate every row's component, which resets per-row state like a
// LineEdit's cursor position.
//
// model has to already be backed by a VecModel, and never replaced after that
pub fn sync_vec_model<T: Clone + 'static>(model: &slint::ModelRc<T>, new_items: Vec<T>) {
    use slint::Model;

    let vec_model = model
        .as_any()
        .downcast_ref::<slint::VecModel<T>>()
        .expect("model set by us should always be a VecModel");
    let old_len = vec_model.row_count();
    let new_len = new_items.len();
    for (i, item) in new_items.into_iter().enumerate() {
        if i < old_len {
            vec_model.set_row_data(i, item);
        } else {
            vec_model.push(item);
        }
    }
    for i in (new_len..old_len).rev() {
        vec_model.remove(i);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipsize_shorter() {
        assert_eq!(ellipsize("some text", 8), "some te…");
    }

    #[test]
    fn test_ellipsize_enough() {
        assert_eq!(ellipsize("some text", 9), "some text");
    }
}
