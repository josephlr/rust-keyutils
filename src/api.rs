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

use std::borrow::Borrow;
use std::convert::TryInto;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::result;
use std::str;
use std::time::Duration;

use keyutils_raw::*;
use log::error;

use crate::constants::{Permission, SpecialKeyring};
use crate::keytype::*;
use crate::keytypes;

/// Reexport of `Errno` as `Error`.
pub type Error = errno::Errno;
/// Simpler `Result` type with the error already set.
pub type Result<T> = result::Result<T, Error>;

fn check_call(res: libc::c_long) -> Result<()> {
    match res {
        -1 => Err(errno::errno()),
        _ => Ok(()),
    }
}

fn check_call_key(res: KeyringSerial) -> Result<Key> {
    check_call(res.get().into())?;
    Ok(Key::new_impl(res))
}

fn check_call_keyring(res: KeyringSerial) -> Result<Keyring> {
    check_call(res.get().into())?;
    Ok(Keyring::new_impl(res))
}

fn into_serial(res: libc::c_long) -> KeyringSerial {
    KeyringSerial::new(res as i32).unwrap()
}

/// Request a key from the kernel.
fn request_impl<K: KeyType>(
    description: &str,
    info: Option<&str>,
    id: Option<KeyringSerial>,
) -> KeyringSerial {
    let type_cstr = CString::new(K::name()).unwrap();
    let desc_cstr = CString::new(description).unwrap();
    let info_cstr = info.map(|i| CString::new(i).unwrap());

    let info_ptr = info_cstr.map_or(ptr::null(), |cs| cs.as_ptr());
    unsafe { request_key(type_cstr.as_ptr(), desc_cstr.as_ptr(), info_ptr, id) }
}

/// Representation of a kernel keyring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keyring {
    id: KeyringSerial,
}

impl Keyring {
    /// Instantiate a keyring from an ID.
    ///
    /// This is unsafe because no keyring is known to exist with the given ID.
    ///
    /// # Safety
    ///
    /// This method assumes that the given serial is a valid keyring ID at the kernel level.
    pub unsafe fn new(id: KeyringSerial) -> Self {
        Keyring {
            id,
        }
    }

    fn new_impl(id: KeyringSerial) -> Self {
        Keyring {
            id,
        }
    }

    #[cfg(test)]
    pub(crate) fn serial(&self) -> KeyringSerial {
        self.id
    }

    /// Set the default keyring to use when implicit requests on the current thread.
    ///
    /// Returns the old default keyring.
    ///
    /// # Panics
    ///
    /// If the kernel returns a keyring value which the library does not understand, the conversion
    /// from the return value into a `DefaultKeyring` will panic.
    pub fn set_default(keyring: DefaultKeyring) -> Result<DefaultKeyring> {
        let ret = unsafe { keyctl_set_reqkey_keyring(keyring as libc::c_int) };
        check_call(ret)?;
        Ok(ret.try_into().unwrap())
    }

    /// Requests a keyring with the given description by searching the thread, process, and session
    /// keyrings.
    ///
    /// If it is not found, the `info` string (if provided) will be handed off to
    /// `/sbin/request-key` to generate the key.
    ///
    /// If `target` is given, the found keyring will be linked into it. If `target` is not given
    /// and a new key is constructed due to the request, it will be linked into the default
    /// keyring (see `Keyring::set_default`).
    pub fn request<'s, 'a, D, I, T>(description: D, info: I, target: T) -> Result<Self>
    where
        D: AsRef<str>,
        I: Into<Option<&'s str>>,
        T: Into<Option<TargetKeyring<'a>>>,
    {
        check_call_keyring(request_impl::<keytypes::Keyring>(
            description.as_ref(),
            info.into().as_ref().copied(),
            target.into().map(TargetKeyring::serial),
        ))
    }

    fn get_keyring(id: SpecialKeyring, create: bool) -> Result<Keyring> {
        check_call_keyring(unsafe { keyctl_get_keyring_ID(id.serial(), create.into()) })
    }

