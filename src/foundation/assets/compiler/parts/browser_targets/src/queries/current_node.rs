#[cfg(any(not(target_arch = "wasm32"), any()))]
use super::Distrib;
use super::QueryResult;
use crate::browser_targets::error::Error;

pub(super) fn current_node() -> QueryResult {
    #[cfg(target_arch = "wasm32")]
    {
        #[cfg(any())]
        {
            use js_sys::{Reflect, global};

            let obj_process = Reflect::get(&global(), &"process".into())
                .map_err(|_| Error::UnsupportedCurrentNode)?;
            let obj_versions = Reflect::get(&obj_process, &"versions".into())
                .map_err(|_| Error::UnsupportedCurrentNode)?;
            let version = Reflect::get(&obj_versions, &"node".into())
                .map_err(|_| Error::UnsupportedCurrentNode)?
                .as_string()
                .ok_or(Error::UnsupportedCurrentNode)?;
            return Ok(vec![Distrib::new("node", version)]);
        }

        #[cfg(not(any()))]
        Err(Error::UnsupportedCurrentNode)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::{process::Command, sync::OnceLock};

        static CURRENT_NODE: OnceLock<QueryResult> = OnceLock::new();

        CURRENT_NODE
            .get_or_init(|| {
                let output = Command::new("node")
                    .arg("-v")
                    .output()
                    .map_err(|_| Error::UnsupportedCurrentNode)?;
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .trim_start_matches('v')
                    .to_owned();

                Ok(vec![Distrib::new("node", version)])
            })
            .clone()
    }
}
