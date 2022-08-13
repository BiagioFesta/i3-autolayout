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
use restore_layout::RestoreLayout;
use save_layout::SaveLayout;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use utilities::find_workspace_by_num;

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
    TabMode(TabModeCmd),

    /// Display i3 information.
    #[clap(name = "i3version")]
    I3Version,

    /// Print a snapshot of the current layout as tree.
    #[clap(name = "print-tree")]
    PrintTree(PrintTreeCmd),

    /// Save a workspace's layout.
    #[clap(name = "save-layout")]
    SaveLayout(SaveLayoutCmd),

    /// Restore a workspace's layout.
    #[clap(name = "restore-layout")]
    RestoreLayout(RestoreLayoutCmd),
}

/// Information about the tabmode command.
#[derive(clap::Args)]
struct TabModeCmd {
    /// The workspace number to apply tab mode. If not specified the focused workspace will be used.
    #[clap(short, long)]
    workspace_num: Option<i32>,

    /// The file where to save/load the layout.
    #[clap(short, long)]
    file_layout: Option<PathBuf>,
}

/// Information about the print-tree command.
#[derive(clap::Args)]
struct PrintTreeCmd {
    /// The workspace number to print of. If not specified prints all workspaces.
    workspace_num: Option<i32>,
}

/// Information about the save-layout command.
#[derive(clap::Args)]
struct SaveLayoutCmd {
    /// The workspace number to save. If not specified the focused workspace will be used.
    #[clap(short, long)]
    workspace_num: Option<i32>,

    /// The output filename where to save the layout. If not specified stdout will be used.
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Format the output with JSON.
    #[clap(short, long, action)]
    json: bool,
}

/// Information about the restore-layout command.
#[derive(clap::Args)]
struct RestoreLayoutCmd {
    /// The input filename where layout has been stored. If not specified stdin will be used.
    #[clap(short, long)]
    input: Option<PathBuf>,

    /// Whether the input is JSON format.
    #[clap(short, long, action)]
    json: bool,

    /// Whether to attempt to restore sizes of windows.
    #[clap(short, long, action)]
    restore_sizes: bool,
}

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    match cli_args.command {
        Command::Autolayout => command_autolayout().context("Failure in command 'autolayout'"),

        Command::TabMode(tabmode_cmd) => {
            command_tabmode(tabmode_cmd).context("Failure in command 'tabmode'")
        }

        Command::I3Version => command_i3_version().context("Failure in command 'i3version'"),

        Command::PrintTree(print_tree_cmd) => {
            command_print_tree(print_tree_cmd).context("Failure in command 'print-tree'")
        }

        Command::SaveLayout(save_layout_cmd) => {
            command_save_layout(save_layout_cmd).context("Failure in command 'save-layout'")
        }

        Command::RestoreLayout(restore_layout_cmd) => command_restore_layout(restore_layout_cmd)
            .context("Failure in command 'restore-layout'"),
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
fn command_tabmode(tabmode_cmd: TabModeCmd) -> Result<()> {
    let command_executor = CommandExecutor::new()?;
    let tabmode = TabMode::new(command_executor);

    tabmode.execute(
        tabmode_cmd.workspace_num,
        tabmode_cmd.file_layout.as_deref(),
    )
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
        Some(workspace_num) => find_workspace_by_num(&root_node, workspace_num)
            .ok_or_else(|| anyhow!("Cannot find the workspace number '{}'", workspace_num))?,

        None => root_node.node(),
    };

    print_tree(node)
}

/// Save a layout for a workspace.
fn command_save_layout(save_layout_cmd: SaveLayoutCmd) -> Result<()> {
    let command_executor = CommandExecutor::new()?;
    let save_layout = SaveLayout::new(command_executor);

    let output: Box<dyn Write> =
        match save_layout_cmd.output {
            Some(output_file) => Box::new(File::create(&output_file).with_context(|| {
                format!("Cannot create layout file '{}'", output_file.display())
            })?),

            None => Box::new(std::io::stdout()),
        };

    save_layout.execute(save_layout_cmd.workspace_num, output, save_layout_cmd.json)
}

/// Restore a previously saved layout on a workspace.
fn command_restore_layout(restore_layout_cmd: RestoreLayoutCmd) -> Result<()> {
    let command_executor = CommandExecutor::new()?;
    let restore_layout = RestoreLayout::new(command_executor);

    let input: Box<dyn Read> =
        match restore_layout_cmd.input {
            Some(input_file) => Box::new(File::open(&input_file).with_context(|| {
                format!("Cannot read the layout file '{}'", input_file.display())
            })?),

            None => Box::new(std::io::stdin()),
        };

    restore_layout.execute(
        input,
        restore_layout_cmd.json,
        restore_layout_cmd.restore_sizes,
    )
}

mod autolayout;
mod command_executor;
mod event_listener;
mod print_tree;
mod restore_layout;
mod save_layout;
mod tabmode;
mod utilities;
