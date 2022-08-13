/*
    Copyright (C) 2022  Biagio Festa

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::command_executor::CommandExecutor;
use crate::save_layout::KindNode;
use crate::save_layout::LayoutNode;
use crate::save_layout::SavedLayout;
use crate::utilities::find_node_by_id;
use crate::utilities::find_node_parent;
use crate::utilities::find_workspace_by_num;
use crate::utilities::set_node_layout;
use crate::utilities::set_node_split;
use crate::utilities::Layout;
use crate::utilities::Split;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::collections::HashMap;
use std::io::Read;
use std::time::Duration;

type NodeId = usize;

/// RestoreLayout executor.
///
/// Restore a previosly saved layout for a workspace.
pub struct RestoreLayout {
    command_executor: CommandExecutor,
}

impl RestoreLayout {
    const SLEEPTIME_BEFORE_RESIZE: Duration = Duration::from_millis(100);
    const SLEEPTIME_INTRA_RESIZE: Duration = Duration::from_micros(50);

    /// Construct the new executor.
    pub fn new(command_executor: CommandExecutor) -> Self {
        Self { command_executor }
    }

    /// It reads the saved workspace from `input`.
    ///
    /// Then it tries to restore the layout saved with a best-effort approach.
    pub fn execute<R>(mut self, input: R, json_input: bool) -> Result<()>
    where
        R: Read,
    {
        let saved_layout = SavedLayout::deserialize(input, json_input)?;

        let workspace_num = match saved_layout.root().kind() {
            KindNode::Workspace(workspace_num) => Ok(*workspace_num),
            _ => Err(anyhow!("Invalid layout. Workspace is missing")),
        }?;

        let mut created_paths = HashMap::new();
        let mut dfs = vec![(saved_layout.root(), Vec::<(NodeId, LayoutNode)>::new())];

        while let Some((saved_node, mut path)) = dfs.pop() {
            if saved_node.children().is_empty() {
                let node_exists = self
                    .move_node_on_ws_if_exists(saved_node.id(), workspace_num)
                    .with_context(|| format!("Cannot move node '{}'", saved_node.id()))?;

                if node_exists {
                    self.create_path_tree_for_node(saved_node.id(), &path, &mut created_paths)?;
                } else {
                    println!(
                        "[WARN]: Cannot restore node '{}' (not found)",
                        saved_node.id()
                    );
                }
            } else {
                path.push((saved_node.id(), saved_node.layout()));

                dfs.extend(
                    saved_node
                        .children()
                        .iter()
                        .rev()
                        .map(|&child_id| (saved_layout.lookup_by_id(child_id), path.clone())),
                )
            }
        }

        std::thread::sleep(Self::SLEEPTIME_BEFORE_RESIZE);

        self.restore_sizes(&saved_layout)
            .context("Cannot restore sizes of layout")?;

        Ok(())
    }

    fn move_node_on_ws_if_exists(&mut self, node_id: usize, workspace_num: i32) -> Result<bool> {
        const MARK_ID: &str = "MARK_TMP_RESTORE";

        let root_node = self.command_executor.query_root_node()?;

        if find_node_by_id(node_id, &root_node).is_some() {
            self.command_executor
                .run_on_node_id(node_id, format!("move to workspace {}", workspace_num))?;

            let workspace = find_workspace_by_num(&root_node, workspace_num)
                .expect("Expected WS to exist after move on it");
            self.command_executor
                .run_on_node_id(workspace.id, format!("mark {}", MARK_ID))?;

            let _ = self
                .command_executor
                .run_on_node_id(node_id, format!("move to mark {}", MARK_ID));

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn create_path_tree_for_node(
        &mut self,
        node_id: usize,
        path: &[(NodeId, LayoutNode)],
        created_paths: &mut HashMap<NodeId, NodeId>,
    ) -> Result<()> {
        const MARK_ID: &str = "MARK_MOVE";

        let mut last_id = node_id;

        for (split_id, split_layout) in path.iter().skip(1).rev() {
            match created_paths.get(split_id) {
                Some(&new_splitter_id) => {
                    self.command_executor
                        .run_on_node_id(new_splitter_id, format!("mark {}", MARK_ID))?;

                    self.command_executor
                        .run_on_node_id(last_id, format!("move to mark {}", MARK_ID))?;

                    return Ok(());
                }
                None => {
                    let layout = match split_layout {
                        LayoutNode::SplitH => Layout::SplitH,
                        LayoutNode::SplitV => Layout::SplitV,
                        LayoutNode::Stacked => Layout::Stacked,
                        LayoutNode::Tabbed => Layout::Tabbed,
                    };

                    set_node_split(last_id, Split::Horizontal, &mut self.command_executor)?;
                    set_node_layout(last_id, layout, &mut self.command_executor)?;

                    let root_node = self.command_executor.query_root_node()?;
                    last_id = find_node_parent(last_id, &root_node)
                        .expect("Expected parent just created")
                        .id;

                    created_paths.insert(*split_id, last_id);
                }
            }
        }

        Ok(())
    }

    fn restore_sizes(&mut self, saved_layout: &SavedLayout) -> Result<()> {
        let mut dfs = vec![saved_layout.root()];

        while let Some(saved_node) = dfs.pop() {
            if let KindNode::NormalWindow(saved_window) = saved_node.kind() {
                let root_node = self.command_executor.query_root_node()?;

                let saved_width = saved_window.width();
                let saved_height = saved_window.height();

                if let Some(node) = find_node_by_id(saved_node.id(), &root_node) {
                    if node.window_rect.width != saved_width {
                        let _ = self.command_executor.run_on_node_id(
                            node.id,
                            format!("resize set width {} px", saved_width),
                        );
                    }

                    if node.window_rect.height != saved_height {
                        let _ = self.command_executor.run_on_node_id(
                            node.id,
                            format!("resize set height {} px", saved_height),
                        );
                    }
                }

                std::thread::sleep(Self::SLEEPTIME_INTRA_RESIZE);
            } else {
                dfs.extend(
                    saved_node
                        .children()
                        .iter()
                        .map(|&child_id| saved_layout.lookup_by_id(child_id)),
                )
            }
        }

        Ok(())
    }
}
