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

use crate::command_executor::I3Node;
use anyhow::Context;
use anyhow::Result;
use ptree::TreeItem;
use std::borrow::Cow;
use std::io::Write;

/// Print the tree associated starting from a root node.
pub fn print_tree(node: &I3Node) -> Result<()> {
    ptree::print_tree(&TreeNode(node)).context("Cannot print i3 tree")
}

#[derive(Clone)]
struct TreeNode<'a>(&'a I3Node);

impl<'a> TreeItem for TreeNode<'a> {
    type Child = TreeNode<'a>;

    fn write_self<W>(&self, f: &mut W, _style: &ptree::Style) -> std::io::Result<()>
    where
        W: Write,
    {
        write!(f,
               "[ID: {id}; \
                Type: {type:?}; \
                Name: {name:?}; \
                Layout: {layout:?}; \
                WinType: {wintype:?}; \
                NumFloatings: {num_floats}]",
               id = self.0.id,
               type = self.0.node_type,
               name = self.0.name,
               layout = self.0.layout,
               wintype = self.0.window_type,
               num_floats = self.0.floating_nodes.len(),
        )
    }

    fn children(&self) -> Cow<[Self::Child]> {
        Cow::from(self.0.nodes.iter().map(TreeNode).collect::<Vec<_>>())
    }
}
