use crate::{BtnDelete, ModalConfirm, Nav, page::create_client};
use leptos::prelude::*;

mod users_state;

use users_state::UsersState;

#[component]
pub fn Admin() -> impl IntoView {
    let user_state = UsersState::new(create_client());

    view! {
        <main class="">
            <Nav/>
            <ModalConfirm/>
            <table class="border-collapse border-spacing-[10rem] border">
              <thead>
                  <tr>
                    <th class="border">username</th>
                    <th class="border">controls</th>
                  </tr>
              </thead>
              <tbody>
                  <tr class="bg-base01">
                    <td class="border">prime1</td>
                    <td class="border">
                        <BtnDelete>"delete"</BtnDelete>
                    </td>
                  </tr>
                  <tr class="bg-base02">
                    <td class="border">prime2</td>
                    <td class="border">
                        <BtnDelete>"delete"</BtnDelete>
                    </td>
                  </tr>
              </tbody>
            </table>
            <table class="border-collapse border border-gray-400 ">
              <thead>
                <tr>
                  <th class="border border-gray-300 ">State</th>
                  <th class="border border-gray-300 ">City</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td class="border border-gray-300 ">Indiana</td>
                  <td class="border border-gray-300 ">Indianapolis</td>
                </tr>
                <tr>
                  <td class="border border-gray-300 ">Ohio</td>
                  <td class="border border-gray-300 ">Columbus</td>
                </tr>
                <tr>
                  <td class="border border-gray-300 ">Michigan</td>
                  <td class="border border-gray-300 ">Detroit</td>
                </tr>
              </tbody>
            </table>
        </main>
    }
}