    /// Attach to a special keyring. Fails if the keyring does not already exist.
    pub fn attach(id: SpecialKeyring) -> Result<Self> {
        Self::get_keyring(id, false)
    }

    /// Attach to a special keyring or create it if it does not exist.
    pub fn attach_or_create(id: SpecialKeyring) -> Result<Self> {
        Self::get_keyring(id, true)
    }

    /// Create a new anonymous keyring and set it as the session keyring.
    pub fn join_anonymous_session() -> Result<Self> {
        check_call_keyring(unsafe { keyctl_join_session_keyring(ptr::null()) })
    }

    /// Attached to a named session keyring.
    ///
    /// If a keyring named `name` exists, attach it as the session keyring (requires the `search`
    /// permission). If a keyring does not exist, create it and attach it as the session keyring.
    pub fn join_session<N>(name: N) -> Result<Self>
    where
        N: AsRef<str>,
    {
        let name_cstr = CString::new(name.as_ref()).unwrap();
        check_call_keyring(unsafe { keyctl_join_session_keyring(name_cstr.as_ptr()) })
    }

    /// Clears the contents of the keyring.
    ///
    /// Requires `write` permission on the keyring.
    pub fn clear(&mut self) -> Result<()> {
        check_call(unsafe { keyctl_clear(self.id) })
    }

    /// Adds a link to `key` to the keyring.
    ///
    /// Any link to an existing key with the same description is removed. Requires `write`
    /// permission on the keyring and `link` permission on the key.
    pub fn link_key(&mut self, key: &Key) -> Result<()> {
        check_call(unsafe { keyctl_link(key.id, self.id) })
    }

    /// Removes the link to `key` from the keyring.
    ///
    /// Requires `write` permission on the keyring.
    pub fn unlink_key(&mut self, key: &Key) -> Result<()> {
        check_call(unsafe { keyctl_unlink(key.id, self.id) })
    }

    /// Adds a link to `keyring` to the keyring.
    ///
    /// Any link to an existing keyring with the same description is removed. Requires `write`
    /// permission on the current keyring and `link` permission on the linked keyring.
    pub fn link_keyring(&mut self, keyring: &Keyring) -> Result<()> {
        check_call(unsafe { keyctl_link(keyring.id, self.id) })
    }

    /// Removes the link to `keyring` from the keyring.
    ///
    /// Requires `write` permission on the keyring.
    pub fn unlink_keyring(&mut self, keyring: &Keyring) -> Result<()> {
        check_call(unsafe { keyctl_unlink(keyring.id, self.id) })
    }

    fn search_impl<K>(&self, description: &str, destination: Option<&mut Keyring>) -> KeyringSerial
    where
        K: KeyType,
    {
        let type_cstr = CString::new(K::name()).unwrap();
        let desc_cstr = CString::new(description).unwrap();
        into_serial(unsafe {
            keyctl_search(
                self.id,
                type_cstr.as_ptr(),
                desc_cstr.as_ptr(),
                destination.map(|dest| dest.id),
            )
        })
    }

    /// Recursively search the keyring for a key with the matching description.
    ///
    /// If it is found, it is attached to the keyring (if `write` permission to the keyring and
    /// `link` permission on the key exist) and return it. Requires the `search` permission on the
    /// keyring. Any children keyrings without the `search` permission are ignored.
    pub fn search_for_key<'a, K, D, DK>(&self, description: D, destination: DK) -> Result<Key>
    where
        K: KeyType,
        D: Borrow<K::Description>,
        DK: Into<Option<&'a mut Keyring>>,
    {
        check_call_key(
            self.search_impl::<K>(&description.borrow().description(), destination.into()),
        )
    }

