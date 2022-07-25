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
use crate::command_executor::RootNode;
use crate::utilities::find_node_by_id;
use crate::utilities::set_node_layout;
use crate::utilities::Layout;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;

/// TabMode executor.
///
/// It represent a one-shot executor which normalizes the current active workspace
/// and display all nodes in tabbed mode.
pub struct TabMode {
    command_executor: CommandExecutor,
}

impl TabMode {
    const MARK_ID: &'static str = "__i3-autolayout__tmp_ID";

    /// Initialize and the create the executor.
    ///
    /// It connects to i3 IPC.
    pub fn new(command_executor: CommandExecutor) -> Self {
        Self { command_executor }
    }

    /// Execute the action.
    ///
    /// It normalizes the current active workspace and displays all nodes
    /// it a tabbed layout.
    pub fn execute(mut self) -> Result<()> {
        let root_node = self.command_executor.query_root_node()?;
        let workspace = self.get_focus_workspace(&root_node)?;

        self.normalize_workspace(workspace)?;

        set_node_layout(workspace.id, Layout::Tabbed, &mut self.command_executor)
            .context("Cannot layout for focused workspace")
    }

    fn get_focus_workspace<'a>(&mut self, root_node: &'a RootNode) -> Result<&'a I3Node> {
        let focused_workspace_id = self
            .command_executor
            .query_workspaces()?
            .into_iter()
            .find(|workspace| workspace.focused)
            .ok_or_else(|| anyhow!("Cannot detect the current focused workspace"))?
            .id;

        find_node_by_id(focused_workspace_id, root_node)
            .ok_or_else(|| anyhow!("Cannot find focused workspace associated with the id"))
    }

    fn normalize_workspace(&mut self, workspace: &I3Node) -> Result<()> {
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
