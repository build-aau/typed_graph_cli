use std::collections::{HashMap, HashSet};
use std::fs::{read_dir, read_to_string, create_dir_all, remove_file};
use std::path::{Path, PathBuf};
use build_changeset_lang::{ChangeSet, DefaultChangeset, ChangeSetBuilder};
use build_script_lang::DefaultSchema;
use build_script_lang::schema::Schema;
use build_script_shared::InputMarker;
use build_script_shared::parsers::{Ident, Mark, ParserDeserialize, ParserSerialize};

use crate::{GenResult, GenError};

#[derive(Default)]
pub struct Project {
    schemas: HashMap<String, Schema<String>>,
    changesets: HashMap<u64, ChangeSet<String>>,
    version_tree: HashMap<String, HashMap<String, (u64, Direction)>>,
    schema_folder: PathBuf,
    changeset_folder: PathBuf
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum Direction {
    Forward, 
    Backwards
}

impl Project {
    pub fn create_project<P: AsRef<Path>>(p: P) -> GenResult<Project> {
        let (schema_folder, changeset_folder) = Project::create_project_directory(p)?;
        Project::open(schema_folder, changeset_folder, true)
    }

    pub fn open_project<P: AsRef<Path>>(p: P) -> GenResult<Project> {
        let (schema_folder, changeset_folder) = Project::open_project_directory(p)?;
        Project::open(schema_folder, changeset_folder, true)
    }

    pub fn open_project_raw<P: AsRef<Path>>(p: P) -> GenResult<Project> {
        let (schema_folder, changeset_folder) = Project::open_project_directory(p)?;
        Project::open(schema_folder, changeset_folder, false)
    }

    fn open(schema_folder: PathBuf, changeset_folder: PathBuf, check_integrity: bool) -> GenResult<Project> {
        let mut project = Project::default();

        project.schema_folder = schema_folder.clone();
        project.changeset_folder = changeset_folder.clone();

        project.load_schemas(schema_folder)?;
        project.load_changesets(changeset_folder)?;

        if check_integrity {
            project.verify_integrity()?;
        }

        Ok(project)
    }

    pub fn update_changeset_hash(&mut self, id: &u64) -> GenResult<()> {
        let changeset = self.changesets.get(id).ok_or_else(|| GenError::UnknownChangeset { name: id.clone() })?;
        let new_schema = self.get_schema(&changeset.new_version)?.get_hash();
        let old_schema = self.get_schema(&changeset.old_version)?.get_hash();
        
        let changeset = self.changesets.get_mut(id).ok_or_else(|| GenError::UnknownChangeset { name: id.clone() })?;
        changeset.old_hash = old_schema;
        changeset.new_hash = new_schema;

        self.save_changeset(id)?;
        Ok(())
    }

    pub fn get_schema_folder(&self) -> &Path {
        &self.schema_folder
    }

    pub fn get_changeset_folder(&self) -> &Path {
        &self.changeset_folder
    }

    pub fn has_schema(&self, id: &str) -> bool {
        self.schemas.contains_key(id)
    }

    pub fn get_schema(&self, id: &String) -> GenResult<&Schema<String>> {
        self.get_schema_safe(id).ok_or_else(|| GenError::UnknownSchema { name: id.clone() })
    }

    pub fn get_changeset(&self, id: &u64) -> GenResult<&ChangeSet<String>> {
        self.get_changeset_safe(id).ok_or_else(|| GenError::UnknownChangeset { name: id.clone() })
    }

    pub fn get_schema_safe(&self, id: &String) -> Option<&Schema<String>> {
        self.schemas.get(id)
    }

    pub fn get_changeset_safe(&self, id: &u64) -> Option<&ChangeSet<String>> {
        self.changesets.get(id)
    }

    pub fn iter_schema(&self) -> impl Iterator<Item = &String> {
        self.schemas
            .keys()
    }

    pub fn iter_changesets(&self) -> impl Iterator<Item = &u64> {
        self.changesets
            .keys()
    }