    /// Recursively search the keyring for a keyring with the matching description.
    ///
    /// If it is found, it is attached to the keyring (if `write` permission to the keyring and
    /// `link` permission on the found keyring exist) and return it. Requires the `search`
    /// permission on the keyring. Any children keyrings without the `search` permission are
    /// ignored.
    pub fn search_for_keyring<'a, D, DK>(&self, description: D, destination: DK) -> Result<Self>
    where
        D: Borrow<<keytypes::Keyring as KeyType>::Description>,
        DK: Into<Option<&'a mut Keyring>>,
    {
        check_call_keyring(self.search_impl::<keytypes::Keyring>(
            &description.borrow().description(),
            destination.into(),
        ))
    }

    /// Return all immediate children of the keyring.
    ///
    /// Requires `read` permission on the keyring.
    pub fn read(&self) -> Result<(Vec<Key>, Vec<Keyring>)> {
        // The `description` check below hides this error code from the kernel.
        if self.id.get() == 0 {
            return Err(errno::Errno(libc::ENOKEY));
        }

        // Avoid a panic in the code below be ensuring that we actually have a keyring. Parsing
        // a key's payload as a keyring payload.
        let desc = self.description()?;
        if desc.type_ != keytypes::Keyring::name() {
            return Err(errno::Errno(libc::ENOTDIR));
        }

        let sz = unsafe { keyctl_read(self.id, ptr::null_mut(), 0) };
        check_call(sz)?;
        let mut buffer = Vec::with_capacity((sz as usize) / mem::size_of::<KeyringSerial>());
        let actual_sz = unsafe {
            keyctl_read(
                self.id,
                buffer.as_mut_ptr() as *mut libc::c_char,
                sz as usize,
            )
        };
        check_call(actual_sz)?;
        unsafe { buffer.set_len((actual_sz as usize) / mem::size_of::<KeyringSerial>()) };

        let mut keys = Vec::new();
        let mut keyrings = Vec::new();
        for key in buffer.into_iter().map(Key::new_impl) {
            match key.description() {
                Ok(description) => {
                    if description.type_ == keytypes::Keyring::name() {
                        keyrings.push(Keyring::new_impl(key.id))
                    } else {
                        keys.push(key)
                    }
                },
                // Keys can be invalidated between reading the keyring and
                // reading the child key's description. If this happens, we get
                // ENOKEY and just skip that key.
                Err(errno::Errno(libc::ENOKEY)) => {},
                Err(e) => return Err(e),
            }
        }
        Ok((keys, keyrings))
    }

    /// Attach the persistent keyring for the current user to the current keyring.
    ///
    /// If one does not exist, it will be created. Requires `write` permission on the keyring.
    pub fn attach_persistent(&mut self) -> Result<Self> {
        check_call_keyring(into_serial(unsafe { keyctl_get_persistent(!0, self.id) }))
    }

    /// Adds a key of a specific type to the keyring.
    ///
    /// If a key with the same description already exists and has the `update` permission, it will
    /// be updated, otherwise the link to the old key will be removed. Requires `write` permission.
    pub fn add_key<K, D, P>(&mut self, description: D, payload: P) -> Result<Key>
    where
        K: KeyType,
        D: Borrow<K::Description>,
        P: Borrow<K::Payload>,
    {
        check_call_key(self.add_key_impl::<K>(description.borrow(), payload.borrow()))
    }

    /// Monomorphization of adding a key.
    fn add_key_impl<K>(
        &mut self,
        description: &K::Description,
        payload: &K::Payload,
    ) -> KeyringSerial
    where
        K: KeyType,
    {
        let type_cstr = CString::new(K::name()).unwrap();
        let desc_cstr = CString::new(description.description().as_bytes()).unwrap();
        let payload = payload.payload();
        unsafe {
            add_key(
                type_cstr.as_ptr(),
                desc_cstr.as_ptr(),
                payload.as_ptr() as *const libc::c_void,
                payload.len(),
                self.id,
            )
        }
    }

