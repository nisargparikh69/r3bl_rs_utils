/*
 *   Copyright (c) 2022 R3BL LLC
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

use std::{collections::HashMap,
          fmt::{Display, Formatter}};

use r3bl_tui::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct State {
  pub buffers: HashMap<String, EditorBuffer>,
}

impl Display for State {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "State {{ buffers: {:?} }}", self.buffers) }
}

impl HasEditorBuffers for State {
  fn get_editor_buffer(&self, id: &str) -> Option<&EditorBuffer> {
    if let Some(buffer) = self.buffers.get(id) {
      Some(buffer)
    } else {
      None
    }
  }
}