    pub fn iter_version(&self, dir: Option<Direction>) -> impl Iterator<Item = (&String, &String, &u64)> {
        self.version_tree
            .iter()
            .map(move |(old, tree)| {
                tree
                    .iter()
                    .filter(move |(_, (_, tree_dir))| dir.map_or_else(|| false, |dir| dir == *tree_dir))
                    .map(move |(new, (id, _))| (old, new, id))
            })
            .flatten()
    }

    pub fn copy_schema(&mut self, id: &String, increment_name: bool) -> GenResult<String> {
        let old_schema = self.get_schema(id)?;
        let mut new_schema = old_schema.clone();

        let name = new_schema.version.to_string();
        let mut new_name = if increment_name {
            Project::increment_name(&name, || "_copy".chars())
        } else {
            name
        };

        while self.has_schema(&new_name) {
            new_name += "_copy";
        }

        new_schema.version = Ident::new(new_name, Mark::default());
        let new_version = new_schema.version.to_string();
        self.add_schema(new_schema.clone())?;
        self.save_schema(&new_version)?;

        Ok(new_version)
    }

    fn increment_name<F, FR>(name: &String, default: F) -> String 
    where
        F: Fn() -> FR,
        FR: Iterator<Item = char>
    {
        let mut chars: Vec<char> = name.chars().collect();
        let mut num = Vec::new();

        while let Some(n) = chars.pop() {
            if n.is_numeric() {
                num.push(n);
            } else {
                chars.push(n);
                break;
            }
        }

        let mut requires_new = true;

        if !num.is_empty() {
            let mut overflow = true;
            for c in &mut num {
                if !overflow {
                    break;
                }
                
                let mut v: u8 = *c as u8;

                if overflow {
                    v += 1;
                }

                if v == '9' as u8 + 1 {
                    overflow = true;
                    v = '0' as u8;
                } else {
                    overflow = false;
                }

                *c = v as char;
            }
            if overflow {
                num.push('1');
            }

            num.reverse();
            for c in num {
                chars.push(c);
            }

            requires_new = false
        }

        if requires_new {
            chars.extend(default());
        }

        chars.into_iter().collect()
    }

    pub fn create_changeset(&mut self, old: &String, new: &String) -> GenResult<u64> {
        let old_schema = self.get_schema(&old)?;
        let new_schema = self.get_schema(&new)?;

        let changeset = self.version_tree
            .get(old)
            .and_then(|inner| inner.get(new));

        if let Some((id, _)) = changeset {
            let changeset = self.get_changeset(id)?;
            return Err(GenError::DuplicateKeys { 
                kind: "changeset".to_string(), 
                old: changeset.old_version.to_string(), 
                new: changeset.new_version.to_string(), 
                old_hash: changeset.old_hash, 
                new_hash: changeset.new_hash 
            });
        }

        let changeset = old_schema.build_changeset(new_schema)?;        

        println!("Found changes:");
        println!("{}", changeset);

        let new = self.add_changeset(changeset)?;

        self.save_changeset(&new)?;
        
        Ok(new)
    }

    pub fn save_schema(&self, schema: &String) -> GenResult<PathBuf> {
        let schema = self.get_schema(schema)?;
        let p = self.schema_folder.join(format!("{}.bs", schema.version));
        schema.serialize_to_file(&p)?;
        Ok(p)
    }

    pub fn save_changeset(&self, changset: &u64) -> GenResult<PathBuf> {
        let changeset = self.get_changeset(changset)?;
        self.test_changeset(changeset)?;
        let p = self.changeset_folder.join(format!("{} {}.bs.diff", changeset.old_version, changeset.new_version));
        changeset.serialize_to_file(&p)?;
        Ok(p)
    }

    pub fn add_schema(&mut self, schema: Schema<String>) -> GenResult<String> {
        let new_version = schema.version.to_string();
        let new_hash = schema.get_hash();
        let duplicate_key = self.schemas.insert(new_version.clone(), schema);

        if let Some(old_schema) = duplicate_key {
            return Err(GenError::DuplicateKeys {
                kind: "schema".to_string(), 
                old: old_schema.version.to_string(), 
                new: new_version,
                old_hash: old_schema.get_hash(), 
                new_hash: new_hash,
            });
        }

        Ok(new_version)
    }

