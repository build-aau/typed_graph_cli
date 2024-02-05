use std::ops::Deref;

use build_script_lang::schema::Fields;

use crate::schema::*;
use crate::traits::ChangeSetBuilder;
use crate::ChangeSetError;
use crate::ChangeSetResult;

impl<I> ChangeSetBuilder<I> for Fields<I>
where
    I: Clone + Default + PartialEq,
{
    fn build_changeset_with_path(
        &self,
        new_version: &Self,
        path: Option<FieldPath<I>>,
    ) -> ChangeSetResult<ChangeSet<I>> {
        let path = path.ok_or_else(|| ChangeSetError::MissingFieldPath)?;

        let mut changes: Vec<_> = new_version
            .iter()
            .filter(|field_value| !self.has_field(field_value.name.as_str()))
            .map(|field_value| {
                SingleChange::AddedField(AddedField {
                    comments: field_value.comments.get_doc_comments(),
                    visibility: field_value.visibility,
                    field_path: path.push(field_value.name.clone()),
                    field_type: field_value.field_type.clone(),
                    order: field_value.order
                })
            })
            .collect();

        let removed_fields = self
            .iter()
            .filter(|field_value| !new_version.has_field(field_value.name.as_str()))
            .map(|field_value| {
                SingleChange::RemovedField(RemovedField {
                    field_path: path.push(field_value.name.clone()),
                })
            });

        let edited_fields = self
            .iter()
            .filter_map(|old_type| {
                new_version
                    .get_field(old_type.name.as_str())
                    .map(|new_type| (old_type, new_type))
            })
            .filter(|(old_type, new_type)| old_type != new_type)
            .map(|(old_type, new_type)| {
                SingleChange::EditedFieldType(EditedField {
                    field_path: path.push(old_type.name.clone()),
                    comments: new_type.comments.get_doc_comments(),
                    old_visibility: old_type.visibility,
                    new_visibility: new_type.visibility,
                    old_type: old_type.field_type.clone(),
                    new_type: new_type.field_type.clone(),
                    old_order: old_type.order,
                    new_order: new_type.order,
                })
            });

        changes.extend(removed_fields);
        changes.extend(edited_fields);

        Ok(ChangeSet {
            new_hash: Default::default(),
            old_hash: Default::default(),
            handler: None,
            new_version: Default::default(),
            old_version: Default::default(),
            changes,
        })
    }
}
