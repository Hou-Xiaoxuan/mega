use std::cell::RefCell;
use std::collections::HashMap;

use callisto::{mega_blob, mega_refs, mega_tree, raw_blob};
use common::utils::generate_id;
use mercury::hash::SHA1;
use mercury::internal::object::blob::Blob;
use mercury::internal::object::commit::Commit;
use mercury::internal::object::tree::{Tree, TreeItem, TreeItemMode};

pub fn generate_git_keep() -> Blob {
    let git_keep_content = String::from("This file was used to maintain the git tree");
    Blob::from_content(&git_keep_content)
}

pub fn generate_git_keep_with_timestamp() -> Blob {
    let git_keep_content = format!(
        "This file was used to maintain the git tree, generate at:{}",
        chrono::Utc::now().naive_utc()
    );
    Blob::from_content(&git_keep_content)
}

pub fn init_trees(git_keep: &Blob) -> (HashMap<SHA1, Tree>, Tree) {
    let tree_item = TreeItem {
        mode: TreeItemMode::Blob,
        id: git_keep.id,
        name: String::from(".gitkeep"),
    };
    let tree = Tree::from_tree_items(vec![tree_item.clone()]).unwrap();

    let root_items = vec![
        TreeItem {
            mode: TreeItemMode::Tree,
            id: tree.id,
            name: String::from("third-part"),
        },
        TreeItem {
            mode: TreeItemMode::Tree,
            id: tree.id,
            name: String::from("project"),
        },
        TreeItem {
            mode: TreeItemMode::Tree,
            id: tree.id,
            name: String::from("doc"),
        },
        TreeItem {
            mode: TreeItemMode::Tree,
            id: tree.id,
            name: String::from("release"),
        },
    ];

    let root = Tree::from_tree_items(root_items).unwrap();
    let trees = vec![tree];
    (trees.into_iter().map(|x| (x.id, x)).collect(), root)
}

pub struct MegaModelConverter {
    pub commit: Commit,
    pub root_tree: Tree,
    pub tree_maps: HashMap<SHA1, Tree>,
    pub blob_maps: HashMap<SHA1, Blob>,
    pub mega_trees: RefCell<HashMap<SHA1, mega_tree::ActiveModel>>,
    pub mega_blobs: RefCell<HashMap<SHA1, mega_blob::ActiveModel>>,
    pub raw_blobs: RefCell<HashMap<SHA1, raw_blob::ActiveModel>>,
    pub refs: mega_refs::ActiveModel,
}

impl MegaModelConverter {
    fn traverse_from_root(&self) {
        let root_tree = &self.root_tree;
        let mut mega_tree: mega_tree::Model = root_tree.to_owned().into();
        mega_tree.commit_id = self.commit.id.to_plain_str();
        self.mega_trees
            .borrow_mut()
            .insert(root_tree.id, mega_tree.clone().into());
        self.traverse_for_update(&self.root_tree);
    }

    fn traverse_for_update(&self, tree: &Tree) {
        for item in &tree.tree_items {
            if item.mode == TreeItemMode::Tree {
                let child_tree = self.tree_maps.get(&item.id).unwrap();
                let mut mega_tree: mega_tree::Model = child_tree.to_owned().into();
                mega_tree.commit_id = self.commit.id.to_plain_str();
                self.mega_trees
                    .borrow_mut()
                    .insert(child_tree.id, mega_tree.clone().into());
                self.traverse_for_update(child_tree);
            } else {
                let blob = self.blob_maps.get(&item.id).unwrap();
                let mut mega_blob: mega_blob::Model = blob.into();
                mega_blob.commit_id = self.commit.id.to_plain_str();
                self.mega_blobs
                    .borrow_mut()
                    .insert(blob.id, mega_blob.clone().into());
                let raw_blob: raw_blob::Model = blob.to_owned().into();
                self.raw_blobs.borrow_mut().insert(blob.id, raw_blob.into());
            }
        }
    }

    pub fn init() -> Self {
        let git_keep = generate_git_keep();
        let (tree_maps, root_tree) = init_trees(&git_keep);
        let commit = Commit::from_tree_id(root_tree.id, vec![], "Init Mega Directory");
        let mut blob_maps = HashMap::new();
        blob_maps.insert(git_keep.id, git_keep);

        let mega_ref = mega_refs::Model {
            id: generate_id(),
            path: "/".to_owned(),
            ref_commit_hash: commit.id.to_plain_str(),
            ref_tree_hash: commit.tree_id.to_plain_str(),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        };

        let converter = MegaModelConverter {
            commit,
            root_tree,
            tree_maps,
            blob_maps,
            mega_trees: RefCell::new(HashMap::new()),
            mega_blobs: RefCell::new(HashMap::new()),
            raw_blobs: RefCell::new(HashMap::new()),
            refs: mega_ref.into(),
        };
        converter.traverse_from_root();
        converter
    }
}

#[cfg(test)]
mod test {

    use std::str::FromStr;

    use mercury::{hash::SHA1, internal::object::commit::Commit};

    use crate::utils::converter::MegaModelConverter;

    #[test]
    pub fn test_init_mega_dir() {
        let converter = MegaModelConverter::init();
        let mega_trees = converter.mega_trees.borrow().clone();
        let mega_blobs = converter.mega_blobs.borrow().clone();
        let raw_blob = converter.raw_blobs.borrow().clone();
        assert_eq!(mega_trees.len(), 2);
        assert_eq!(mega_blobs.len(), 1);
        assert_eq!(raw_blob.len(), 1);
    }

    #[test]
    pub fn test_init_commit() {
        let commit = Commit::from_tree_id(
            SHA1::from_str("bd4a28f2d8b2efc371f557c3b80d320466ed83f3").unwrap(),
            vec![],
            "Init Mega Directory",
        );
        println!("{}", commit);
    }
}
