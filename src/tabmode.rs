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

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use i3_ipc::reply::Node;
use i3_ipc::Connect;
use i3_ipc::I3Stream;
use i3_ipc::I3;

/// TabMode executor.
///
/// It represent a one-shot executor which normalizes the current active workspace
/// and display all nodes in tabbed mode.
pub struct TabMode {
    i3_stream: I3Stream,
}

impl TabMode {
    const MARK_ID: &'static str = "__i3-autolayout__tmp_ID";

    /// Initialize and the create the executor.
    ///
    /// It connects to i3 IPC.
    pub fn new() -> Result<Self> {
        I3::connect()
            .context("Cannot connect to I3")
            .map(|i3_stream| Self { i3_stream })
    }

    /// Execute the action.
    ///
    /// It normalizes the current active workspace and displays all nodes
    /// it a tabbed layout.
    pub fn execute(&mut self) -> Result<()> {
        let workspace = self.get_focus_workspace()?;
        let workspace_id = workspace.id;

        self.normalize_workspace(workspace)?;

        self.run_cmd(format!("[con_id={}] layout tabbed", workspace_id))
            .context("Cannot set tabbed layout on workspace")?;

        Ok(())
    }

    fn get_focus_workspace(&mut self) -> Result<Node> {
        let workspace_id = self
            .i3_stream
            .get_workspaces()
            .context("Cannot get list of workspaces from I3")?
            .into_iter()
            .find(|workspace| workspace.focused)
            .ok_or_else(|| anyhow!("Cannot detect the active workspace"))?
            .id;

        let root = self
            .i3_stream
            .get_tree()
            .context("Cannot obtain the tree root")?;

        let mut dfs = vec![root];

        while let Some(current) = dfs.pop() {
            if current.id == workspace_id {
                return Ok(current);
            }

            dfs.extend(current.nodes);
        }

        Err(anyhow!("Cannot find workspace associated with id"))
    }

    fn run_cmd<S>(&mut self, payload: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.i3_stream
            .run_command(payload)
            .context("Cannot execute command")?
            .into_iter()
            .map(|result| {
                if result.success {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Command failed with: {}",
                        result.error.unwrap_or_else(|| "N/A".to_string())
                    ))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(())
    }

    fn set_default_layout(&mut self, node: &Node) -> Result<()> {
        self.run_cmd(format!("[con_id={}] layout default", node.id))
            .context("Cannot set default layout for node")
    }

    fn normalize_workspace(&mut self, workspace: Node) -> Result<()> {
        self.run_cmd(format!(
            "[con_id={}] mark \"{}\"",
            workspace.id,
            Self::MARK_ID
        ))
        .context("Cannot set temporary mark")?;

        let normalize_result = || -> Result<()> {
            let mut dfs = workspace
                .nodes
                .into_iter()
                .fold(Vec::new(), |mut dfs, node| {
                    let _ = self.set_default_layout(&node);

                    dfs.extend(node.nodes);
                    dfs
                });

            while let Some(current) = dfs.pop() {
                let _ = self.set_default_layout(&current);

                self.run_cmd(format!(
                    "[con_id={}] move window to mark \"{}\"",
                    current.id,
                    Self::MARK_ID
                ))
                .context("Cannot move window")?;

                dfs.extend(current.nodes);
            }

            Ok(())
        }();

        self.run_cmd(format!("unmark \"{}\"", Self::MARK_ID))
            .context("Cannot remove temporary mark")?;

        normalize_result
    }
}
