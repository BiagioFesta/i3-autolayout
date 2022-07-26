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
use crate::event_listener::EventListener;
use crate::utilities::find_node_parent;
use crate::utilities::find_workspace_of_node;
use crate::utilities::is_floating_container;
use crate::utilities::set_node_split;
use crate::utilities::Split;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use i3_ipc::event::Event;
use i3_ipc::event::WindowChange;
use i3_ipc::reply::NodeLayout;

/// The size ratio for a rectangle container.
enum RectRatio {
    /// Width greater or equal than height.
    Horizontal,

    /// Height greater than width.
    Vertical,
}

impl RectRatio {
    /// If ratio is vertical.
    ///
    /// Same as: `matches!(sefl, RectRatio::Vertical)`.
    fn is_vertical(&self) -> bool {
        matches!(self, RectRatio::Vertical)
    }
}

/// AutoLayout service.
///
/// It represent the service which implements the auto-layout functionality.
pub struct AutoLayout {
    /// Event listener.
    event_listener: EventListener,

    /// Command executor.
    command_executor: CommandExecutor,
}

impl AutoLayout {
    /// Initialize and create the service.
    pub fn new(event_listener: EventListener, command_executor: CommandExecutor) -> Self {
        Self {
            event_listener,
            command_executor,
        }
    }

    /// Run the service.
    ///
    /// Start the service itself within this *blocking* function.
    /// It only returns when the service stops for some critical error.
    pub fn serve(mut self) -> Result<()> {
        loop {
            let event = self.event_listener.receive_event()?;

            debug_assert!(
                matches!(event, Event::Window(_)),
                "Received an unexpected event"
            );

            if let Event::Window(window_data) = event {
                if let WindowChange::Focus = window_data.change {
                    let node = window_data.container;
                    let result = self.on_window_focus(&node).with_context(|| {
                        format!(
                            "AutoLayout failure for window [{}; '{:?}'; '{:?}'; {}]",
                            node.id, node.name, node.floating, node.focused,
                        )
                    });

                    if let Err(error) = result {
                        println!(
                            "[WARN]: Failure to set split mode for focused window: {:?}",
                            error
                        );
                    }
                }
            }
        }
    }

    /// Logic to trigger when receiving a Window/Focus event.
    fn on_window_focus(&mut self, node: &I3Node) -> Result<()> {
        if is_floating_container(node) {
            return Ok(());
        }

        let root_node = self.command_executor.query_root_node()?;
        let parent_node = find_node_parent(node.id, &root_node)
            .ok_or_else(|| anyhow!("Cannot find parent of focused window"))?;

        match parent_node.layout {
            NodeLayout::SplitH | NodeLayout::SplitV => {
                let split = match find_workspace_of_node(node.id, &root_node) {
                    Some(workspace) if Self::ratio_of_node(workspace).is_vertical() => {
                        Split::Vertical
                    }
                    _ => match Self::ratio_of_node(node) {
                        RectRatio::Horizontal => Split::Horizontal,
                        RectRatio::Vertical => Split::Vertical,
                    },
                };

                set_node_split(node.id, split, &mut self.command_executor)
            }
            _ => Ok(()),
        }
    }

    /// Check the ratio of a node.
    fn ratio_of_node(node: &I3Node) -> RectRatio {
        if node.window_rect.height > node.window_rect.width {
            RectRatio::Vertical
        } else {
            RectRatio::Horizontal
        }
    }
}
