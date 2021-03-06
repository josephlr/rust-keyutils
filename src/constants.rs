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

use bitflags::bitflags;
use keyutils_raw::*;

/// Special keyrings predefined for a process.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SpecialKeyring {
    /// A thread-specific keyring.
    Thread,
    /// A process-specific keyring.
    Process,
    /// A session-specific keyring.
    Session,
    /// A user-specific keyring.
    User,
    /// A user session-specific keyring.
    UserSession,
    /// A group-specific keyring.
    Group,
}

impl SpecialKeyring {
    /// Retrieve the serial number for the special keyring.
    pub fn serial(self) -> KeyringSerial {
        match self {
            SpecialKeyring::Thread => KEY_SPEC_THREAD_KEYRING,
            SpecialKeyring::Process => KEY_SPEC_PROCESS_KEYRING,
            SpecialKeyring::Session => KEY_SPEC_SESSION_KEYRING,
            SpecialKeyring::User => KEY_SPEC_USER_KEYRING,
            SpecialKeyring::UserSession => KEY_SPEC_USER_SESSION_KEYRING,
            SpecialKeyring::Group => KEY_SPEC_GROUP_KEYRING,
        }
    }
}

bitflags! {
    /// Permission bits for keyring objects.
    ///
    /// Keyrings and keys contain four sets of permissions. First, there are three permission sets
    /// used is based on which of the owning user's ID, the group ID associated with the key or
    /// keyring, and a third set which is used when neither of the other two match.
    ///
    /// The fourth set is combined with the permission set used above (priority to user, then
    /// group, finally other) where either set granting a permission allows it. This set is,
    /// however, only used if the caller is a "possessor" of they key or keyring. Generally,
    /// "possession" requires the `search` permission, association from the calling thread
    /// (the session, process, and thread keyrings), or is linked to from a possessed keyring. See
    /// `keyrings(7)` for complete details.
    pub struct Permission: KeyPermissions {
        /// Possession allows viewing attributes about the key or keyring.
        const POSSESSOR_VIEW            = KEY_POS_VIEW;
        /// Possession allows reading a key's contents or a keyring's subkeys.
        const POSSESSOR_READ            = KEY_POS_READ;
        /// Possession allows writing a key's content, revoking a key, or adding and removing a
        /// keyring's links.
        const POSSESSOR_WRITE           = KEY_POS_WRITE;
        /// Possession allows searching within a keyring and the key or keyring may be discovered
        /// during a search.
        const POSSESSOR_SEARCH          = KEY_POS_SEARCH;
        /// Possession allows linking to the key from a keyring.
        const POSSESSOR_LINK            = KEY_POS_LINK;
        /// Possession allows changing ownership details, security labels, and the expiration
        /// time of a key.
        const POSSESSOR_SET_ATTRIBUTE   = KEY_POS_SETATTR;
        /// Possession grants all permissions.
        const POSSESSOR_ALL             = KEY_POS_ALL;

        /// A user ID match allows viewing attributes about the key or keyring.
        const USER_VIEW             = KEY_USR_VIEW;
        /// A user ID match allows reading a key's contents or a keyring's subkeys.
        const USER_READ             = KEY_USR_READ;
        /// A user ID match allows writing a key's content, revoking a key, or adding and removing
        /// a keyring's links.
        const USER_WRITE            = KEY_USR_WRITE;
        /// A user ID match allows searching within a keyring and the key or keyring may be
        /// discovered during a search.
        const USER_SEARCH           = KEY_USR_SEARCH;
        /// A user ID match allows linking to the key from a keyring.
        const USER_LINK             = KEY_USR_LINK;
        /// A user ID match allows changing ownership details, security labels, and the expiration
        /// time of a key.
        const USER_SET_ATTRIBUTE    = KEY_USR_SETATTR;
        /// The user is granted all permissions.
        const USER_ALL              = KEY_USR_ALL;

        /// A group ID match allows viewing attributes about the key or keyring.
        const GROUP_VIEW            = KEY_GRP_VIEW;
        /// A group ID match allows reading a key's contents or a keyring's subkeys.
        const GROUP_READ            = KEY_GRP_READ;
        /// A group ID match allows writing a key's content, revoking a key, or adding and removing
        /// a keyring's links.
        const GROUP_WRITE           = KEY_GRP_WRITE;
        /// A group ID match allows searching within a keyring and the key or keyring may be
        /// discovered during a search.
        const GROUP_SEARCH          = KEY_GRP_SEARCH;
        /// A group ID match allows linking to the key from a keyring.
        const GROUP_LINK            = KEY_GRP_LINK;
        /// A group ID match allows changing ownership details, security labels, and the expiration
        /// time of a key.
        const GROUP_SET_ATTRIBUTE   = KEY_GRP_SETATTR;
        /// The group is granted all permissions.
        const GROUP_ALL             = KEY_GRP_ALL;

        /// Allows viewing attributes about the key or keyring.
        const OTHER_VIEW            = KEY_OTH_VIEW;
        /// Allows reading a key's contents or a keyring's subkeys.
        const OTHER_READ            = KEY_OTH_READ;
        /// Allows writing a key's content, revoking a key, or adding and removing a keyring's
        /// links.
        const OTHER_WRITE           = KEY_OTH_WRITE;
        /// Allows searching within a keyring and the key or keyring may be discovered during a
        /// search.
        const OTHER_SEARCH          = KEY_OTH_SEARCH;
        /// Allows linking to the key from a keyring.
        const OTHER_LINK            = KEY_OTH_LINK;
        /// Allows changing ownership details, security labels, and the expiration time of a key.
        const OTHER_SET_ATTRIBUTE   = KEY_OTH_SETATTR;
        /// All permissions.
        const OTHER_ALL             = KEY_OTH_ALL;
    }
}

