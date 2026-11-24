use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use mesh_core::{AccountId, ReserveConfig};

use super::form::Fields;
use crate::state::{Action, use_store};

/// An inline form for a subscription's reserve, in percent. Each field is a
/// slider beside a number box, and both edit the same value. Empty 5-hour
/// or weekly fields fall back to the default, which they show greyed out; "Reset to default" clears the
/// subscription's own reserve so the daemon default applies again. With
/// `five_hour` false the provider has no 5-hour limit: the field is hidden
/// and the stored 5-hour value is kept as it is.
#[component]
pub fn ReserveEditor(
    id: AccountId,
    initial: ReserveConfig,
    five_hour: bool,
    on_close: Callback<()>,
) -> impl IntoView {
    let store = use_store();
    let fields = Fields::from_reserve(&initial);
    let default = RwSignal::new(fields.default);
    let five_hour_text = RwSignal::new(fields.five_hour);
    let weekly = RwSignal::new(fields.weekly);
    let error = RwSignal::new(None::<String>);
    let base = StoredValue::new(initial);
    let id = StoredValue::new(id);
    let busy = move || id.with_value(|id| store.is_applying(id));

    let submit = move |reserve: Option<ReserveConfig>| {
        if untrack(busy) {
            return;
        }
        error.set(None);
        store.apply(id.get_value(), Action::SetReserve(reserve), move |ok| {
            if ok {
                let _ = on_close.try_run(());
            }
        });
    };
    let save = move |ev: SubmitEvent| {
        ev.prevent_default();
        let fields = Fields {
            default: default.get_untracked(),
            five_hour: five_hour_text.get_untracked(),
            weekly: weekly.get_untracked(),
        };
        match base.with_value(|base| fields.to_reserve(base, five_hour)) {
            Ok(reserve) => submit(Some(reserve)),
            Err(message) => error.set(Some(message)),
        }
    };

    let field = move |label: &'static str, signal: RwSignal<String>, required: bool| {
        view! {
            <label class="field">
                <span>{label}</span>
                <span class="field-input">
                    <input
                        type="range"
                        class="slider"
                        class:inherited=move || signal.get().trim().is_empty()
                        min="0"
                        max="100"
                        step="1"
                        aria-label=label
                        prop:disabled=busy
                        prop:value=move || {
                            let own = signal.get();
                            if own.trim().is_empty() { default.get() } else { own }
                        }
                        on:input=move |ev| {
                            signal.set(event_target_value(&ev));
                            error.set(None);
                        }
                    />
                    <input
                        type="number"
                        min="0"
                        max="100"
                        step="any"
                        required=required
                        autofocus=required
                        prop:disabled=busy
                        placeholder=move || (!required).then(|| default.get().trim().to_owned())
                        prop:value=move || signal.get()
                        on:input=move |ev| {
                            signal.set(event_target_value(&ev));
                            error.set(None);
                        }
                    />
                    <span class="unit">"%"</span>
                </span>
            </label>
        }
    };

    view! {
        <form class="editor" on:submit=save>
            <div class="editor-fields">
                {field("Default", default, true)}
                {five_hour.then(|| field("5-hour", five_hour_text, false))}
                {field("Weekly", weekly, false)}
            </div>
            {move || error.get().map(|message| view! { <p class="editor-error" role="alert">{message}</p> })}
            <div class="editor-buttons">
                <button class="btn btn-primary" type="submit" prop:disabled=busy>
                    "Save"
                </button>
                <button
                    class="btn"
                    type="button"
                    prop:disabled=busy
                    on:click=move |_| submit(None)
                >
                    "Reset to default"
                </button>
                <button class="btn btn-ghost" type="button" prop:disabled=busy on:click=move |_| on_close.run(())>
                    "Cancel"
                </button>
            </div>
        </form>
    }
}
