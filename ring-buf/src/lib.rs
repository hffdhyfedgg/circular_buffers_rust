#![no_std]
#![deny(unsafe_code)]

//! `ring-buf` crate: ring buffers, stacks, tails, filter banks, views, and mathematical operations.

pub mod cbuf;
pub mod cbuf2n;
pub mod error;
pub mod filter;
pub mod iter;
pub mod math;
pub mod ops;
pub mod stack;
pub mod stack2n;
pub mod stack_tail;
pub mod stack_tail2n;
pub mod tail;
pub mod tail2n;
pub mod view;
pub mod view2n;

pub use cbuf::CBuf;
pub use cbuf2n::CBuf2N;
pub use error::{Result, RingBufError};
pub use filter::FilterBank;
pub use iter::{CBufIter, CBufIterMut};
pub use stack::CBufStack;
pub use stack2n::CBuf2NStack;
pub use stack_tail::CBufStackTail;
pub use stack_tail2n::CBuf2NStackTail;
pub use tail::CBufTail;
pub use tail2n::CBuf2NTail;
pub use view::{CBufView, CBufViewMut};
pub use view2n::{CBuf2NView, CBuf2NViewMut};
