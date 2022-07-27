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

#![warn(missing_docs)]

//! i3-autolayout is a simple service which helps keep a reasonable
//! windows layout for your i3 manager.

use crate::autolayout::AutoLayout;
use crate::command_executor::CommandExecutor;
use crate::event_listener::EventListener;
use crate::event_listener::EventSubscribe;
use crate::tabmode::TabMode;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use print_tree::print_tree;
use utilities::find_node_by_id;

/// CLI arguments.
#[derive(clap::Parser)]
#[clap(about, author, version)]
struct CliArgs {
    /// The subcommand to apply.
    #[clap(subcommand)]
    command: Command,
}

/// Subcommand of CLI.
#[derive(clap::Subcommand)]
enum Command {
    /// Run autolayout service.
    #[clap(name = "autolayout")]
    Autolayout,

    /// Toggle tabmode on the current focused workspace.
    #[clap(name = "tabmode")]
    TabMode,

    /// Display i3 information.
    #[clap(name = "i3version")]
    I3Version,

    /// Print a snapshot of the current layout as tree.
    #[clap(name = "print-tree")]
    PrintTree(PrintTreeCmd),
}

/// Information about the print-tree command.
#[derive(clap::Args)]
struct PrintTreeCmd {
    /// The workspace number to print of. If not specified prints all workspaces.
    workspace_num: Option<i32>,
}

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    match cli_args.command {
        Command::Autolayout => command_autolayout().context("Failure in command 'autolayout'"),
        Command::TabMode => command_tabmode().context("Failure in command 'tabmode'"),
        Command::I3Version => command_i3_version().context("Failure in command 'i3version'"),
        Command::PrintTree(print_tree_cmd) => {
            command_print_tree(print_tree_cmd).context("Failure in command 'print-tree'")
        }
    }
}

/// Execute autolayout service.
fn command_autolayout() -> Result<()> {
    let event_listener = EventListener::new(&[EventSubscribe::Window])?;
    let command_executor = CommandExecutor::new()?;
    let autolayout = AutoLayout::new(event_listener, command_executor);

    autolayout.serve()
}

/// Execute tabmode.
fn command_tabmode() -> Result<()> {
    let command_executor = CommandExecutor::new()?;
    let tabmode = TabMode::new(command_executor);

    tabmode.execute()
}

/// Display i3 information.
fn command_i3_version() -> Result<()> {
    let mut command_executor = CommandExecutor::new()?;
    let i3_version = command_executor.query_i3_version()?;

    println!(
        "I3 version: '{}'\n\
         Config File: '{}'",
        i3_version.human_readable, i3_version.loaded_config_file_name
    );

    Ok(())
}

/// Print the snapshot of I3 layout in the tree fashion.
fn command_print_tree(print_tree_cmd: PrintTreeCmd) -> Result<()> {
    let mut command_executor = CommandExecutor::new()?;
    let root_node = command_executor.query_root_node()?;

    let node = match print_tree_cmd.workspace_num {
        Some(workspace_num) => {
            let workspace = command_executor
                .query_workspaces()?
                .into_iter()
                .find(|workspace| workspace.num == workspace_num)
                .ok_or_else(|| anyhow!("Cannot find the workspace number '{}'", workspace_num))?;

            find_node_by_id(workspace.id, &root_node)
                .context("Cannot find the workspace associated with the id")?
        }
        None => root_node.node(),
    };

    print_tree(node)
}

mod autolayout;
mod command_executor;
mod event_listener;
mod print_tree;
mod tabmode;
mod utilities;