    /// Adds a keyring to the current keyring.
    ///
    /// If a keyring with the same description already, the link to the old keyring will be
    /// removed. Requires `write` permission on the keyring.
    pub fn add_keyring<D>(&mut self, description: D) -> Result<Self>
    where
        D: Borrow<<keytypes::Keyring as KeyType>::Description>,
    {
        check_call_keyring(self.add_key_impl::<keytypes::Keyring>(description.borrow(), &()))
    }

    /// Revokes the keyring.
    ///
    /// Requires `write` permission on the keyring.
    pub fn revoke(self) -> Result<()> {
        check_call(unsafe { keyctl_revoke(self.id) })
    }

    /// Change the user which owns the keyring.
    ///
    /// Requires the `setattr` permission on the keyring and the SysAdmin capability to change it
    /// to anything other than the current user.
    pub fn chown(&mut self, uid: libc::uid_t) -> Result<()> {
        check_call(unsafe { keyctl_chown(self.id, uid, !0) })
    }

    /// Change the group which owns the keyring.
    ///
    /// Requires the `setattr` permission on the keyring and the SysAdmin capability to change it
    /// to anything other than a group of which the current user is a member.
    pub fn chgrp(&mut self, gid: libc::gid_t) -> Result<()> {
        check_call(unsafe { keyctl_chown(self.id, !0, gid) })
    }

    /// Set the permissions on the keyring.
    ///
    /// Requires the `setattr` permission on the keyring and the SysAdmin capability if the current
    /// user does not own the keyring.
    pub fn set_permissions(&mut self, perms: Permission) -> Result<()> {
        check_call(unsafe { keyctl_setperm(self.id, perms.bits()) })
    }

    #[cfg(test)]
    pub(crate) fn set_permissions_raw(&mut self, perms: KeyPermissions) -> Result<()> {
        check_call(unsafe { keyctl_setperm(self.id, perms) })
    }

    fn description_raw(&self) -> Result<String> {
        let sz = unsafe { keyctl_describe(self.id, ptr::null_mut(), 0) };
        check_call(sz)?;
        let mut buffer = Vec::with_capacity(sz as usize);
        let actual_sz = unsafe {
            keyctl_describe(
                self.id,
                buffer.as_mut_ptr() as *mut libc::c_char,
                sz as usize,
            )
        };
        check_call(actual_sz)?;
        unsafe { buffer.set_len((actual_sz - 1) as usize) };
        let str_slice = str::from_utf8(&buffer[..]).unwrap();
        Ok(str_slice.to_owned())
    }

    /// Retrieve metadata about the keyring.
    ///
    /// # Panics
    ///
    /// If the kernel returns malformed data, the the parser will panic.
    pub fn description(&self) -> Result<Description> {
        self.description_raw()
            .and_then(|desc| Description::parse(&desc).ok_or(errno::Errno(libc::EINVAL)))
    }

    /// Set an expiration timer on the keyring to `timeout`.
    ///
    /// Any partial seconds are ignored. A timeout of 0 means "no expiration". Requires the
    /// `setattr` permission on the keyring.
    pub fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
        check_call(unsafe { keyctl_set_timeout(self.id, timeout.as_secs() as TimeoutSeconds) })
    }

    /// The security context of the keyring. Depends on the security manager loaded into the kernel
    /// (e.g., SELinux or AppArmor).
    pub fn security(&self) -> Result<String> {
        let sz = unsafe { keyctl_get_security(self.id, ptr::null_mut(), 0) };
        check_call(sz)?;
        let mut buffer = Vec::with_capacity(sz as usize);
        let actual_sz = unsafe {
            keyctl_get_security(
                self.id,
                buffer.as_mut_ptr() as *mut libc::c_char,
                sz as usize,
            )
        };
        check_call(actual_sz)?;
        unsafe { buffer.set_len(actual_sz as usize) };
        let str_slice = str::from_utf8(&buffer[..]).unwrap();
        Ok(str_slice.to_owned())
    }

    /// Invalidates the keyring and schedules it for removal. Requires the `search` permission on
    /// the keyring.
    pub fn invalidate(self) -> Result<()> {
        check_call(unsafe { keyctl_invalidate(self.id) })
    }
}

