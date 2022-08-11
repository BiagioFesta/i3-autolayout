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

pub struct SaveLayout {
    command_executor: CommandExecutor,
}

impl SaveLayout {
    pub fn new(command_executor: CommandExecutor) -> Self {
        Self { command_executor }
    }

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

        Ok(SavedLayout::new(SavedNodes(nodes)))
    }
}

pub struct SavedLayout {
    nodes: SavedNodes,
    map_id: HashMap<NodeId, NodeIndex>,
}

impl SavedLayout {
    fn new(nodes: SavedNodes) -> Self {
        let map_id = nodes
            .0
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id, index))
            .collect();

        Self { nodes, map_id }
    }

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

    pub fn deserialize<R>(reader: R, json_input: bool) -> Result<Self>
    where
        R: Read,
    {
        let nodes = if json_input {
            let mut deserializer = serde_json::de::Deserializer::from_reader(reader);

            SavedNodes::deserialize(&mut deserializer).context("Cannot JSON deserialize layout")?
        } else {
            let options = Self::bincode_options();
            let mut deserializer = bincode::de::Deserializer::with_reader(reader, options);

            SavedNodes::deserialize(&mut deserializer)
                .context("Cannot binary deserialize layout")?
        };

        Ok(Self::new(nodes))
    }

    pub fn root(&self) -> &SavedNode {
        self.nodes
            .0
            .first()
            .expect("Expected at least workspace node")
    }

    pub fn lookup_by_id(&self, node_id: usize) -> Option<&SavedNode> {
        self.map_id
            .get(&node_id)
            .map(|node_index| &self.nodes.0[*node_index])
    }

    fn bincode_options() -> impl BinCodeOptions {
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes()
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct SavedNodes(Vec<SavedNode>);

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SavedNode {
    id: NodeId,
    kind: KindNode,
    layout: LayoutNode,
    children: Vec<NodeId>,
}

impl SavedNode {
    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn kind(&self) -> &KindNode {
        &self.kind
    }

    pub fn layout(&self) -> LayoutNode {
        self.layout
    }

    pub fn children(&self) -> &[NodeId] {
        self.children.as_slice()
    }
}

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
            _ => Err(anyhow!("Unexpcted layout to save: '{:?}'", layout)),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum KindNode {
    Workspace(WorkspaceNum),
    NormalWindow,
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
                    Ok(Self::NormalWindow)
                } else {
                    Ok(Self::Splitter)
                }
            }

            _ => Err(anyhow!("Unexpected node type: '{:?}'", node.node_type)),
        }
    }
}
