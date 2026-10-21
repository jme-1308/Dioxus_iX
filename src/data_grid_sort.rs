//! Controlled sort priorities for the independent Rust grid.
use crate::{DataGridSortDirection, DataGridState};
use std::collections::BTreeSet;

impl DataGridState {
    /// Effective primary-to-secondary order. The primary `sort` is authoritative:
    /// without it, the tail is ignored. Duplicate IDs are removed, first wins.
    /// Unknown IDs retain the projection's legacy case-insensitive text semantics.
    pub fn sort_chain(&self) -> Vec<(&str, DataGridSortDirection)> {
        let Some((column, direction)) = &self.sort else {
            return Vec::new();
        };
        let mut seen = BTreeSet::new();
        std::iter::once((column.as_str(), *direction))
            .chain(
                self.secondary_sorts
                    .iter()
                    .map(|(id, direction)| (id.as_str(), *direction)),
            )
            .filter(|(id, _)| seen.insert(*id))
            .collect()
    }
    /// One-based effective priority and direction for a column.
    pub fn sort_for_column(&self, column: &str) -> Option<(usize, DataGridSortDirection)> {
        self.sort_chain()
            .into_iter()
            .enumerate()
            .find_map(|(index, (id, direction))| (id == column).then_some((index + 1, direction)))
    }
    /// Remove every priority and return to page zero, preserving other state.
    pub fn clear_sorts(&mut self) {
        self.sort = None;
        self.secondary_sorts.clear();
        self.reset_pagination_page();
    }
    /// Ordinary activation makes this the sole sort. Repeated activation cycles
    /// the current primary ascending, descending, then off.
    pub fn cycle_sort(&mut self, column: &str) {
        self.cycle_sort_with_append(column, false);
    }
    /// Additive activation appends an unsorted column, or cycles its current
    /// direction at the same priority. Removing the primary promotes its tail.
    /// Every request returns to page zero without changing filters or selection.
    pub fn cycle_sort_with_append(&mut self, column: &str, additive: bool) {
        self.reset_pagination_page();
        if !additive {
            self.secondary_sorts.clear();
            self.sort = match &self.sort {
                Some((current, DataGridSortDirection::Ascending)) if current == column => {
                    Some((column.into(), DataGridSortDirection::Descending))
                }
                Some((current, DataGridSortDirection::Descending)) if current == column => None,
                _ => Some((column.into(), DataGridSortDirection::Ascending)),
            };
            return;
        }
        let mut chain: Vec<_> = self
            .sort_chain()
            .into_iter()
            .map(|(id, direction)| (id.to_string(), direction))
            .collect();
        match chain.iter().position(|(id, _)| id == column) {
            Some(index) if chain[index].1 == DataGridSortDirection::Ascending => {
                chain[index].1 = DataGridSortDirection::Descending
            }
            Some(index) => {
                chain.remove(index);
            }
            None => chain.push((column.into(), DataGridSortDirection::Ascending)),
        }
        let mut chain = chain.into_iter();
        self.sort = chain.next();
        self.secondary_sorts = chain.collect();
    }
}