    pub fn rename_schema(&mut self, old_schema: &String, new_name: String) -> GenResult<()> {
        // Check the input
        if !self.has_schema(old_schema) {
            return Err(GenError::UnknownSchema { name: old_schema.clone() });
        }

        let schema_ref = self.get_schema(old_schema)?;
        if let Ok(new_schema) = self.get_schema(&new_name) {
            return Err(GenError::DuplicateKeys {
                kind: "schema".to_string(), 
                old: schema_ref.version.to_string(), 
                new: new_schema.version.to_string(),
                old_hash: schema_ref.get_hash(), 
                new_hash: new_schema.get_hash(),
            });
        }

        // Update the schema
        let mut schema = self.schemas.remove(old_schema).unwrap();
        schema.version = Ident::new_alone(new_name.clone());
        let new_schema_hash = schema.get_hash();
        
        self.schemas.insert(new_name.clone(), schema);

        let mut changesets_to_update = Vec::new();
        if let Some(changesets) = self.version_tree.get(old_schema) {
            for changeset in changesets.values() {
                changesets_to_update.push(changeset.clone());
            }
        }

        // Update the changeset
        let mut updated_changesets = Vec::new();
        for (old_changeset_id, dir) in &changesets_to_update {
            // Update the changeset
            let mut changeset = self.changesets.remove(old_changeset_id).unwrap();
            
            let old_source = changeset.old_version.to_string();
            let old_target = changeset.new_version.to_string();

            let new_changeset_id = changeset.get_hash();

            match dir {
                Direction::Forward => {
                    changeset.old_version = Ident::new_alone(new_name.clone());
                    changeset.old_hash = new_schema_hash;
                },
                Direction::Backwards => {
                    changeset.new_version = Ident::new_alone(new_name.clone());
                    changeset.new_hash = new_schema_hash;
                }
            }

            let new_source = changeset.old_version.to_string();
            let new_target = changeset.new_version.to_string();

            self.changesets.insert(new_changeset_id, changeset);

            let p = self.changeset_folder.join(format!("{} {}.bs.diff", old_source, old_target));
            updated_changesets.push((new_changeset_id, p));

            // Remove the old changeset in the version tree
            let forward = self.version_tree.entry(old_source.clone()).or_default();
            forward.remove(&old_target);

            if forward.is_empty() {
                self.version_tree.remove(&old_source);
            }

            let backwards = self.version_tree.entry(old_target.clone()).or_default();
            backwards.remove(&old_source);

            if backwards.is_empty() {
                self.version_tree.remove(&old_target);
            }

            // Add the new changest in the version tree
            self.version_tree.entry(new_source.clone()).or_default().insert(new_target.clone(), (new_changeset_id, Direction::Forward));
            self.version_tree.entry(new_target.clone()).or_default().insert(new_source.clone(), (new_changeset_id, Direction::Backwards));

        }

        // Update changeset files
        for (changeset_id, p) in updated_changesets {
            remove_file(p)?;
            self.save_changeset(&changeset_id)?;
        }

        // Update schema file
        let p = self.schema_folder.join(format!("{}.bs", old_schema));
        remove_file(p)?;
        self.save_schema(&new_name)?;
        
        Ok(())
    }

    fn add_changeset(&mut self, changeset: ChangeSet<String>) -> GenResult<u64> {
        let id = changeset.get_hash();
                    
        self.version_tree
            .entry(changeset.old_version.to_string())
            .or_default()
            .insert(changeset.new_version.to_string(), (id, Direction::Forward));
        self.version_tree
            .entry(changeset.new_version.to_string())
            .or_default()
            .insert(changeset.old_version.to_string(), (id, Direction::Backwards));
        
        let duplicate_key: Option<ChangeSet<String>> = self.changesets.insert(id, changeset);
        if let Some(changeset) = duplicate_key {
            return Err(GenError::DuplicateKeys {
                kind: "changset".to_string(), 
                old: changeset.old_version.to_string(), 
                new: changeset.new_version.to_string(),
                old_hash: changeset.old_hash, 
                new_hash: changeset.new_hash,
            });
        }

        Ok(id)
    }
    
