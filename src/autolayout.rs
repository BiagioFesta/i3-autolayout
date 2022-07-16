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

use anyhow::Context;
use anyhow::Result;
use i3_ipc::event::Event;
use i3_ipc::event::Subscribe;
use i3_ipc::event::WindowChange;
use i3_ipc::reply::Node;
use i3_ipc::reply::NodeLayout;
use i3_ipc::reply::NodeType;
use i3_ipc::I3Stream;

#[derive(Clone, Copy, Eq, PartialEq)]
enum SplitLayout {
    Horizontal,
    Vertical,
}

impl SplitLayout {
    fn is_vertical(&self) -> bool {
        matches!(self, SplitLayout::Vertical)
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
        match node.layout {
            NodeLayout::SplitH | NodeLayout::SplitV => {
                let split_layout = match self.find_parent_workspace(&node) {
                    Some(workspace) if Self::layout_of_node(&workspace).is_vertical() => {
                        SplitLayout::Vertical
                    }
                    _ => Self::layout_of_node(&node),
                };

                self.set_split_layout(split_layout)
            }
            _ => Ok(()),
        }
    }

    fn set_split_layout(&mut self, split_layout: SplitLayout) -> Result<()> {
        let split_cmd = match split_layout {
            SplitLayout::Horizontal => "split horizontal",
            SplitLayout::Vertical => "split vertical",
        };

        self.i3_stream
            .run_command(split_cmd)
            .context("Cannot execute split command")?;

        Ok(())
    }

    fn find_parent_workspace(&mut self, node: &Node) -> Option<Node> {
        let root = self
            .i3_stream
            .get_tree()
            .expect("Cannot obtain root-node tree");

        let mut workspace = None;
        let mut dfs = vec![root];

        while let Some(current) = dfs.pop() {
            if current.node_type == NodeType::Workspace {
                workspace = Some(current.clone());
            }

            if current.id == node.id {
                break;
            }

            dfs.extend(current.nodes);
        }

        workspace
    }

    fn layout_of_node(node: &Node) -> SplitLayout {
        if node.window_rect.height > node.window_rect.width {
            SplitLayout::Vertical
        } else {
            SplitLayout::Horizontal
        }
    }
}
