use std::{fs, path::Path};

use anyhow::Result;

use crate::{constants, models::InitPreviewItem};

pub struct InitService;

impl InitService {
    pub fn get_preview(&self) -> Vec<InitPreviewItem> {
        constants::init_preview()
    }

    pub fn init_system(&self, inbox: &Path, archive_root: &Path) -> Result<()> {
        fs::create_dir_all(inbox)?;
        fs::create_dir_all(inbox.join("_Failed"))?;
        fs::create_dir_all(inbox.join("_Review"))?;

        for node in constants::init_preview() {
            self.create_node(archive_root, &node)?;
        }

        Ok(())
    }

    fn create_node(&self, parent: &Path, node: &InitPreviewItem) -> Result<()> {
        let current = parent.join(format!("{}_{}", node.code, node.folder));
        fs::create_dir_all(&current)?;

        if let Some(children) = &node.children {
            for child in children {
                self.create_node(&current, child)?;
            }
        }

        Ok(())
    }
}
