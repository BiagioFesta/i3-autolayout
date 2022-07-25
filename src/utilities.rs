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
use anyhow::Context;
use anyhow::Result;
use i3_ipc::reply::NodeType;

pub enum Layout {
    Default,
    Tabbed,
}

pub enum Split {
    Horizontal,
    Vertical,
}

pub fn find_node_by_id(node_id: usize, root_node: &RootNode) -> Option<&I3Node> {
    let mut dfs = vec![root_node.i3_node()];

    while let Some(current) = dfs.pop() {
        if current.id == node_id {
            return Some(current);
        }

        dfs.extend(current.nodes.as_slice())
    }

    None
}

pub fn find_node_parent(node_id: usize, root_node: &RootNode) -> Option<&I3Node> {
    // It's not a real problem, but a waste of CPU cycles
    debug_assert!(node_id != root_node.i3_node().id);

    let mut dfs: Vec<(&I3Node, &I3Node)> = root_node
        .i3_node()
        .nodes
        .iter()
        .map(|n| (n, root_node.i3_node()))
        .collect();

    while let Some((current, parent)) = dfs.pop() {
        if current.id == node_id {
            return Some(parent);
        }

        dfs.extend(current.nodes.iter().map(|n| (n, current)));
    }

    None
}

pub fn find_workspace_of_node(node_id: usize, root_node: &RootNode) -> Option<&I3Node> {
    let mut workspace = None;
    let mut dfs = vec![root_node.i3_node()];

    while let Some(current) = dfs.pop() {
        if current.node_type == NodeType::Workspace {
            workspace = Some(current);
        }

        if current.id == node_id {
            break;
        }

        dfs.extend(current.nodes.as_slice());
    }

    workspace
}

pub fn set_node_layout(
    node_id: usize,
    layout: Layout,
    command_executor: &mut CommandExecutor,
) -> Result<()> {
    let layout_cmd = match layout {
        Layout::Default => "layout default",
        Layout::Tabbed => "layout tabbed",
    };

    command_executor
        .run_on_node_id(node_id, layout_cmd)
        .with_context(|| format!("Cannot set layout for a node ('{}')", layout_cmd))
}

pub fn set_node_split(
    node_id: usize,
    split: Split,
    command_executor: &mut CommandExecutor,
) -> Result<()> {
    let split_cmd = match split {
        Split::Horizontal => "split horizontal",
        Split::Vertical => "split vertical",
    };

    command_executor
        .run_on_node_id(node_id, split_cmd)
        .with_context(|| format!("Cannot split a node ('{}')", split_cmd))
}