/// Representation of a kernel key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    id: KeyringSerial,
}

impl Key {
    /// Instantiate a key from an ID.
    ///
    /// This is unsafe because no key is known to exist with the given ID.
    ///
    /// # Safety
    ///
    /// This method assumes that the given serial is a valid key ID at the kernel level.
    pub unsafe fn new(id: KeyringSerial) -> Self {
        Self::new_impl(id)
    }

    fn new_impl(id: KeyringSerial) -> Self {
        Key {
            id,
        }
    }

    #[cfg(test)]
    pub(crate) fn serial(&self) -> KeyringSerial {
        self.id
    }

    /// Requests a key with the given type and description by searching the thread, process, and
    /// session keyrings.
    ///
    /// If it is not found, the `info` string (if provided) will be handed off to
    /// `/sbin/request-key` to generate the key.
    ///
    /// If `target` is given, the found keyring will be linked into it. If `target` is not given
    /// and a new key is constructed due to the request, it will be linked into the default
    /// keyring (see `Keyring::set_default`).
    pub fn request<'s, 'a, K, D, I, T>(description: D, info: I, target: T) -> Result<Self>
    where
        K: KeyType,
        D: Borrow<K::Description>,
        I: Into<Option<&'s str>>,
        T: Into<Option<TargetKeyring<'a>>>,
    {
        check_call_key(request_impl::<K>(
            &description.borrow().description(),
            info.into().as_ref().copied(),
            target.into().map(TargetKeyring::serial),
        ))
    }

    /// Update the payload in the key.
    pub fn update<D>(&mut self, data: D) -> Result<()>
    where
        D: AsRef<[u8]>,
    {
        let data = data.as_ref();
        check_call(unsafe {
            keyctl_update(self.id, data.as_ptr() as *const libc::c_void, data.len())
        })
    }

    /// Revokes the key. Requires `write` permission on the key.
    pub fn revoke(self) -> Result<()> {
        Keyring::new_impl(self.id).revoke()
    }

    /// Change the user which owns the key.
    ///
    /// Requires the `setattr` permission on the key and the SysAdmin capability to change it to
    /// anything other than the current user.
    pub fn chown(&mut self, uid: libc::uid_t) -> Result<()> {
        Keyring::new_impl(self.id).chown(uid)
    }

    /// Change the group which owns the key.
    ///
    /// Requires the `setattr` permission on the key and the SysAdmin capability to change it to
    /// anything other than a group of which the current user is a member.
    pub fn chgrp(&mut self, gid: libc::gid_t) -> Result<()> {
        Keyring::new_impl(self.id).chgrp(gid)
    }

    /// Set the permissions on the key.
    ///
    /// Requires the `setattr` permission on the key and the SysAdmin capability if the current
    /// user does not own the key.
    pub fn set_permissions(&mut self, perms: Permission) -> Result<()> {
        Keyring::new_impl(self.id).set_permissions(perms)
    }

    #[cfg(test)]
    pub(crate) fn set_permissions_raw(&mut self, perms: KeyPermissions) -> Result<()> {
        Keyring::new_impl(self.id).set_permissions_raw(perms)
    }

    /// Retrieve metadata about the key.
    ///
    /// # Panics
    ///
    /// If the kernel returns malformed data, the parser will panic.
    pub fn description(&self) -> Result<Description> {
        Keyring::new_impl(self.id).description()
    }

    /// Read the payload of the key. Requires `read` permissions on the key.
    pub fn read(&self) -> Result<Vec<u8>> {
        let sz = unsafe { keyctl_read(self.id, ptr::null_mut(), 0) };
        check_call(sz)?;
        let mut buffer = Vec::with_capacity(sz as usize);
        let actual_sz = unsafe {
            keyctl_read(
                self.id,
                buffer.as_mut_ptr() as *mut libc::c_char,
                sz as usize,
            )
        };
        check_call(actual_sz)?;
        unsafe { buffer.set_len(actual_sz as usize) };
        Ok(buffer)
    }