    pub fn find_heads(&self) -> Vec<String> {
        let mut schema_keys: HashSet<String> = self.schemas.keys().cloned().collect();
        for (old, _, _) in self.iter_version(Some(Direction::Forward)) {
            schema_keys.remove(old);
        }
        schema_keys
            .into_iter()
            .collect()
    }

    pub fn find_roots(&self) -> Vec<String> {
        let mut schema_keys: HashSet<String> = self.schemas.keys().cloned().collect();
        for (old, _, _) in self.iter_version(Some(Direction::Backwards)) {
            schema_keys.remove(old);
        }
        schema_keys
            .into_iter()
            .collect()
    }

    fn open_project_directory<P: AsRef<Path>>(p: P) -> GenResult<(PathBuf, PathBuf)> {
        let root = p.as_ref();
        if !root.exists() {
            return Err(GenError::InvalidProjectPath(root.to_path_buf()));
        }

        let schema_folder = root.join("schemas");
        let changeset_folder = root.join("changesets");

        if !schema_folder.exists() || !schema_folder.is_dir() {
            return Err(GenError::MissingFolder {
                folder: schema_folder
                    .to_str()
                    .ok_or_else(|| GenError::MalformedPath)?
                    .to_string()
            });
        }

        if !changeset_folder.exists() || !changeset_folder.is_dir() {
            return Err(GenError::MissingFolder {
                folder: changeset_folder
                    .to_str()
                    .ok_or_else(|| GenError::MalformedPath)?
                    .to_string()
            });
        }

        Ok((schema_folder, changeset_folder))
    }

    fn create_project_directory<P: AsRef<Path>>(p: P) -> GenResult<(PathBuf, PathBuf)> {
        let root = p.as_ref();
        if !root.exists() {
            return Err(GenError::InvalidProjectPath(root.to_path_buf()));
        }

        let schema_folder = root.join("schemas");
        let changeset_folder = root.join("changesets");

        if !schema_folder.exists() {
            create_dir_all(&schema_folder)?;
        }

        if !changeset_folder.exists() {
            create_dir_all(&changeset_folder)?;
        }

        Ok((schema_folder, changeset_folder))
    }

    fn load_schemas<P: AsRef<Path>>(&mut self, schema_folder: P) -> GenResult<()> {
        let schema_iter = read_dir(schema_folder)?;
        for schema_file in schema_iter {
            let schema_path = schema_file?;
            if let Ok(s) =  schema_path.file_name().into_string() {
                if s.ends_with(".bs") {
                    let content = read_to_string(schema_path.path())?;
                    let input = InputMarker::new_from_file(
                        content.as_str(), 
                        schema_path
                            .path()
                            .to_str()
                            .ok_or_else(|| GenError::MalformedPath)?
                            .to_string()
                    );
                    let schema = DefaultSchema::deserialize(input)?;
                    let owned_schema = schema.map(|i| i.to_string());
                    self.add_schema(owned_schema)?;
                }

            }
        }

        Ok(())
    }

    fn load_changesets<P: AsRef<Path>>(&mut self, changeset_folder: P) -> GenResult<()> {
        let changeset_iter = read_dir(changeset_folder)?;
        for changeset_file in changeset_iter {
            let changeset_path = changeset_file?;
            if let Ok(s) =  changeset_path.file_name().into_string() {
                if s.ends_with(".bs.diff") {

                    let content = read_to_string(changeset_path.path())?;
                    let input = InputMarker::new_from_file(
                        content.as_str(), 
                        changeset_path
                            .path()
                            .to_str()
                            .ok_or_else(|| GenError::MalformedPath)?
                            .to_string()
                    );
                    let changeset = DefaultChangeset::deserialize(input)?;
                    let owned_changeset = changeset.map(|i| i.to_string());
                    self.add_changeset(owned_changeset)?;
                }
            }
        }
        Ok(())
    }

