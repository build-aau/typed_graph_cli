use crate::schema::{ChangeSet, FieldPath};
use crate::ChangeSetResult;

pub trait ChangeSetBuilder<I> {
    /// Build a changeset between two versions of Self
    fn build_changeset(&self, new_version: &Self) -> ChangeSetResult<ChangeSet<I>> {
        self.build_changeset_with_path(new_version, None)
    }

    /// Build a changeset between two versions of Self
    /// with information about where Self is located in the parent type
    ///
    /// This allows types such as Fields to create a full path to the specific field that was changed
    ///
    /// For some type providing a path may fail as they do not expect to be a child of another type
    fn build_changeset_with_path(
        &self,
        new_version: &Self,
        path: Option<FieldPath<I>>,
    ) -> ChangeSetResult<ChangeSet<I>>;
}

#[test]
fn changeset_builder_compose() {
    use build_script_lang::schema::Schema;
    use build_script_shared::error::ParserResult;
    use build_script_shared::parsers::ParserDeserialize;
    use build_script_shared::parsers::ParserSerialize;
    use fake::Fake;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    for i in 0..500 {
        let schema0: Schema<String> = fake::Faker.fake();
        let schema1: Schema<String> = fake::Faker.fake();

        let value: ChangeSet<String> = schema0.build_changeset(&schema1).unwrap();

        let s: String = value.serialize_to_string().unwrap();
        let new_value: ParserResult<&str, ChangeSet<&str>> = ParserDeserialize::parse(s.as_str());
        let owned_value = new_value.map(|(s, v)| (s, v.map(ToString::to_string)));

        let mut hasher = DefaultHasher::new();
        if let Ok((_, ref v)) = owned_value {
            v.hash(&mut hasher);
        }
        let old_hash = hasher.finish();

        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let new_hash = hasher.finish();

        let new_changeset_hash = value.new_hash;
        let result = Ok(("", value));

        if &owned_value != &result {
            println!("{}", s);
            println!("");
            println!("After {}", i);
        }

        assert_eq!(owned_value, result);
        assert_eq!(old_hash, new_hash);
        assert_eq!(new_changeset_hash, schema1.get_hash());
    }
}
