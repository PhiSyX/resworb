/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::fmt;

// ----------- //
// Énumération //
// ----------- //

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
#[repr(u8)]
pub enum DOMException {
    #[deprecated = "Use `RangeError` instead."]
    IndexSizeError = 1,

    HierarchyRequestError = 3,
    WrongDocumentError = 4,
    InvalidCharacterError = 5,
    NoModificationAllowedError = 7,
    NotFoundError = 8,
    NotSupportedError = 9,
    InUseAttributeError = 10,
    InvalidStateError = 11,
    SyntaxError = 12,
    InvalidModificationError = 13,
    NamespaceError = 14,

    #[deprecated = "\
        Use `TypeError` for invalid arguments, `NotSupportedError` \
        DOMException for unsupported operations, and `NotAllowedError` \
        DOMException for denied requests instead.\
    "]
    InvalidAccessError = 15,

    #[deprecated = "Use `TypeError` instead."]
    TypeMismatchError = 17,

    SecurityError = 18,
    NetworkError = 19,
    AbortError = 20,
    URLMismatchError = 21,
    QuotaExceededError = 22,
    TimeoutError = 23,
    InvalidNodeTypeError = 24,
    DataCloneError = 25,
    EncodingError,
    NotReadableError,
    UnknownError,
    ConstraintError,
    DataError,
    TransactionInactiveError,
    ReadOnlyError,
    VersionError,
    OperationError,
    NotAllowedError,
}

// -------------- //
// Implémentation // -> Interface
// -------------- //

impl fmt::Display for DOMException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                #[allow(deprecated)]
                | DOMException::IndexSizeError => "",
                | DOMException::HierarchyRequestError =>
                    "The operation would yield an incorrect node tree.",
                | DOMException::WrongDocumentError =>
                    "The object is in the wrong document.",
                | DOMException::InvalidCharacterError =>
                    "The string contains invalid characters.",
                | DOMException::NoModificationAllowedError =>
                    "The object can not be modified.",
                | DOMException::NotFoundError =>
                    "The object can not be found here.",
                | DOMException::NotSupportedError =>
                    "The operation is not supported.",
                | DOMException::InUseAttributeError =>
                    "The attribute is in use.",
                | DOMException::InvalidStateError =>
                    "The object is in an invalid state.",
                | DOMException::SyntaxError =>
                    "The string did not match the expected pattern.",
                | DOMException::InvalidModificationError =>
                    "The object can not be modified in this way.",
                | DOMException::NamespaceError =>
                    "The operation is not allowed by Namespaces in XML.",
                #[allow(deprecated)]
                | DOMException::InvalidAccessError => "",
                #[allow(deprecated)]
                | DOMException::TypeMismatchError => "",
                | DOMException::SecurityError =>
                    "The operation is insecure.",
                | DOMException::NetworkError =>
                    "A network error occurred.",
                | DOMException::AbortError => "The operation was aborted.",
                | DOMException::URLMismatchError =>
                    "The given URL does not match another URL.",
                | DOMException::QuotaExceededError =>
                    "The quota has been exceeded.",
                | DOMException::TimeoutError => "The operation timed out.",
                | DOMException::InvalidNodeTypeError =>
                    "The supplied node is incorrect or has an incorrect \
                     ancestor for this operation.",
                | DOMException::DataCloneError =>
                    "The object can not be cloned.",
                | DOMException::EncodingError =>
                    "The encoding operation (either encoded or decoding) \
                     failed.",
                | DOMException::NotReadableError =>
                    "The I/O read operation failed.",
                | DOMException::UnknownError =>
                    "The operation failed for an unknown transient reason \
                     (e.g. out of memory).",
                | DOMException::ConstraintError =>
                    "A mutation operation in a transaction failed because a \
                     constraint was not satisfied.",
                | DOMException::DataError => "Provided data is inadequate.",
                | DOMException::TransactionInactiveError =>
                    "A request was placed against a transaction which is \
                     currently not active, or which is finished.",
                | DOMException::ReadOnlyError =>
                    "The mutating operation was attempted in a `readonly` \
                     transaction.",
                | DOMException::VersionError =>
                    "An attempt was made to open a database using a lower \
                     version than the existing version.",
                | DOMException::OperationError =>
                    "The operation failed for an operation-specific reason.",
                | DOMException::NotAllowedError =>
                    "The request is not allowed by the user agent or the \
                     platform in the current context, possibly because the \
                     user denied permission.",
            }
        )
    }
}
