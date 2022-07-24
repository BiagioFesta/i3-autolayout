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

use crate::utilities::execute_i3_command;
use anyhow::Context;
use anyhow::Result;
use i3_ipc::event::Event;
use i3_ipc::event::Subscribe;
use i3_ipc::event::WindowChange;
use i3_ipc::reply::Node;
use i3_ipc::reply::NodeLayout;
use i3_ipc::reply::NodeType;
use i3_ipc::I3Stream;

type NodeId = usize;

enum SplitMode {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum RectRatio {
    Horizontal,
    Vertical,
}

impl RectRatio {
    fn is_vertical(&self) -> bool {
        matches!(self, RectRatio::Vertical)
    }
}

/// AutoLayout service.
///
/// It represent the service which implements the auto-layout functionality.
pub struct AutoLayout {
    i3_stream: I3Stream,
}

impl AutoLayout {
    /// Initialize and create the service.
    ///
    /// Create the service starting a connection with the i3 window manager.
    pub fn new() -> Result<Self> {
        println!("Connecting and subscribing i3 events");

        I3Stream::conn_sub(&[Subscribe::Window])
            .map(|i3_stream| {
                println!("Connected");

                Self { i3_stream }
            })
            .context("Cannot connect to i3")
    }

    /// Run the service.
    ///
    /// Start the service itself within this blocking function.
    /// It only returns when the service stops for some error.
    pub fn serve(mut self) -> Result<()> {
        loop {
            let event = self
                .i3_stream
                .receive_event()
                .context("Cannot receive event")?;

            if let Event::Window(window_data) = event {
                if let WindowChange::Focus = window_data.change {
                    self.on_window_focus(window_data.container)?;
                }
            }
        }
    }

    fn on_window_focus(&mut self, node: Node) -> Result<()> {
        let root = self.build_root();
        let parent = Self::find_parent(&node, &root).unwrap();

        match parent.layout {
            NodeLayout::SplitH | NodeLayout::SplitV => {
                let split_layout = match Self::find_parent_workspace(&node, &root) {
                    Some(workspace) if Self::ratio_of_node(&workspace).is_vertical() => {
                        SplitMode::Vertical
                    }
                    _ => match Self::ratio_of_node(&node) {
                        RectRatio::Horizontal => SplitMode::Horizontal,
                        RectRatio::Vertical => SplitMode::Vertical,
                    },
                };

                self.set_split_layout(split_layout)
            }
            _ => Ok(()),
        }
    }

    fn set_split_layout(&mut self, split_mode: SplitMode) -> Result<()> {
        let split_cmd = match split_mode {
            SplitMode::Horizontal => "split horizontal",
            SplitMode::Vertical => "split vertical",
        };

        execute_i3_command(&mut self.i3_stream, split_cmd).context("Cannot execute split command")?;

        Ok(())
    }

    fn find_parent_workspace<'a>(node: &'a Node, root: &'a Node) -> Option<&'a Node> {
        let mut workspace_id = None;
        let mut dfs = vec![root];

        while let Some(current) = dfs.pop() {
            if current.node_type == NodeType::Workspace {
                workspace_id = Some(current.id);
            }

            if current.id == node.id {
                return workspace_id.map(|ws_id| {
                    Self::find_node(ws_id, root).expect("Cannot find node associated with id")
                });
            }

            dfs.extend(current.nodes.iter().map(|c| c))
        }

        None
    }

    fn find_parent<'a>(node: &'a Node, root: &'a Node) -> Option<&'a Node> {
        let mut dfs = root.nodes.iter().map(|c| (c, root.id)).collect::<Vec<_>>();

        while let Some((current, parent)) = dfs.pop() {
            if current.id == node.id {
                return Some(
                    Self::find_node(parent, root).expect("Cannot find node associated with id"),
                );
            }

            dfs.extend(current.nodes.iter().map(|c| (c, current.id)));
        }

        None
    }

    fn find_node<'a>(id: NodeId, root: &'a Node) -> Option<&'a Node> {
        let mut dfs = vec![root];

        while let Some(current) = dfs.pop() {
            if current.id == id {
                return Some(current);
            }

            dfs.extend(current.nodes.iter().map(|c| c));
        }

        None
    }

    fn build_root(&mut self) -> Node {
        self.i3_stream
            .get_tree()
            .expect("Cannot obtain root-node tree")
    }

    fn ratio_of_node(node: &Node) -> RectRatio {
        if node.window_rect.height > node.window_rect.width {
            RectRatio::Vertical
        } else {
            RectRatio::Horizontal
        }
    }
}
