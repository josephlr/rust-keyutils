// Copyright (c) 2015, Ben Boeckel
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of this project nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR
// ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
// (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
// LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
// ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! Logon keys
//!
//! Logon keys are arbitrary keys that userspace cannot read once set.

use crates::libkeyutils_sys::KEY_TYPE_LOGON;

use keytype::*;

use std::borrow::Cow;

/// Keys which can only be created and updated from userspace but not read back.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Logon;

impl KeyType for Logon {
    /// Logon key descriptions are free-form.
    type Description = Description;
    /// Logon payloads are free-form.
    type Payload = [u8];

    fn name() -> &'static str {
        KEY_TYPE_LOGON
    }
}

/// The description of a logon key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description {
    /// They subtype of the key.
    pub subtype: String,
    /// The description of the key.
    pub description: String,
}

impl KeyDescription for Description {
    fn description(&self) -> Cow<str> {
        format!("{}:{}", self.subtype, self.description).into()
    }
}