use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct BtnProps {
  pub title: String,
  pub color: BtnColors,
  pub onclick: Callback<MouseEvent>,
}

#[derive(PartialEq)]
pub enum BtnColors {
  Purple,
  Green,
  Violet,
}

impl BtnColors {
  pub fn to_classes(&self) -> &'static str {
    match self {
      BtnColors::Purple => "bg-purple-800 hover:bg-purple-700",
      BtnColors::Green => "bg-emerald-800 hover:bg-emerald-700",
      BtnColors::Violet => "bg-violet-800 hover:bg-violet-700",
    }
  }
}

#[function_component]
pub fn Btn(props: &BtnProps) -> Html {
  html! {
    <button
      onclick={props.onclick.clone()}
      class={classes!(vec![
        "flex items-center justify-center leading-none px-4 py-2 cursor-pointer rounded-md text-base",
        "transition-colors duration-400 ease-in-out",
        props.color.to_classes()
      ])}
    >
      { props.title.clone() }
    </button>
  }
}
