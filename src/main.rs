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
use anyhow::Context;
use anyhow::Result;
use clap::Parser;

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
    /// Autolayout service.
    #[clap(name = "autolayout")]
    Autolayout,

    /// TabMode executor.
    #[clap(name = "tabmode")]
    TabMode,

    /// Display i3 information.
    #[clap(name = "i3version")]
    I3Version,
}

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    match cli_args.command {
        Command::Autolayout => command_autolayout().context("Failure in command 'autolayout'"),
        Command::TabMode => command_tabmode().context("Failure in command 'tabmode'"),
        Command::I3Version => command_i3_version().context("Failure in command 'i3version'"),
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

mod autolayout;
mod command_executor;
mod event_listener;
mod tabmode;
mod utilities;
