use leptos::{component, create_signal, event_target_value, view, IntoView, SignalGet};

#[component]
pub fn PickRoom() -> impl IntoView {
    let (room_id, set_room_id) = create_signal(String::new());
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        use leptos::window;

        ev.prevent_default();
        window()
            .location()
            .set_href(&format!("/rooms/{}", room_id.get()))
            .unwrap()
    };
    let on_input = move |ev| {
        set_room_id(event_target_value(&ev));
    };
    view! {
        <div class="max-w-4xl mx-auto px-8 sm:px-4 lg:px-6 pt-6">
            <form on:submit=on_submit>
                <input
                    type="text"
                    placeholder="Room id"
                    class="input input-bordered w-full max-w-xs"
                    prop:value=room_id
                    on:input=on_input
                />
                { move || {
                    match room_id().parse::<u64>() {
                        Ok(v) => {
                            view! {
                                <input type="submit" class="btn" value="Go!" />
                            }.into_view()
                        }
                        Err(e) => {
                            view! {
                                <input type="submit" class="btn btn-disabled" value="Nope" />
                                <span> { format!("{e:?}") } </span>
                            }.into_view()
                        }
                    }
                }}
            </form>
        </div>
    }
}