    /// Set an expiration timer on the keyring to `timeout`.
    ///
    /// Any partial seconds are ignored. A timeout of 0 means "no expiration". Requires the
    /// `setattr` permission on the key.
    pub fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
        Keyring::new_impl(self.id).set_timeout(timeout)
    }

    /// The security context of the key.
    ///
    /// Depends on the security manager loaded into the kernel (e.g., SELinux or AppArmor).
    pub fn security(&self) -> Result<String> {
        Keyring::new_impl(self.id).security()
    }

    /// Invalidates the key and schedules it for removal.
    ///
    /// Requires the `search` permission on the key.
    pub fn invalidate(self) -> Result<()> {
        Keyring::new_impl(self.id).invalidate()
    }

    /// Create an object to manage a key request.
    ///
    /// Before a key may be managed on a thread, an authorization key must be attached to an
    /// available thread keyring.
    ///
    /// Only one key may be managed on a thread at a time. Managing a second key will
    /// invalidate any previous `KeyManager` constructions.
    ///
    /// See `KeyManager::request_key_auth_key`.
    pub fn manage(&mut self) -> Result<KeyManager> {
        check_call(unsafe { keyctl_assume_authority(Some(self.id)) })?;
        Ok(KeyManager::new(Key::new_impl(self.id)))
    }

    /// Compute a Diffie-Hellman prime for use as a shared secret or public key.
    pub fn compute_dh(private: &Key, prime: &Key, base: &Key) -> Result<Vec<u8>> {
        let sz = unsafe {
            keyctl_dh_compute(
                private.id,
                prime.id,
                base.id,
                ptr::null_mut() as *mut libc::c_char,
                0,
            )
        };
        check_call(sz)?;
        let mut buffer = Vec::with_capacity(sz as usize);
        let actual_sz = unsafe {
            keyctl_dh_compute(
                private.id,
                prime.id,
                base.id,
                buffer.as_mut_ptr() as *mut libc::c_char,
                sz as usize,
            )
        };
        check_call(actual_sz)?;
        unsafe { buffer.set_len(actual_sz as usize) };
        Ok(buffer)
    }
}

/// Structure representing the metadata about a key or keyring.
#[derive(Debug, Clone)]
pub struct Description {
    /// The type of the key.
    pub type_: String,
    /// The user owner of the key.
    pub uid: libc::uid_t,
    /// The group owner of the key.
    pub gid: libc::gid_t,
    /// The permissions of the key.
    pub perms: Permission,
    /// The plaintext description of the key.
    pub description: String,
}

impl Description {
    fn parse(desc: &str) -> Option<Description> {
        let mut pieces = desc.split(';').collect::<Vec<_>>();
        // Reverse the string because the kernel plans to extend it by adding fields to the
        // beginning of the string. By doing this, the fields are at a constant position in the
        // split string.
        pieces.reverse();
        let len = pieces.len();
        if len < 5 {
            None
        } else {
            if len > 5 {
                error!(
                    "New fields detected! Please report this upstream to \
                     https://github.com/mathstuf/rust-keyutils: {}",
                    desc,
                );
            }
            let bits = KeyPermissions::from_str_radix(pieces[1], 16).unwrap();
            if Permission::from_bits(bits).is_none() {
                error!(
                    "New permission bits detected! Please report this upstream to \
                     https://github.com/mathstuf/rust-keyutils: {}",
                    bits,
                );
            }
            Some(Description {
                type_: pieces[4].to_owned(),
                uid: pieces[3].parse::<libc::uid_t>().unwrap(),
                gid: pieces[2].parse::<libc::gid_t>().unwrap(),
                perms: Permission::from_bits_truncate(bits),
                description: pieces[0].to_owned(),
            })
        }
    }
}

/// The destination keyring of an instantiation request.
#[derive(Debug)]
pub enum TargetKeyring<'a> {
    /// A special keyring.
    Special(SpecialKeyring),
    /// A specific keyring.
    Keyring(&'a mut Keyring),
}

