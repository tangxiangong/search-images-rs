use std::marker::PhantomData;

use leptos::{logging, prelude::*};

fn main() {
    mount_to_body(Counter);
}

#[component]
fn Counter() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <h1>"Counter"</h1>
        <div>
            <button
                on:click=move |_| set_count.update(|n| *n += 1)
                class:red=move || count.get() % 2 == 0
                style="position: absolute"
                style:left=move || format!("{}px", count.get() + 100)
                style:background-color=move || format!("rgb({}, {}, 100)", count.get(), 100)
                style:max-width="400px"
                style=("--columns", move || count.get().to_string())
            >
                "Click me: "
                {count}
            </button>
            <progress max="50" value=count />
            <SizeOf<String> />
            <StaticList />
        </div>
        <div>
            <DynamicList />
        </div>
    }
}

#[component]
fn SizeOf<T: Sized>(#[prop(optional)] _ty: PhantomData<T>) -> impl IntoView {
    std::mem::size_of::<T>()
}

#[component]
fn StaticList() -> impl IntoView {
    let item = (0..10).map(|i| view! { <li>{i}</li> }).collect_view();

    view! { <ul>{item}</ul> }
}

#[derive(Debug, Clone)]
struct DatabaseEntry {
    key: String,
    value: i32,
}

#[component]
fn DynamicList() -> impl IntoView {
    let (data, set_data) = signal(vec![
        DatabaseEntry {
            key: "foo".to_string(),
            value: 0,
        },
        DatabaseEntry {
            key: "bar".to_string(),
            value: 1,
        },
        DatabaseEntry {
            key: "baz".to_string(),
            value: 2,
        },
    ]);

    view! {
        <button on:click=move |_| {
            set_data
                .update(|data| {
                    for entry in data {
                        entry.value *= 2;
                    }
                });
            logging::log!("{:?}", data.get())
        }>"Update"</button>
        <For each=move || data.get() key=|state| state.key.clone() let(child)>
            <p>{child.value}</p>
        </For>
    }
}
