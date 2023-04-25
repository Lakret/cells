use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ModalProps {
  pub children: Children,
  pub is_visible: bool,
  pub onclose: Callback<()>,
}

#[function_component]
pub fn Modal(props: &ModalProps) -> Html {
  let onclose = {
    let parent_onclose = props.onclose.clone();

    Callback::from(move |_ev: MouseEvent| {
      parent_onclose.emit(());
    })
  };

  if props.is_visible {
    html! {
      <div class={classes!(vec![
          "z-[100] fixed top-0 left-0 right-0 w-full p-4 overflow-x-hidden overflow-y-auto h-full max-h-full",
          "flex flex-col items-center justify-center backdrop-blur-sm"
        ])}
      >
        <div class="flex flex-col w-[32rem] py-2 px-4 bg-violet-900 rounded-md">
          <div class="flex justify-between pb-2">
            <h1 class="italic text-neutral-200">{ "Paste All Cells from JSON" }</h1>
            <button onclick={onclose}>{ "⨉" }</button>
          </div>

          { for props.children.iter() }
        </div>
      </div>
    }
  } else {
    html! {}
  }
}