impl<'a> TargetKeyring<'a> {
    fn serial(self) -> KeyringSerial {
        match self {
            TargetKeyring::Special(special) => special.serial(),
            TargetKeyring::Keyring(keyring) => keyring.id,
        }
    }
}

impl<'a> From<SpecialKeyring> for TargetKeyring<'a> {
    fn from(special: SpecialKeyring) -> Self {
        TargetKeyring::Special(special)
    }
}

impl<'a> From<&'a mut Keyring> for TargetKeyring<'a> {
    fn from(keyring: &'a mut Keyring) -> Self {
        TargetKeyring::Keyring(keyring)
    }
}

impl<'a> From<SpecialKeyring> for Option<TargetKeyring<'a>> {
    fn from(special: SpecialKeyring) -> Self {
        Some(special.into())
    }
}

impl<'a> From<&'a mut Keyring> for Option<TargetKeyring<'a>> {
    fn from(keyring: &'a mut Keyring) -> Self {
        Some(keyring.into())
    }
}

/// A manager for a key to respond to instantiate a key request by the kernel.
#[derive(Debug, PartialEq, Eq)]
pub struct KeyManager {
    key: Key,
}

impl KeyManager {
    fn new(key: Key) -> Self {
        KeyManager {
            key,
        }
    }

    #[cfg(test)]
    pub(crate) fn test_new(key: Key) -> Self {
        Self::new(key)
    }

    /// Requests the authorization key created by `request_key`.
    ///
    /// This key must be present in an available keyring before `Key::manage` may be called.
    pub fn request_key_auth_key(create: bool) -> Result<Key> {
        check_call_key(unsafe { keyctl_get_keyring_ID(KEY_SPEC_REQKEY_AUTH_KEY, create.into()) })
    }

    /// Drop authority for the current thread.
    ///
    /// This invalidates
    pub fn drop_authority() -> Result<()> {
        check_call(unsafe { keyctl_assume_authority(None) })
    }

    /// Instantiate the key with the given payload.
    pub fn instantiate<'a, T, P>(self, keyring: T, payload: P) -> Result<()>
    where
        T: Into<Option<TargetKeyring<'a>>>,
        P: AsRef<[u8]>,
    {
        let payload = payload.as_ref();
        check_call(unsafe {
            keyctl_instantiate(
                self.key.id,
                payload.as_ptr() as *const libc::c_void,
                payload.len(),
                keyring.into().map(TargetKeyring::serial),
            )
        })
    }

    /// Reject the key with the given `error`.
    ///
    /// Requests for the key will fail until `timeout` has elapsed (partial
    /// seconds are ignored). This is to prevent a denial-of-service by
    /// requesting a non-existant key repeatedly. The requester must have
    /// `write` permission on the keyring.
    pub fn reject<'a, T>(self, keyring: T, timeout: Duration, error: errno::Errno) -> Result<()>
    where
        T: Into<Option<TargetKeyring<'a>>>,
    {
        let errno::Errno(errval) = error;
        check_call(unsafe {
            keyctl_reject(
                self.key.id,
                timeout.as_secs() as TimeoutSeconds,
                errval as u32,
                keyring.into().map(TargetKeyring::serial),
            )
        })
    }

    /// Reject the key with `ENOKEY`.
    ///
    /// Requests for the key will fail until `timeout` has elapsed (partial
    /// seconds are ignored). This is to prevent a denial-of-service by
    /// requesting a non-existant key repeatedly. The requester must have
    /// `write` permission on the keyring.
    pub fn negate<'a, T>(self, keyring: T, timeout: Duration) -> Result<()>
    where
        T: Into<Option<TargetKeyring<'a>>>,
    {
        check_call(unsafe {
            keyctl_negate(
                self.key.id,
                timeout.as_secs() as TimeoutSeconds,
                keyring.into().map(TargetKeyring::serial),
            )
        })
    }
}
