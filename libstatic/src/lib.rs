// Copyright 2018 Cássio Kirch.

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::{io, time};

mod node;

#[macro_use]
mod macros {
    macro_rules! generate_cache {
        ($newname:ident, $map:tt) => {
            pub struct $newname {
                cache: Arc<RwLock<$map<PathBuf, Arc<RwLock<node::Node>>>>>,
            }
            impl $newname {
                pub fn get(&self, path: &PathBuf) -> Result<Vec<u8>, String> {
                    let (content, node) = {
                        let cache = self.cache.read().unwrap();
                        let node = cache.get(path);
                        match node {
                            Some(node) => {
                                let file = match File::open(path) {
                                    Ok(file) => file,
                                    Err(e) => {
                                        return Err(e.to_string());
                                    }
                                };

                                let must_update = {
                                    let node = node.read().unwrap();
                                    let must_update = node.must_update(&file);
                                    must_update.is_err() || must_update.unwrap()
                                };
                                return if must_update {
                                    let mut node = node.write().unwrap();
                                    node.update(&file);
                                    node.content.clone()
                                } else {
                                    let node = node.read().unwrap();
                                    node.content.clone()
                                };
                            }
                            None => {
                                let mut node = node::Node::default();
                                let file = File::open(path);
                                if file.is_err() {
                                    node.content = Err(file.unwrap_err().to_string());
                                } else {
                                    node.update(&file.unwrap());
                                }
                                (node.content.clone(), Arc::new(RwLock::new(node)))
                            }
                        }
                    };
                    let mut cache = self.cache.write().unwrap();
                    cache.insert(path.into(), node);
                    content
                }
            }

            impl Default for $newname {
                fn default() -> Self {
                    $newname {
                        cache: Arc::default(),
                    }
                }
            }
        };
    }
}

generate_cache!(HashCache, HashMap);
generate_cache!(BTreeCache, BTreeMap);
