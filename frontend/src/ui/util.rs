use yew::prelude::*;

/// A reusable circular loading indicator. It should be contained within a sized div for correct rendering.
///
/// Example:
/// ```rust
/// html! {
///   <div class="list-loading">
///      <CircularLoadingIndicator/>
///   </div>
/// }
/// ```
/// ```css
/// .list-loading {
///   width: 5em;
///   height: 5em;
/// }
/// ```
pub struct CircularLoadingIndicator;

impl Component for CircularLoadingIndicator {
  type Message = ();
  type Properties = ();

  fn create(_ctx: &yew::Context<Self>) -> Self {
    Self
  }

  fn view(&self, _ctx: &yew::Context<Self>) -> yew::Html {
    html! {
        <div class="circular-loading-indicator">
          {"Loading"}
        </div>
    }
  }
}
