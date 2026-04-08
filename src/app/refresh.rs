//! Coordinator for background and synchronous tree refreshes.
//!
//! The app deals with three overlapping concerns when refreshing tree + git
//! state:
//!
//! 1. **Coalescing** — filesystem watcher bursts (e.g. `cargo build` touching
//!    dozens of files) should collapse into at most one in-flight refresh
//!    plus one coalesced catch-up.
//! 2. **Staleness** — if a synchronous refresh runs while a background task
//!    is in flight, the background task's result must be dropped when it
//!    eventually lands so it doesn't clobber the fresher sync snapshot.
//! 3. **Follow-up** — after a background result is applied, any events that
//!    arrived during the run should trigger exactly one more refresh.
//!
//! `RefreshCoordinator` captures all of this in a single three-field state
//! machine so callers don't have to touch the raw bits.

#[derive(Debug, Default)]
pub(crate) struct RefreshCoordinator {
    /// Monotonic counter bumped on every background spawn and every
    /// synchronous refresh. Results whose generation no longer matches
    /// must be discarded on arrival.
    generation: u64,
    /// `true` while a background task is running. Cleared by
    /// [`finish_background`](Self::finish_background) when the result
    /// arrives on the channel.
    in_flight: bool,
    /// `true` when a refresh was requested while `in_flight` was already set.
    /// Consumed by [`finish_background`](Self::finish_background) to decide
    /// whether to spawn one follow-up task.
    pending: bool,
}

impl RefreshCoordinator {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Called when the caller wants to spawn a background refresh.
    ///
    /// Returns `Some(generation)` if the caller should actually spawn the
    /// task (and use the returned generation when sending the result).
    /// Returns `None` if a task is already in flight; in that case the
    /// coordinator records a pending follow-up internally.
    pub(crate) fn try_start_background(&mut self) -> Option<u64> {
        if self.in_flight {
            self.pending = true;
            return None;
        }
        self.in_flight = true;
        self.generation = self.generation.wrapping_add(1);
        Some(self.generation)
    }

    /// Called at the start of a synchronous refresh.
    ///
    /// Bumps the generation so any in-flight background task becomes stale,
    /// and clears `pending` so a queued follow-up cannot overwrite the
    /// fresher sync snapshot. Leaves `in_flight` untouched; the event loop
    /// still clears it via [`finish_background`](Self::finish_background)
    /// when the stale result arrives.
    pub(crate) fn start_sync(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.pending = false;
    }

    /// Returns `true` if `generation` matches the current generation.
    /// Used by the event loop to discard stale background results.
    pub(crate) fn is_current(&self, generation: u64) -> bool {
        self.generation == generation
    }

    /// Called when a background task result arrives on the channel.
    ///
    /// Clears `in_flight` and returns `true` if the caller should spawn a
    /// coalesced follow-up refresh (i.e. `pending` was set during the run).
    pub(crate) fn finish_background(&mut self) -> bool {
        self.in_flight = false;
        if self.pending {
            self.pending = false;
            true
        } else {
            false
        }
    }

    /// Test-only accessor for the current generation.
    #[cfg(test)]
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    /// Test-only accessor for the in-flight flag.
    #[cfg(test)]
    pub(crate) fn in_flight(&self) -> bool {
        self.in_flight
    }

    /// Test-only accessor for the pending flag.
    #[cfg(test)]
    pub(crate) fn pending(&self) -> bool {
        self.pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_idle() {
        let r = RefreshCoordinator::new();
        assert_eq!(r.generation(), 0);
        assert!(!r.in_flight());
        assert!(!r.pending());
    }

    #[test]
    fn try_start_background_first_call_bumps_generation() {
        let mut r = RefreshCoordinator::new();
        let gen = r.try_start_background();
        assert_eq!(gen, Some(1));
        assert!(r.in_flight());
        assert!(!r.pending());
    }

    #[test]
    fn try_start_background_coalesces_while_in_flight() {
        let mut r = RefreshCoordinator::new();
        let first = r.try_start_background();
        assert_eq!(first, Some(1));

        // Second and third triggers only set `pending` — no new generation.
        assert_eq!(r.try_start_background(), None);
        assert!(r.pending());
        assert_eq!(r.generation(), 1);

        assert_eq!(r.try_start_background(), None);
        assert!(r.pending());
        assert_eq!(r.generation(), 1);
    }

    #[test]
    fn finish_background_clears_in_flight_and_returns_pending_state() {
        let mut r = RefreshCoordinator::new();
        let _ = r.try_start_background();
        assert!(!r.finish_background(), "no pending → no follow-up");
        assert!(!r.in_flight());
    }

    #[test]
    fn finish_background_returns_true_when_pending_then_clears_it() {
        let mut r = RefreshCoordinator::new();
        let _ = r.try_start_background();
        let _ = r.try_start_background(); // sets pending
        assert!(r.pending());

        let follow_up = r.finish_background();
        assert!(follow_up);
        assert!(!r.pending());
        assert!(!r.in_flight());
    }

    #[test]
    fn start_sync_bumps_generation_and_clears_pending() {
        let mut r = RefreshCoordinator::new();
        let _ = r.try_start_background();
        let _ = r.try_start_background(); // pending = true
        let before = r.generation();

        r.start_sync();

        assert!(!r.pending());
        assert_eq!(r.generation(), before.wrapping_add(1));
        // in_flight is deliberately left on — the stale background result
        // still arrives and the event loop clears it via finish_background.
        assert!(r.in_flight());
    }

    #[test]
    fn is_current_detects_stale_generation() {
        let mut r = RefreshCoordinator::new();
        let stale = r.try_start_background().unwrap();
        assert!(r.is_current(stale));

        r.start_sync();
        assert!(!r.is_current(stale));
        assert!(r.is_current(r.generation()));
    }

    #[test]
    fn follow_up_cycle_resets_cleanly() {
        // Simulate: start → coalesce → finish → follow-up start → finish.
        let mut r = RefreshCoordinator::new();
        let gen1 = r.try_start_background().unwrap();
        assert_eq!(gen1, 1);

        assert_eq!(r.try_start_background(), None);
        assert!(r.pending());

        assert!(r.finish_background()); // reports follow-up needed
        assert!(!r.in_flight());
        assert!(!r.pending());

        let gen2 = r.try_start_background().unwrap();
        assert_eq!(gen2, 2);

        assert!(!r.finish_background()); // no more pending
        assert!(!r.in_flight());
    }
}
