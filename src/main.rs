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

use anyhow::Result;
use autolayout::AutoLayout;
use clap::Parser;
use clap::Subcommand;
use tabmode::TabMode;

#[derive(Parser)]
#[clap(author, version, about)]
struct CliArgs {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[clap(name = "autolayout")]
    Autolayout,

    #[clap(name = "tabmode")]
    TabMode,
}

fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    match cli_args.command {
        Command::Autolayout => AutoLayout::new()?.serve(),
        Command::TabMode => TabMode::new()?.execute(),
    }
}

mod autolayout;
mod tabmode;
