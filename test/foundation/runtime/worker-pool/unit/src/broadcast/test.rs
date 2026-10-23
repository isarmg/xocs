#![cfg(test)]

use crate::ThreadPoolBuilder;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::{thread, time};

#[cfg(test)]
include!("test/169.rs");

#[cfg(test)]
include!("test/317.rs");

#[cfg(test)]
include!("test/671.rs");

#[cfg(test)]
include!("test/936.rs");

#[cfg(test)]
include!("test/1332.rs");

#[cfg(test)]
include!("test/1616.rs");

#[cfg(test)]
include!("test/2029.rs");

#[cfg(test)]
include!("test/2543.rs");

#[cfg(test)]
include!("test/3154.rs");

#[cfg(test)]
include!("test/3850.rs");

#[cfg(test)]
include!("test/4696.rs");

#[cfg(test)]
include!("test/5235.rs");

#[cfg(test)]
include!("test/5836.rs");

#[cfg(test)]
include!("test/6380.rs");

#[cfg(test)]
include!("test/6986.rs");

#[cfg(test)]
include!("test/7589.rs");

#[cfg(test)]
include!("test/8085.rs");
