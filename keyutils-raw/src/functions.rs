// Copyright (c) 2018, Ben Boeckel
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

use std::ffi::{CStr, CString};
use std::io::Result;

use crate::{KeyringSerial, KeyPermissions, DefaultKeyring, TimeoutSeconds};
use libc::{gid_t, uid_t};

// TODO, when can optional be passed

pub fn add_key(
    type_: &CStr,
    description: &CStr,
    payload: &[u8],
    keyring: KeyringSerial,
) -> Result<KeyringSerial> {
    unimplemented!()
}

pub fn request_key(
    type_: &CStr,
    description: &CStr,
    callout_info: Option<&CStr>,
    keyring: Option<KeyringSerial>,
) -> Result<KeyringSerial> {
    unimplemented!()
}

pub fn keyctl_get_keyring_id(id: KeyringSerial, create: bool) -> Result<KeyringSerial> {
    unimplemented!()
}

pub fn keyctl_join_session_keyring(name: Option<&CStr>) -> Result<KeyringSerial> {
    unimplemented!()
}

pub fn keyctl_update(id: KeyringSerial, payload: &[u8]) -> Result<()> {
    unimplemented!()
}

pub fn keyctl_revoke(id: KeyringSerial) -> Result<()> {
    unimplemented!()
}

pub fn keyctl_chown(id: KeyringSerial, uid: uid_t, gid: gid_t) -> Result<()> {
    unimplemented!()
}

pub fn keyctl_setperm(id: KeyringSerial, perm: KeyPermissions) -> Result<()> {
    unimplemented!()
}

pub fn keyctl_describe(id: KeyringSerial, buffer: &[u8]) -> Result<usize> {
    unimplemented!()
}

pub fn keyctl_describe_alloc(id: KeyringSerial) -> Result<CString> {
    unimplemented!()
}

pub fn keyctl_clear(ringid: KeyringSerial) -> Result<()> {
    unimplemented!()
}

pub fn keyctl_link(id: KeyringSerial, ringid: KeyringSerial) -> Result<()> {
    unimplemented!()
}

pub fn keyctl_unlink(id: KeyringSerial, ringid: KeyringSerial) -> Result<()> {
    unimplemented!()
}

pub fn keyctl_search(
    ringid: KeyringSerial,
    type_: &CStr,
    description: &CStr,
    destringid: KeyringSerial,
) -> Result<KeyringSerial> {
    unimplemented!()
}

pub fn keyctl_read(id: KeyringSerial, buffer: &[u8]) -> Result<usize> {
    unimplemented!()
}

pub fn keyctl_read_alloc(id: KeyringSerial) -> Result<Vec<u8>> {
    unimplemented!()
}

pub fn keyctl_instantiate(id: KeyringSerial, payload: &[u8], ringid: KeyringSerial) -> Result<()> {
    unimplemented!()
}

pub fn keyctl_negate(
    id: KeyringSerial,
    timeout: TimeoutSeconds,
    ringid: KeyringSerial,
) -> Result<()> {
    unimplemented!()
}

pub fn keyctl_set_reqkey_keyring(reqkey_defl: DefaultKeyring) -> Result<DefaultKeyring> {
    unimplemented!()
}

pub fn keyctl_set_timeout(key: KeyringSerial, timeout: TimeoutSeconds) -> Result<()> {
    unimplemented!()
}

pub fn keyctl_assume_authority(key: Option<KeyringSerial>) -> Result<Option<KeyringSerial>> {
    unimplemented!()
}

pub fn keyctl_get_security(key: KeyringSerial, buffer: &[u8]) -> Result<usize> {
    unimplemented!()
}

pub fn keyctl_get_security_alloc(key: KeyringSerial) -> Result<CString> {
    unimplemented!()
}

pub fn keyctl_session_to_parent() -> Result<()> {
    unimplemented!()
}

// No fallback
pub fn keyctl_reject(
    key: KeyringSerial,
    timeout: TimeoutSeconds,
    error: i32,
    keyring: KeyringSerial,
) -> Result<()> {
    unimplemented!()
}

// No fallback
pub fn keyctl_instantiate_iov(
    key: KeyringSerial,
    payload: &[&[u8]],
    keyring: KeyringSerial,
) -> Result<()> {
    unimplemented!()
}

pub fn keyctl_invalidate(key: KeyringSerial) -> Result<()> {
    unimplemented!()
}

pub fn keyctl_get_persistent(uid: uid_t, keyring: KeyringSerial) -> Result<KeyringSerial> {
    unimplemented!()
}

// pub fn keyctl_dh_compute(dh: &DHParams, buffer: &[u8], kdf: Option<KDFParams>) -> Result<usize> {
//     unimplemented!()
// }

// pub fn keyctl_dh_compute_alloc(dh: &DHParams, kdf: Option<KDFParams>) -> Result<Vec<u8>> {
//     unimplemented!()
// }

pub fn keyctl_restrict_keyring(
    keyring: KeyringSerial,
    type_: Option<&CStr>,
    restriction: Option<&CStr>,
) -> Result<()> {
    unimplemented!()
}
