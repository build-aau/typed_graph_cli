use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use build_changeset_lang::{ChangeSet, FieldPath, SingleChange};
use build_script_shared::parsers::{Generics, Ident, Mark, Types};
use std::fmt::Write;

use crate::{GenResult, ToRustType};

pub fn get_generic_changes<'a, I: PartialEq + Clone>(
    type_name: &'a Ident<I>, 
    generics: &'a Generics<I>, 
    changeset: &'a ChangeSet<I>
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

pub fn get_generic_field_type_changes<I: Ord + Default + Clone>(
    type_name: &Ident<I>,
    generics: &Generics<I>,
    new_generics: &Vec<&Ident<I>>, 
    old_generics: &Vec<&Ident<I>>,
    changeset: &ChangeSet<I>
) -> (BTreeMap<Types<I>, BTreeSet<Types<I>>>, BTreeSet<String>) {
    let changes = changeset.get_changes(FieldPath::new(type_name.clone()));

    let mut into_mapping: BTreeMap<Types<I>, BTreeSet<Types<I>>> = BTreeMap::new();
    let mut default_mapping = BTreeSet::new();

    // Create mapping from generic to generic
    for generic in new_generics {
        if !old_generics.contains(generic) {
            continue;
        }

        let old_ident: Ident<I> = Ident::new_alone(format!("{}Old", generic));
        let new_ident: Ident<I> = Ident::new_alone(format!("{}New", generic));

        let old_types = Types::Reference { 
            inner: old_ident, 
            generics: Default::default(), 
            marker: Mark::null() 
        };
        let new_types = Types::Reference { 
            inner: new_ident, 
            generics: Default::default(), 
            marker: Mark::null() 
        };

        into_mapping
            .entry(old_types)
            .or_default()
            .insert(new_types);
    }

    let available_generics: HashSet<_> = generics
        .generics
        .iter()
        .map(|g| &g.letter)
        .cloned()
        .collect();

    for change in &changes {
        // Create mapping from one field to another
        if let SingleChange::EditedFieldType(f) = change {
            let new_field_type = f.new_type().clone();
            let old_field_type = f.old_type().clone();

            let mut new_generics = available_generics.clone();
            new_field_type.remove_used(&mut new_generics);

            let mut old_generics = available_generics.clone();
            old_field_type.remove_used(&mut old_generics);

            let new_field_type = new_field_type
                .map_reference(|ident| {
                    if available_generics.contains(&&ident) {
                        Ident::new_alone(format!("{ident}New"))
                    } else {
                        ident
                    }
                });

            let old_field_type = old_field_type
                .map_reference(|ident| {
                    if available_generics.contains(&&ident) {
                        Ident::new_alone(format!("{ident}Old"))
                    } else {
                        ident
                    }
                });
            
            into_mapping
                .entry(old_field_type)
                .or_default()
                .insert(new_field_type);
        }

        // Create mapping from no instance to one
        // By figure out which generics need a Default implementation
        if let SingleChange::AddedField(f) = change {
            let new_field_type = f.field_type();
            
            let mut available_generics = available_generics.clone();
            new_field_type.remove_used(&mut available_generics);
            let used_generics = available_generics
                .iter()
                .filter(|g| available_generics.contains(g))
                .map(|letter| format!("{letter}New"));

            default_mapping.extend(used_generics)
        }
    }

    (into_mapping, default_mapping)
}

pub fn create_generics<I: PartialEq + Ord + Clone + Default>(
    type_name: &Ident<I>,
    generics: &Generics<I>,
    changeset: &ChangeSet<I>
) -> GenResult<(String, String, String, String)> 
where
    Types<I>: ToRustType
{
    let (old_generics, new_generics) = get_generic_changes(type_name, generics, changeset);

    let old_generic_letters: Vec<_> = old_generics.iter().map(|g| format!("{}Old", g)).collect();
    let new_generic_letters: Vec<_> = new_generics.iter().map(|g| format!("{}New", g)).collect();
    let all_generics: Vec<_> = old_generic_letters.iter().chain(new_generic_letters.iter()).cloned().collect();

    let fmt_old_generic = old_generic_letters.join(", ");
    let fmt_new_generic = new_generic_letters.join(", ");
    let fmt_all_generic = all_generics.join(", ");

    let impl_generics = if !fmt_all_generic.is_empty() {
        format!("<{fmt_all_generic}>")
    } else {
        "".to_string()
    };

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

    let (into_mapping, default_mapping) = get_generic_field_type_changes(
        type_name, 
        generics, 
        &new_generics, 
        &old_generics, 
        changeset
    );

    // Build where clause
    let end_bracket = if !into_mapping.is_empty() || !default_mapping.is_empty() {
        let mut end_bracket = String::new();
        writeln!(end_bracket)?;
        writeln!(end_bracket, "where")?;
        for default in default_mapping {
            writeln!(end_bracket, "    {default}: Default,")?;
        }
        for (from_type, to_types) in into_mapping {
            let into_impl = to_types
                .into_iter()
                .map(|ty| format!("Into<{}>", ty.to_rust_type()))
                .collect::<Vec<_>>()
                .join(" + ");

            let rust_from_type = from_type.to_rust_type();
            writeln!(end_bracket, "    {rust_from_type}: {into_impl},")?;
        }
        writeln!(end_bracket, "{{")?;

        end_bracket
    } else {
        " {{".to_string()
    };

    Ok((end_bracket, new_type_generics, old_type_generics, impl_generics))
}