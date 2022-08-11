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
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use i3_ipc::reply::Floating;
use i3_ipc::reply::NodeType;

/// The node layout.
pub enum Layout {
    /// Default layout.
    Default,

    /// Tabbed layout.
    Tabbed,

    /// Split horizontal layout.
    SplitH,

    /// Split vertical layout.
    SplitV,

    /// Stacked layout.
    Stacked,
}

/// A split operation request.
pub enum Split {
    /// Split horizontal.
    Horizontal,

    /// Split vertical.
    Vertical,
}

/// The size ratio for a rectangle container.
pub enum RectRatio {
    /// Width greater or equal than height.
    Horizontal,

    /// Height greater than width.
    Vertical,
}

impl RectRatio {
    /// If ratio is vertical.
    ///
    /// Same as: `matches!(sefl, RectRatio::Vertical)`.
    pub fn is_vertical(&self) -> bool {
        matches!(self, RectRatio::Vertical)
    }
}

/// Find a node by id.
#[allow(unused)]
pub fn find_node_by_id(node_id: usize, root_node: &RootNode) -> Option<&I3Node> {
    let mut dfs = vec![root_node.node()];

    while let Some(current) = dfs.pop() {
        if current.id == node_id {
            return Some(current);
        }

        dfs.extend(current.nodes.as_slice())
    }

    None
}

/// Find a node's parent.
pub fn find_node_parent(node_id: usize, root_node: &RootNode) -> Option<&I3Node> {
    // It's not a real problem, but a waste of CPU cycles
    debug_assert!(node_id != root_node.node().id);

    let mut dfs: Vec<(&I3Node, &I3Node)> = root_node
        .node()
        .nodes
        .iter()
        .map(|n| (n, root_node.node()))
        .collect();

    while let Some((current, parent)) = dfs.pop() {
        if current.id == node_id {
            return Some(parent);
        }

        dfs.extend(current.nodes.iter().map(|n| (n, current)));
    }

    None
}

/// Find the workspace which contains the node.
///
/// Note: a window might not be always associated with a workspace.
/// For instance, floating windows or windows on scratchpad.
pub fn find_workspace_of_node(node_id: usize, root_node: &RootNode) -> Option<&I3Node> {
    let mut workspace = None;
    let mut dfs = vec![root_node.node()];

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

/// Find all I3 nodes in the tree that are workspaces type.
pub fn find_workspaces(root_node: &RootNode) -> Vec<&I3Node> {
    let mut workspaces = vec![];
    let mut dfs = vec![root_node.node()];

    while let Some(current) = dfs.pop() {
        if current.node_type == NodeType::Workspace {
            workspaces.push(current);
        } else {
            dfs.extend(current.nodes.as_slice());
        }
    }

    workspaces
}

/// Find the workspace associated with the number in the nodes tree.
pub fn find_workspace_by_num(root_node: &RootNode, workspace_num: i32) -> Option<&I3Node> {
    find_workspaces(root_node).into_iter().find(|workspace| {
        debug_assert_eq!(workspace.node_type, NodeType::Workspace);

        let num = workspace
            .num
            .expect("The node is expected to have a number as workspace");

        num == workspace_num
    })
}

/// Query and retrieve the currently focused workspace.
///
/// It queries via `command_executor` the currently focused workspace number;
/// afterwards, it performs a research across the give nodes tree for that workspace-node.
///
/// *Note*: the `root_node` might be inconsistent (older state-snapshot).
pub fn query_workspace_focused<'a>(
    root_node: &'a RootNode,
    command_executor: &mut CommandExecutor,
) -> Result<&'a I3Node> {
    let workspace_num = command_executor
        .query_workspaces()?
        .into_iter()
        .find(|workspace| workspace.focused)
        .ok_or_else(|| anyhow!("Cannot detect the current focused workspace"))?
        .num;

    find_workspace_by_num(root_node, workspace_num)
        .ok_or_else(|| anyhow!("Cannot find the workspace number '{}'", workspace_num))
}

/// Set the layout for a particular node.
pub fn set_node_layout(
    node_id: usize,
    layout: Layout,
    command_executor: &mut CommandExecutor,
) -> Result<()> {
    let layout_cmd = match layout {
        Layout::Default => "layout default",
        Layout::Tabbed => "layout tabbed",
        Layout::SplitH => "layout splith",
        Layout::SplitV => "layout splitv",
        Layout::Stacked => "layout stacked",
    };

    command_executor
        .run_on_node_id(node_id, layout_cmd)
        .with_context(|| format!("Cannot set layout for a node ('{}')", layout_cmd))
}

/// Set a split operation for a particular node.
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

/// Check whether the node is a floating container or not.
pub fn is_floating_container(node: &I3Node) -> bool {
    match node.floating {
        Some(Floating::AutoOn) | Some(Floating::UserOn) => true,
        None | Some(Floating::AutoOff) | Some(Floating::UserOff) => false,
    }
}

/// Check the ratio of a node.
pub fn ratio_of_node(node: &I3Node) -> RectRatio {
    if node.window_rect.height > node.window_rect.width {
        RectRatio::Vertical
    } else {
        RectRatio::Horizontal
    }
}
