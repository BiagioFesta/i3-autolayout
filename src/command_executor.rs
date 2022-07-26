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
use i3_ipc::Connect;
use i3_ipc::I3Stream;
use i3_ipc::I3;
use std::fmt::Display;

/// The I3 version data.
pub type I3Version = i3_ipc::reply::Version;

/// An I3 workspace.
pub type I3Workspace = i3_ipc::reply::Workspace;

/// An I3 node.
pub type I3Node = i3_ipc::reply::Node;

/// A connection with I3 IPC for command execution.
pub struct CommandExecutor {
    /// The connection with I3 for IPC.
    i3_stream: I3Stream,
}

impl CommandExecutor {
    /// Connect to I3.
    pub fn new() -> Result<Self> {
        println!("Creating command executor...");
        let i3_stream = I3::connect().context("Cannot create command executor")?;
        println!("  Ok");

        Ok(Self { i3_stream })
    }

    /// Execute an I3 command.
    pub fn run<C>(&mut self, command: C) -> Result<()>
    where
        C: AsRef<str>,
    {
        let response = self
            .i3_stream
            .run_command(command)
            .context("Cannot execute the command")?;

        for resp in response.into_iter() {
            if !resp.success {
                return Err(anyhow!(
                    "Command execution returned a failure response: '{}'",
                    resp.error.unwrap_or_else(|| "N/A".to_string())
                ));
            }
        }

        Ok(())
    }

    /// Execute an I3 command on a particular node.
    pub fn run_on_node_id<C>(&mut self, node_id: usize, command: C) -> Result<()>
    where
        C: Display,
    {
        self.run(format!("[con_id={}] {}", node_id, command))
    }

    /// Return a list of all workspaces.
    pub fn query_workspaces(&mut self) -> Result<Vec<I3Workspace>> {
        self.i3_stream
            .get_workspaces()
            .context("Cannot query i3 workspaces")
    }

    /// Return the current snapshot of I3 state as root node.
    pub fn query_root_node(&mut self) -> Result<RootNode> {
        Ok(RootNode(
            self.i3_stream
                .get_tree()
                .context("Cannot query i3 root-node")?,
        ))
    }

    /// Return I3 version.
    pub fn query_i3_version(&mut self) -> Result<I3Version> {
        self.i3_stream
            .get_version()
            .context("Cannot query i3 version")
    }
}

/// The root node of I3 containers.
pub struct RootNode(I3Node);

impl RootNode {
    /// As I3 node.
    pub fn node(&self) -> &I3Node {
        &self.0
    }
}
