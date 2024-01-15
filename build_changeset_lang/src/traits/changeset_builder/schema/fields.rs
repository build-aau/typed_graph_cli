use build_script_lang::schema::Fields;

use crate::ChangeSetError;
use crate::ChangeSetResult;
use crate::schema::*;
use crate::traits::ChangeSetBuilder;

impl<I> ChangeSetBuilder<I> for Fields<I>
where
    I: Clone + Default + PartialEq
{
    fn build_changeset_with_path(
        &self,
        new_version: &Self,
        path: Option<FieldPath<I>>,
    ) -> ChangeSetResult<ChangeSet<I>> {
        let path = path.ok_or_else(|| {
            ChangeSetError::MissingFieldPath
        })?;

        let mut changes: Vec<_> = new_version
            .fields
            .iter()
            .filter(|(field, _)| !self.fields.contains_key(field))
            .map(|(field, field_value)| SingleChange::AddedField(AddedField{
                comments: field_value.comments.get_doc_comments(),
                visibility: field_value.visibility,
                field_path: path.push(field.clone()),
                field_type: field_value.ty.clone(),
            }))
            .collect();

        let removed_fields = self
            .fields
            .iter()
            .filter(|(field, _)| !new_version.fields.contains_key(field))
            .map(|(field, _)| SingleChange::RemovedField(RemovedField{
                field_path: path.push(field.clone()),
            }));

        let edited_fields = self
            .fields
            .iter()
            .filter_map(|(field, old_type)| {
                new_version
                    .fields
                    .get(field)
                    .map(|new_type| (field, old_type, new_type))
            })
            .filter(|(_, old_type, new_type)| old_type != new_type)
            .map(
                |(field, old_type, new_type)| SingleChange::EditedFieldType(EditedField{
                    field_path: path.push(field.clone()),
                    old_visibility: old_type.visibility,
                    new_visibility: new_type.visibility,
                    old_type: old_type.ty.clone(),
                    new_type: new_type.ty.clone(),
                }),
            );

        changes.extend(removed_fields);
        changes.extend(edited_fields);

        Ok(ChangeSet {
            new_hash: Default::default(),
            old_hash: Default::default(),
            handler: None,
            new_version: Default::default(),
            old_version: Default::default(),
            changes
        })
    }
}
