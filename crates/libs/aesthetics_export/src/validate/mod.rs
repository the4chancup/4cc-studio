//! The structure pass: the draft's issues, its scope/disposition rules, and
//! the sanitized export it produces. Only the issue types exist so far.

mod issues;

pub use issues::{Disposition, ISSUE_CODES, IssueScope, ValidationIssue};