#[test]
fn test_keyring_ids() {
    assert_eq!(SpecialKeyring::Thread.serial(), KEY_SPEC_THREAD_KEYRING);
    assert_eq!(SpecialKeyring::Process.serial(), KEY_SPEC_PROCESS_KEYRING);
    assert_eq!(SpecialKeyring::Session.serial(), KEY_SPEC_SESSION_KEYRING);
    assert_eq!(SpecialKeyring::User.serial(), KEY_SPEC_USER_KEYRING);
    assert_eq!(
        SpecialKeyring::UserSession.serial(),
        KEY_SPEC_USER_SESSION_KEYRING
    );
    assert_eq!(SpecialKeyring::Group.serial(), KEY_SPEC_GROUP_KEYRING);
}

#[test]
fn test_permission_bits() {
    assert_eq!(Permission::POSSESSOR_VIEW.bits, KEY_POS_VIEW);
    assert_eq!(Permission::POSSESSOR_READ.bits, KEY_POS_READ);
    assert_eq!(Permission::POSSESSOR_WRITE.bits, KEY_POS_WRITE);
    assert_eq!(Permission::POSSESSOR_SEARCH.bits, KEY_POS_SEARCH);
    assert_eq!(Permission::POSSESSOR_LINK.bits, KEY_POS_LINK);
    assert_eq!(Permission::POSSESSOR_SET_ATTRIBUTE.bits, KEY_POS_SETATTR);
    assert_eq!(Permission::POSSESSOR_ALL.bits, KEY_POS_ALL);

    assert_eq!(Permission::USER_VIEW.bits, KEY_USR_VIEW);
    assert_eq!(Permission::USER_READ.bits, KEY_USR_READ);
    assert_eq!(Permission::USER_WRITE.bits, KEY_USR_WRITE);
    assert_eq!(Permission::USER_SEARCH.bits, KEY_USR_SEARCH);
    assert_eq!(Permission::USER_LINK.bits, KEY_USR_LINK);
    assert_eq!(Permission::USER_SET_ATTRIBUTE.bits, KEY_USR_SETATTR);
    assert_eq!(Permission::USER_ALL.bits, KEY_USR_ALL);

    assert_eq!(Permission::GROUP_VIEW.bits, KEY_GRP_VIEW);
    assert_eq!(Permission::GROUP_READ.bits, KEY_GRP_READ);
    assert_eq!(Permission::GROUP_WRITE.bits, KEY_GRP_WRITE);
    assert_eq!(Permission::GROUP_SEARCH.bits, KEY_GRP_SEARCH);
    assert_eq!(Permission::GROUP_LINK.bits, KEY_GRP_LINK);
    assert_eq!(Permission::GROUP_SET_ATTRIBUTE.bits, KEY_GRP_SETATTR);
    assert_eq!(Permission::GROUP_ALL.bits, KEY_GRP_ALL);

    assert_eq!(Permission::OTHER_VIEW.bits, KEY_OTH_VIEW);
    assert_eq!(Permission::OTHER_READ.bits, KEY_OTH_READ);
    assert_eq!(Permission::OTHER_WRITE.bits, KEY_OTH_WRITE);
    assert_eq!(Permission::OTHER_SEARCH.bits, KEY_OTH_SEARCH);
    assert_eq!(Permission::OTHER_LINK.bits, KEY_OTH_LINK);
    assert_eq!(Permission::OTHER_SET_ATTRIBUTE.bits, KEY_OTH_SETATTR);
    assert_eq!(Permission::OTHER_ALL.bits, KEY_OTH_ALL);
}
