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
use i3_ipc::event::Subscribe;
use i3_ipc::I3Stream;

pub type I3Event = i3_ipc::event::Event;

#[derive(Copy, Clone)]
pub enum EventSubscribe {
    Window,
}

pub struct EventListener {
    i3_stream: I3Stream,
}

impl EventListener {
    pub fn new(event_subscribe: &[EventSubscribe]) -> Result<Self> {
        println!("Creating event listener...");
        let i3_stream = I3Stream::conn_sub(
            event_subscribe
                .iter()
                .map(|&e| e.into())
                .collect::<Vec<_>>(),
        )
        .context("Cannot create event listener")?;
        println!("  Ok");

        Ok(Self { i3_stream })
    }

    pub fn receive_event(&mut self) -> Result<I3Event> {
        self.i3_stream
            .receive_event()
            .context("Cannot receive event from i3 listener")
    }
}

impl From<EventSubscribe> for Subscribe {
    fn from(e: EventSubscribe) -> Self {
        match e {
            EventSubscribe::Window => Subscribe::Window,
        }
    }
}
