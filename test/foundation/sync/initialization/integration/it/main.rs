#[path = "unsync_once_cell.rs"]
mod unsync_once_cell;
#[path = "sync_once_cell.rs"]
#[cfg(any(feature = "std", feature = "critical-section"))]
mod sync_once_cell;

#[path = "unsync_lazy.rs"]
mod unsync_lazy;
#[path = "sync_lazy.rs"]
#[cfg(any(feature = "std", feature = "critical-section"))]
mod sync_lazy;

#[path = "race.rs"]
#[cfg(feature = "race")]
mod race;
#[path = "race_once_box.rs"]
#[cfg(all(feature = "race", feature = "alloc"))]
mod race_once_box;
