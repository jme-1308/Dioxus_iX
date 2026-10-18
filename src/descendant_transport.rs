//! Exact native opening-frame authorization, separate from item nearest jobs.
use super::*;

impl RuntimeHandle {
    pub(super) fn query_descendant_focus(&self, query: DescendantQuery, event: &FormEvent) {
        let sequence = query.sequence;
        let root = query.root.as_str();
        let generation = query.generation;
        let reference = query.reference.as_str();
        if query.phase == "discard" {
            self.0
                .borrow_mut()
                .descendant_focus
                .discard(sequence, root, generation);
            return;
        }
        if query.phase == "cancel" {
            if self
                .0
                .borrow_mut()
                .descendant_focus
                .cancel(sequence, root, generation, reference)
            {
                self.0.borrow_mut().descendant_visual.remove(&sequence);
            }
            return;
        }
        let Some(live) = self
            .with_owner(root, generation, || {
                let entry = self.entry(root)?;
                let open = self.read_current(root, &entry)?;
                Some((entry, open))
            })
            .flatten()
        else {
            return;
        };
        match query.phase.as_str() {
            "arm" => {
                if self
                    .0
                    .borrow_mut()
                    .descendant_focus
                    .arm(sequence, root, generation, reference)
                {
                    event.prevent_default();
                }
            }
            "prepare" => {
                let Some(job) = self
                    .0
                    .borrow()
                    .descendant_focus
                    .pending_prepare(sequence, root, generation, reference)
                else {
                    return;
                };
                let returning = matches!(job.purpose, DescendantPurpose::Return { .. });
                if !returning && !live.1 {
                    self.0
                        .borrow_mut()
                        .descendant_focus
                        .cancel(sequence, root, generation, reference);
                    return;
                }
                let Some(facts) = query.facts.as_ref() else {
                    return;
                };
                if matches!(
                    job.purpose,
                    DescendantPurpose::Opening | DescendantPurpose::Return { .. }
                ) && facts.fallback != reference
                {
                    return;
                }
                let choice = if let DescendantPurpose::Return { capture } = &job.purpose {
                    crate::descendant_focus::project_candidate(facts, capture)
                } else {
                    let (Some(inventory), Some(handler)) =
                        (query.inventory.as_ref(), live.0.descendant_focus.as_ref())
                    else {
                        return;
                    };
                    let checked = self
                        .with_owner(root, generation, || {
                            handler(
                                inventory,
                                job.last,
                                job.allow_checked,
                                matches!(job.purpose, DescendantPurpose::Submenu),
                            )
                        })
                        .flatten();
                    let Some(checked) = checked else {
                        self.0
                            .borrow_mut()
                            .descendant_focus
                            .cancel(sequence, root, generation, reference);
                        return;
                    };
                    if self
                        .entry(root)
                        .is_none_or(|entry| entry.generation != generation)
                        || self
                            .entry(root)
                            .and_then(|entry| self.read_current(root, &entry))
                            != Some(true)
                    {
                        return;
                    }
                    crate::descendant_focus::choose(facts, job.last, checked)
                };
                let Some(choice) = choice else { return };
                let is_html = choice.is_html;
                let mut state = self.0.borrow_mut();
                if state
                    .descendant_focus
                    .prepare(sequence, root, generation, reference, choice)
                {
                    state.descendant_visual.insert(sequence, (job, is_html));
                    event.prevent_default();
                }
            }
            "focus" => {
                let Some(target) = query.target.as_deref() else {
                    return;
                };
                let returning = self
                    .0
                    .borrow()
                    .descendant_visual
                    .get(&sequence)
                    .is_some_and(|(job, _)| {
                        matches!(job.purpose, DescendantPurpose::Return { .. })
                    });
                if !returning && !live.1 {
                    self.0
                        .borrow_mut()
                        .descendant_focus
                        .cancel(sequence, root, generation, reference);
                    self.0.borrow_mut().descendant_visual.remove(&sequence);
                    return;
                }
                if self
                    .0
                    .borrow_mut()
                    .descendant_focus
                    .focus(sequence, root, generation, reference, target)
                    .is_some()
                {
                    event.prevent_default();
                }
            }
            "visual" => {
                let Some(target) = query.target.as_deref() else {
                    return;
                };
                // The source patch follows its original focus call. Current
                // show can now be false through a reentrant focusin listener;
                // owner lifetime, not a new hide/revision, bounds this receipt.
                if self
                    .0
                    .borrow_mut()
                    .descendant_focus
                    .visual(sequence, root, generation, reference, target)
                    .is_some()
                {
                    let visual = self.0.borrow_mut().descendant_visual.remove(&sequence);
                    if let Some((_, true)) = visual {
                        // The source patches every HTMLElement.focus, including
                        // a native no-op. A plain attempted target clears the
                        // previous path even when no focusin occurred. A
                        // selected SVG never calls that patch and changes none.
                        let host = query.attempt_path.iter().find_map(|capture| {
                            query
                                .hosts
                                .iter()
                                .find(|host| host.capture == *capture && host.source_focusable && (host.kind != ItemKind::Category || host.parts.iter().any(|part|part.kind == crate::dropdown_inventory::HostPartKind::CategoryParent && part.source_focusable && query.attempt_path.contains(&part.capture))))
                                .filter(|host| self.0.borrow_mut().item_cue_hosts.observe(host))
                                .cloned()
                        });
                        if let Some(host) = host {
                            self.mark_item_visual(&host);
                        } else {
                            self.clear_item_visual();
                        }
                    }
                    event.prevent_default();
                }
            }
            _ => {}
        }
    }
}
