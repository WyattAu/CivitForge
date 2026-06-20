#![forbid(unsafe_code)]

use either_of::Either;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum InputType {
    #[default]
    Text,
    Email,
    Password,
    Textarea,
    Select,
}

#[component]
pub fn Input(
    #[prop(optional)] label: &'static str,
    #[prop(optional)] input_type: InputType,
    #[prop(optional)] name: &'static str,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] value: String,
    #[prop(optional)] required: bool,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] id: &'static str,
    #[prop(optional)] options: Vec<(&'static str, &'static str)>,
) -> impl IntoView {
    let input_id = if id.is_empty() { name } else { id };
    let aria_label = if label.is_empty() { name } else { "" };
    let input_classes = "w-full px-3 py-2 border-2 border-gray-300 rounded-none \
                        dark:border-gray-600 dark:bg-gray-700 \
                        dark:text-gray-100 text-sm font-mono \
                        focus:outline-none focus:ring-2 focus:ring-blue-500 \
                        focus:border-transparent placeholder-gray-400 \
                        dark:placeholder-gray-500";
    let label_classes = "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1";

    if input_type == InputType::Textarea {
        Either::Left(view! {
            <div>
                {(!label.is_empty()).then(|| view! {
                    <label class=label_classes for=input_id>{label}</label>
                })}
                <textarea
                    id=input_id
                    name=name
                    class=input_classes
                    placeholder=placeholder
                    required=required
                    disabled=disabled
                    aria-label=aria_label
                >
                    {value}
                </textarea>
            </div>
        })
    } else if input_type == InputType::Select {
        Either::Right(Either::Left(view! {
            <div>
                {(!label.is_empty()).then(|| view! {
                    <label class=label_classes for=input_id>{label}</label>
                })}
                <select id=input_id name=name class=input_classes required=required disabled=disabled aria-label=aria_label>
                    <For each=move || options.clone() key=|o| o.0 let:opt>
                        <option value=opt.0>{opt.1}</option>
                    </For>
                </select>
            </div>
        }))
    } else {
        let tpe = match input_type {
            InputType::Email => "email",
            InputType::Password => "password",
            _ => "text",
        };
        Either::Right(Either::Right(view! {
            <div>
                {(!label.is_empty()).then(|| view! {
                    <label class=label_classes for=input_id>{label}</label>
                })}
                <input
                    type=tpe
                    id=input_id
                    name=name
                    class=input_classes
                    placeholder=placeholder
                    value=value
                    required=required
                    disabled=disabled
                    aria-label=aria_label
                />
            </div>
        }))
    }
}
