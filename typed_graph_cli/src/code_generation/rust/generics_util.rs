use build_changeset_lang::{ChangeSet, FieldPath, SingleChange};
use build_script_shared::parsers::{Generics, Ident, Types};

use crate::{GenResult, ToRustType};

pub fn get_generic_changes<'a, I: PartialEq + Clone>(
    type_name: &'a Ident<I>,
    generics: &'a Generics<I>,
    changeset: &'a ChangeSet<I>,
) -> (Vec<&'a Ident<I>>, Vec<&'a Ident<I>>) {
    let changes = changeset.get_changes(FieldPath::new(type_name.clone()));

    let mut old_generics = Vec::new();

    // Check if the old generics were different
    for change in &changes {
        if let SingleChange::EditedGenerics(g) = change {
            for old_generic in &g.old_generics.generics {
                old_generics.push(&old_generic.letter);
            }
        }
    }

    // If not they must be the same
    if old_generics.is_empty() {
        for generic in &generics.generics {
            old_generics.push(&generic.letter);
        }
    }

    // Retrieve new generics
    let mut new_generics = Vec::new();
    for generic in &generics.generics {
        new_generics.push(&generic.letter);
    }

    (old_generics, new_generics)
}

pub fn create_generics<I: PartialEq + Ord + Clone + Default>(
    type_name: &Ident<I>,
    generics: &Generics<I>,
    changeset: &ChangeSet<I>,
) -> GenResult<(String, String)>
where
    Types<I>: ToRustType<I>,
{
    let (old_generics, new_generics) = get_generic_changes(type_name, generics, changeset);

    let old_generic_letters: Vec<_> = old_generics.iter().map(|g| format!("{}Old", g)).collect();
    let new_generic_letters: Vec<_> = new_generics.iter().map(|g| format!("{}New", g)).collect();

    let fmt_old_generic = old_generic_letters.join(", ");
    let fmt_new_generic = new_generic_letters.join(", ");

    let old_type_generics = if !fmt_old_generic.is_empty() {
        format!("<{fmt_old_generic}>")
    } else {
        "".to_string()
    };

    let new_type_generics = if !fmt_new_generic.is_empty() {
        format!("<{fmt_new_generic}>")
    } else {
        "".to_string()
    };

    Ok((
        new_type_generics,
        old_type_generics,
    ))
}
