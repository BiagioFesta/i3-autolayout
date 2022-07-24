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

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use i3_ipc::I3Stream;

/// Execute i3 command.
pub fn execute_i3_command<S>(i3_stream: &mut I3Stream, command: S) -> Result<()>
where
    S: AsRef<str>,
{
    i3_stream
        .run_command(command)
        .context("Cannot execute command")?
        .into_iter()
        .map(|result| {
            if result.success {
                Ok(())
            } else {
                Err(anyhow!(
                    "Command failed with: {}",
                    result.error.unwrap_or_else(|| "N/A".to_string())
                ))
            }
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}
