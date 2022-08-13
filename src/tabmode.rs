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
use crate::command_executor::I3Node;
use crate::restore_layout::RestoreLayout;
use crate::save_layout::SaveLayout;
use crate::utilities::find_workspace_by_num;
use crate::utilities::query_workspace_focused;
use crate::utilities::set_node_layout;
use crate::utilities::Layout;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use i3_ipc::reply::NodeLayout;
use i3_ipc::reply::NodeType;
use std::fs::File;
use std::path::Path;

/// TabMode executor.
///
/// It represents a one-shot executor which normalizes the current active workspace
/// and display all nodes in tabbed mode.
///
/// One tab per window.
pub struct TabMode {
    /// Command executor.
    command_executor: CommandExecutor,
}

impl TabMode {
    /// A temporary mark for moving nodes.
    const MARK_ID: &'static str = "__i3-autolayout__tmp_ID";

    /// A new tabmode executor.
    pub fn new(command_executor: CommandExecutor) -> Self {
        Self { command_executor }
    }

    /// Execute the action.
    ///
    /// It normalizes a workspace and displays all nodes it a tabbed layout.
    /// It can be toggled: if the workspace is already in tab-mode it will restore the default layout.
    ///
    /// The action will be appliced on a specific workspace number (argument).
    /// If `workspace_num` is `None` the currently focused workspace will be used.
    pub fn execute(mut self, workspace_num: Option<i32>, file_layout: Option<&Path>) -> Result<()> {
        let root_node = self.command_executor.query_root_node()?;

        let workspace = match workspace_num {
            Some(workspace_num) => find_workspace_by_num(&root_node, workspace_num)
                .ok_or_else(|| anyhow!("Cannot find the workspace number '{}'", workspace_num))?,
            None => query_workspace_focused(&root_node, &mut self.command_executor)?,
        };
        let workspace_num = workspace.num.expect("Expected workspace have number");

        if Self::is_tabmode(workspace) {
            if let Some(file_layout) = file_layout {
                let file = File::open(file_layout).with_context(|| {
                    format!("Cannot open the layout file '{}'", file_layout.display())
                })?;

                let restore_layout = RestoreLayout::new(self.command_executor);

                restore_layout
                    .execute(file, false, true)
                    .context("Cannot restore layout")
            } else {
                self.normalize_workspace(workspace)
                    .context("Cannot normalize the workspace for tabmode")?;

                set_node_layout(workspace.id, Layout::Default, &mut self.command_executor)
                    .context("Cannot set default layout for workspace")
            }
        } else {
            if let Some(file_layout) = file_layout {
                let file = File::create(file_layout).with_context(|| {
                    format!("Cannot save the layout on file '{}'", file_layout.display())
                })?;

                let save_layout = SaveLayout::new(
                    CommandExecutor::new()
                        .context("Cannot create a new executor for saving layout")?,
                );

                save_layout
                    .execute(Some(workspace_num), file, false)
                    .context("Cannot save the layout")?;
            }

            self.normalize_workspace(workspace)
                .context("Cannot normalize the workspace for tabmode")?;

            set_node_layout(workspace.id, Layout::Tabbed, &mut self.command_executor)
                .context("Cannot set tab layout for workspace")
        }
    }

    /// Whether the workspace is already in tabmode or not.
    fn is_tabmode(workspace: &I3Node) -> bool {
        if workspace.nodes.len() == 1 {
            let child = unsafe { workspace.nodes.get_unchecked(0) };

            child.window_type.is_none() && matches!(child.layout, NodeLayout::Tabbed)
        } else {
            false
        }
    }

    /// Normalize a workspace.
    ///
    /// Move all leaf nodes as workspace children.
    fn normalize_workspace(&mut self, workspace: &I3Node) -> Result<()> {
        debug_assert!(matches!(workspace.node_type, NodeType::Workspace));

        self.command_executor
            .run_on_node_id(workspace.id, format!("mark \"{}\"", Self::MARK_ID))
            .context("Cannot set temporary mark on focused workspace")?;

        let mut dfs = workspace
            .nodes
            .iter()
            .map(|node| (node, workspace.id))
            .collect::<Vec<_>>();

        while let Some((current, parent)) = dfs.pop() {
            if current.nodes.is_empty() && parent != workspace.id {
                self.command_executor
                    .run_on_node_id(
                        current.id,
                        format!("move window to mark \"{}\"", Self::MARK_ID),
                    )
                    .context("Cannot mode window on mark")?;
            } else {
                dfs.extend(current.nodes.iter().map(|node| (node, current.id)));
            }
        }

        self.command_executor
            .run(format!("unmark \"{}\"", Self::MARK_ID))
            .context("Cannot unset temporary mark")
    }
}
