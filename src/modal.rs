use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ModalProps {
  #[prop_or_default]
  pub title: String,
  pub is_visible: bool,
  pub onclose: Callback<()>,
  #[prop_or_default]
  pub children: Children,
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
          "z-[100] fixed top-0 left-0 right-0 w-full overflow-x-hidden overflow-y-auto h-full max-h-full",
          "flex flex-col items-center justify-center backdrop-blur-sm"
        ])}
      >
        <div class="flex flex-col w-[32rem] p-4 bg-violet-900 rounded-md">
          <div class="flex justify-between pb-2">
            <h1 class="italic text-neutral-200">{ props.title.clone() }</h1>
            <button onclick={onclose} class="hover:text-red-400 transition duration-400 ease-in-out">
              { "⨉" }
            </button>
          </div>

          <div class="grow py-2">
            { for props.children.iter() }
          </div>
        </div>
      </div>
    }
  } else {
    html! {}
  }
}
