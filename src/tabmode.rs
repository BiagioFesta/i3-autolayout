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
use crate::utilities::find_workspace_by_num;
use crate::utilities::query_workspace_focused;
use crate::utilities::set_node_layout;
use crate::utilities::Layout;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use i3_ipc::reply::NodeLayout;
use i3_ipc::reply::NodeType;

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
    pub fn execute(mut self, workspace_num: Option<i32>) -> Result<()> {
        let root_node = self.command_executor.query_root_node()?;

        let workspace = match workspace_num {
            Some(workspace_num) => find_workspace_by_num(&root_node, workspace_num)
                .ok_or_else(|| anyhow!("Cannot find the workspace number '{}'", workspace_num))?,
            None => query_workspace_focused(&root_node, &mut self.command_executor)?,
        };

        self.normalize_workspace(workspace)?;

        let layout = match workspace.nodes.first().map(|n| n.layout) {
            Some(NodeLayout::Tabbed) => Layout::Default,
            _ => Layout::Tabbed,
        };

        set_node_layout(workspace.id, layout, &mut self.command_executor)
            .context("Cannot layout for focused workspace")
    }

    /// Normalize a workspace.
    ///
    /// Recursively, all nodes in that workspace will be placed as direct children of
    /// the workspace node itself.
    fn normalize_workspace(&mut self, workspace: &I3Node) -> Result<()> {
        debug_assert!(matches!(workspace.node_type, NodeType::Workspace));

        self.command_executor
            .run_on_node_id(workspace.id, format!("mark \"{}\"", Self::MARK_ID))
            .context("Cannot set temporary mark on focused workspace")?;

        let subtree = workspace
            .nodes
            .iter()
            .fold(Vec::new(), |mut subtree, node| {
                subtree.extend(node.nodes.as_slice());
                subtree
            });

        let move_result = self.move_recursively_on_mark(subtree, Self::MARK_ID);

        self.command_executor
            .run(format!("unmark \"{}\"", Self::MARK_ID))
            .context("Cannot unset temporary mark")?;

        move_result
    }

    /// It recursively moves all nodes in a subtree to a specific mark.
    fn move_recursively_on_mark(&mut self, mut subtree: Vec<&I3Node>, mark_id: &str) -> Result<()> {
        while let Some(current) = subtree.pop() {
            let _ = set_node_layout(current.id, Layout::Default, &mut self.command_executor);

            self.command_executor
                .run_on_node_id(current.id, format!("move window to mark \"{}\"", mark_id))
                .context("Cannot mode window on mark")?;

            subtree.extend(current.nodes.as_slice());
        }

        Ok(())
    }
}