    pub fn build_missing_schemas(&mut self) -> GenResult<()> {
        // First figure out which we have changesets for but no actual implementation
        let mut missing_schema = Vec::new();
        for (_, new, _) in self.iter_version(Some(Direction::Forward)) {
            if !self.schemas.contains_key(new) {
                missing_schema.push(new.to_owned());
            }
        }

        // Next we make a list of all the changesets that should be used to create the missing schemas
        let mut process_queue = Vec::new();
        let mut processed: HashSet<String> = self.schemas.keys().cloned().collect();
        while let Some(missing) = missing_schema.pop() {
            if processed.contains(&missing) {
                continue;
            }
            // Find a parent schema to the current one
            let all_paths = &self.version_tree[&missing];
            let changeset_id = all_paths
                .iter()
                .filter(|(_, (_, dir))| dir == &Direction::Backwards)
                .map(|(_, (id, _))| id)
                .next()
                .ok_or_else(|| GenError::UnreachableSchema { target: missing.clone() })?;
    
            // add the changeset to the  workload
            let changeset = &self.changesets[changeset_id];
            assert_eq!(changeset.new_version.to_string(), missing);
            processed.insert(missing);
            process_queue.push(*changeset_id);

            // Make sure the parent is created before the current one
            missing_schema.push(changeset.old_version.to_string());
        }

        // Finally we apply the changesets to generate the missing schemas
        while let Some(changeset_id) = process_queue.pop() {
            let changeset = &self.changesets[&changeset_id];
            if self.schemas.contains_key(&changeset.new_version.to_string()) {
                continue;
            }

            let old_schema = &self.schemas[&changeset.old_version.to_string()];
            let new_schema = changeset.apply(old_schema.clone())?;
            let new = self.add_schema(new_schema)?;
            self.save_schema(&new)?;
        }

        Ok(())
    }

    fn verify_integrity(&self) -> GenResult<()> {
        self.check_version_tree()?;
        self.check_changset_hashes()?;
        Ok(())
    }

    fn check_changset_hashes(&self) -> GenResult<()> {
        for changeset in self.changesets.values() {
            self.test_changeset(changeset)?;
        }

        Ok(())
    }

    fn test_changeset(&self, changeset: &ChangeSet<String>) -> GenResult<()> {
        let old = self.get_schema(&changeset.old_version)?;
        let new = self.get_schema(&changeset.new_version)?;

        let old_hash = old.get_hash();
        let new_hash = new.get_hash();

        if old_hash != changeset.old_hash {
            return Err(GenError::DivergentChangeset { 
                old_version: changeset.old_version.to_string(),
                new_version: changeset.new_version.to_string(),
                schema: changeset.old_version.to_string(),
                expected: changeset.old_hash,
                recieved: old_hash 
            });
        }

        if new_hash != changeset.new_hash {
            return Err(GenError::DivergentChangeset { 
                old_version: changeset.old_version.to_string(),
                new_version: changeset.new_version.to_string(),
                schema: changeset.new_version.to_string(),
                expected: changeset.new_hash,
                recieved: new_hash 
            });
        }

        changeset.apply(old.clone())?;

        Ok(())
    }

    fn check_version_tree(&self) -> GenResult<()> {
        for (old, new_tree) in &self.version_tree {
            for (new, (change_id, _)) in new_tree {
                if !self.changesets.contains_key(change_id) {
                    return Err(GenError::MalformedVersionTree { 
                        kind: "changset".to_string(),
                        missing_key: format!("{:#16x}", change_id)
                    });
                }

                if !self.schemas.contains_key(new) {
                    return Err(GenError::MalformedVersionTree { 
                        kind: "new schema".to_string(),
                        missing_key: new.clone() 
                    });
                }

                if !self.schemas.contains_key(old) {
                    return Err(GenError::MalformedVersionTree { 
                        kind: "old schema".to_string(),
                        missing_key: old.clone()
                    });
                }                
            }
        }
        Ok(())
    }
}

#[test]
fn simple_project() -> GenResult<()> {
    let prj = Project::open_project("test/simple_project")?;
    
    assert_eq!(prj.schemas.len(), 1);

    Ok(())
}