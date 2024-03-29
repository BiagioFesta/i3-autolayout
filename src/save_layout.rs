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
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bincode::Options as BinCodeOptions;
use i3_ipc::reply::NodeLayout as I3NodeLayout;
use i3_ipc::reply::NodeType as I3NodeType;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;

type NodeId = usize;
type NodeIndex = usize;
type WorkspaceNum = i32;

/// SaveLayout executor.
///
/// Describe a snapshot of a workspace's layout.
/// Shortly, it allows saving a layout.
///
/// It supports two output format: binary and JSON.
pub struct SaveLayout {
    command_executor: CommandExecutor,
}

impl SaveLayout {
    /// A new SaveLayout executor.
    pub fn new(command_executor: CommandExecutor) -> Self {
        Self { command_executor }
    }

    /// Write the workspace's layout on `output`.
    ///
    /// Specify the workspace with `workspace_num`. If `None` the currently focused
    /// workspace will be saved.
    ///
    /// `json_output` for JSON format, otherwise binary format will be used.
    pub fn execute<W>(
        mut self,
        workspace_num: Option<i32>,
        output: W,
        json_output: bool,
    ) -> Result<()>
    where
        W: Write,
    {
        let root_node = self.command_executor.query_root_node()?;

        let workspace = match workspace_num {
            Some(workspace_num) => find_workspace_by_num(&root_node, workspace_num)
                .ok_or_else(|| anyhow!("Cannot find the workspace number '{}'", workspace_num))?,
            None => query_workspace_focused(&root_node, &mut self.command_executor)?,
        };

        Self::save_subtree(workspace)?.serialize(output, json_output)
    }

    fn save_subtree(subtree: &I3Node) -> Result<SavedLayout> {
        let mut nodes = vec![];
        let mut dfs = vec![subtree];

        while let Some(current) = dfs.pop() {
            nodes.push(SavedNode {
                id: current.id,
                kind: KindNode::new(current)?,
                layout: current.layout.try_into()?,
                children: current.nodes.iter().map(|node| node.id).collect(),
            });

            dfs.extend(current.nodes.as_slice());
        }

        SavedLayout::new(SavedNodes(nodes))
    }
}

/// SavedLayout
///
/// Representation of a layout of a workspace.
pub struct SavedLayout {
    nodes: SavedNodes,
    map_id: HashMap<NodeId, NodeIndex>,
}

impl SavedLayout {
    fn new(nodes: SavedNodes) -> Result<Self> {
        if nodes.0.is_empty() {
            return Err(anyhow!("Empty layout"));
        }

        let map_id: HashMap<NodeId, NodeIndex> = nodes
            .0
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id, index))
            .collect();

        let missing_node = nodes.0.iter().find_map(|node| {
            let missing_child_id = node
                .children
                .iter()
                .find(|child_id| !map_id.contains_key(child_id));

            missing_child_id.map(|missing_child_id| (node.id(), missing_child_id))
        });

        if let Some(missing_node) = missing_node {
            return Err(anyhow!(
                "The node '{}' has a missing child ('{}')",
                missing_node.0,
                missing_node.1
            ));
        }

        Ok(Self { nodes, map_id })
    }

    /// Serialize the layout into `output`.
    pub fn serialize<W>(&self, output: W, json_output: bool) -> Result<()>
    where
        W: Write,
    {
        if json_output {
            let mut serializer = serde_json::ser::Serializer::pretty(output);

            self.nodes
                .serialize(&mut serializer)
                .context("Cannot JSON serialize layout")?;
        } else {
            let options = Self::bincode_options();
            let mut serializer = bincode::Serializer::new(output, options);

            self.nodes
                .serialize(&mut serializer)
                .context("Cannot binary serialize layout")?;
        }

        Ok(())
    }

    /// Load a layout from `input`.
    pub fn deserialize<R>(input: R, json_input: bool) -> Result<Self>
    where
        R: Read,
    {
        let nodes = if json_input {
            let mut deserializer = serde_json::de::Deserializer::from_reader(input);

            SavedNodes::deserialize(&mut deserializer).context("Cannot JSON deserialize layout")?
        } else {
            let options = Self::bincode_options();
            let mut deserializer = bincode::de::Deserializer::with_reader(input, options);

            SavedNodes::deserialize(&mut deserializer)
                .context("Cannot binary deserialize layout")?
        };

        Self::new(nodes)
    }

    /// Get the first node (this should be the workspace).
    pub fn root(&self) -> &SavedNode {
        self.nodes
            .0
            .first()
            .expect("Expected at least workspace node")
    }

    /// Return the saved node given its id.
    pub fn lookup_by_id(&self, node_id: usize) -> &SavedNode {
        self.map_id
            .get(&node_id)
            .map(|node_index| &self.nodes.0[*node_index])
            .expect("Expected saved layout to be validated during construction")
    }

    fn bincode_options() -> impl BinCodeOptions {
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct SavedNodes(Vec<SavedNode>);

/// SavedNode
///
/// Representation of a node in the saved layout.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SavedNode {
    id: NodeId,
    kind: KindNode,
    layout: LayoutNode,
    children: Vec<NodeId>,
}

impl SavedNode {
    /// The id of the node.
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// The type of the node.
    pub fn kind(&self) -> &KindNode {
        &self.kind
    }

    /// The layout saved for this node.
    pub fn layout(&self) -> LayoutNode {
        self.layout
    }

    /// Node children.
    pub fn children(&self) -> &[NodeId] {
        self.children.as_slice()
    }
}

/// Saved layout applied for a saved node.
#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub enum LayoutNode {
    SplitH,
    SplitV,
    Stacked,
    Tabbed,
}

impl TryFrom<I3NodeLayout> for LayoutNode {
    type Error = anyhow::Error;

    fn try_from(layout: I3NodeLayout) -> Result<Self, Self::Error> {
        match layout {
            I3NodeLayout::SplitH => Ok(LayoutNode::SplitH),
            I3NodeLayout::SplitV => Ok(LayoutNode::SplitV),
            I3NodeLayout::Stacked => Ok(LayoutNode::Stacked),
            I3NodeLayout::Tabbed => Ok(LayoutNode::Tabbed),
            _ => Err(anyhow!("Unexpected layout to save: '{:?}'", layout)),
        }
    }
}

/// Type of saved node.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum KindNode {
    /// The saved node is a workspace.
    Workspace(WorkspaceNum),

    /// The node is a normal window (leaf of the tree).
    NormalWindow(SavedWindow),

    /// The node is a container (children >= 1; intermediate node in tree).
    Splitter,
}

impl KindNode {
    fn new(node: &I3Node) -> Result<Self> {
        match node.node_type {
            I3NodeType::Workspace => {
                let workspace_num = node.num.expect("Expected workspace having 'num' field");
                Ok(Self::Workspace(workspace_num))
            }

            I3NodeType::Con => {
                if node.nodes.is_empty() {
                    Ok(Self::NormalWindow(SavedWindow {
                        width: node.window_rect.width,
                        height: node.window_rect.height,
                    }))
                } else {
                    Ok(Self::Splitter)
                }
            }

            _ => Err(anyhow!("Unexpected node type: '{:?}'", node.node_type)),
        }
    }
}

/// Information about the saved window.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SavedWindow {
    width: isize,
    height: isize,
}

impl SavedWindow {
    pub fn width(&self) -> isize {
        self.width
    }

    pub fn height(&self) -> isize {
        self.height
    }
}
