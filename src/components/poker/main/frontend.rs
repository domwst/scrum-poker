use leptos::either::Either;
use leptos::prelude::*;

#[component]
pub fn PickRoom() -> impl IntoView {
    let (room_id, set_room_id) = signal(String::new());
    let on_input = move |ev| {
        set_room_id(event_target_value(&ev));
    };
    let parse_error = Memo::new(move |_| room_id.get().parse::<u64>().err());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        use leptos::prelude::window;

        ev.prevent_default();
        if parse_error.get().is_none() {
            window()
                .location()
                .set_href(&format!("/rooms/{}", room_id.get()))
                .unwrap()
        }
    };

    view! {
        <div class="max-w-4xl mx-auto px-8 sm:px-4 lg:px-6 pt-6">
            <h1 class="text-base md:text-xl lg:text-3xl font-bold my-2 text-center">"Enter room id"</h1>
            <form class="flex justify-center my-3" name="room_id" on:submit=on_submit>
                <div class="flex mx-auto">
                    <input
                        type="text"
                        placeholder="Room id"
                        class="input input-bordered w-full max-w-xs"
                        prop:value=room_id
                        on:input=on_input
                    />
                    <div class="w-2 h-auto"></div>
                    { move || {
                        match parse_error.get() {
                            None => Either::Left(view! {
                                <input type="submit" class="btn" value="Go!" />
                            }),
                            Some(e) => Either::Right(view! {
                                <div class="tooltip tooltip-right before:whitespace-pre before:content-[attr(data-tip)]" data-tip=format!("Wrong room number: {:?}\n(u64 expected)", e.kind())>
                                    <input type="submit" class="btn btn-disabled" value="Nope" />
                                </div>
                            }),
                        }
                    }}
                </div>
            </form>
        </div>
    }
}
