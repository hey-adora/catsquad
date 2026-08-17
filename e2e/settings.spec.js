import { test, expect } from "@playwright/test";
import { 
    gallery_search,
    get_parsed_debug_state_fn,
    get_manual_data,
    get_signal_data,
    get_signal_data_latest,
    scroll_down_fn,
    login,
    get_email_change_current,
    get_email_change_new,
    USER1_USERNAME,
    USER1_EMAIL,
    USER1_PASSWORD,
    USER99_USERNAME,
    USER99_EMAIL,
    USER99_PASSWORD,
} from "./utils";
import * as path from 'path';

test("change_email", async ({ page }) => {
  await login(page, USER1_EMAIL, USER1_PASSWORD);

  let change_email = async (current_email, new_email) => {
    await page.goto("http://localhost:3000/settings");

    await page.locator('[id="email_change_btn"]').click();

    await page.locator('[id="current_add_component"]').waitFor(); // step 0

    await page.locator('[id="confirm_btn"]').click();

    await page.locator('[id="current_check_component"]').waitFor(); // step 1

    let link = await get_email_change_current(page, current_email);
    console.log(`look at link ${link}`);
    await page.goto(link);

    await page.locator('[id="current_confirm_component"]').waitFor(); // step 2

    await page.locator('[id="confirm_btn"]').click();

    await page.locator('[id="new_add_component"]').waitFor(); // step 3

    await page.locator('[id="new_email"]').fill(new_email);

    await page.locator('[id="confirm_btn"]').click();

    await page.locator('[id="new_check_component"]').waitFor(); // step 3
  
    let link2 = await get_email_change_new(page, current_email);
    console.log(`look at link ${link2}`);
    await page.goto(link2);

    await page.locator('[id="new_confirm_component"]').waitFor(); // step 4

    await page.locator('[id="confirm_btn"]').click();

    await page.locator('[id="finish_component"]').waitFor(); // step 5

    await page.locator('[id="confirm_btn"]').click();

    await page.locator('[id="finished_component"]').waitFor(); // step 6

    await page.locator('[id="close_btn"]').click();

    let current_user_email = await page.locator('[id="current_user_email"]').evaluate((elm) => elm.value);
    expect(current_user_email).toBe(new_email);
  };

  let cancel_email = async () => {
    await page.goto("http://localhost:3000/settings");
    await page.locator('[id="email_change_btn"]').click();
    await page.locator('[id="current_add_component"]').waitFor(); // step 0

    await page.locator('[id="confirm_btn"]').click();
    await page.locator('[id="current_check_component"]').waitFor(); // step 1

    await page.locator('[id="cancel_btn"]').click();
    await page.locator('[id="canceled_component"]').waitFor(); // step 1
    await page.locator('[id="close_btn"]').click();
  };

  await change_email(USER1_EMAIL, USER99_EMAIL);
  await change_email(USER99_EMAIL, USER1_EMAIL);
  await cancel_email();
});

test("change_username", async ({ page }) => {
  await login(page, USER1_EMAIL, USER1_PASSWORD);

  let change_cancel = async () => {
    await page.goto("http://localhost:3000/settings");

    await page.locator('[id="username_change_btn"]').click();

    const component = page.locator('[id="username_change_component"]');

    await component.waitFor();

    await page.locator('[id="cancel_btn"]').click();

    await expect(component).toBeHidden();
  };

  let change_err = async () => {
    await page.goto("http://localhost:3000/settings");

    await page.locator('[id="username_change_btn"]').click();

    const component = page.locator('[id="username_change_component"]');

    await component.waitFor(); 

    await page.locator('[id="confirm_btn"]').click();

    await page.locator('[id="username_change_general_error"]').waitFor(); 

    await page.locator('[id="cancel_btn"]').click();
    await expect(component).toBeHidden();
  };

  let change_username = async (current_password, new_username) => {
    await page.goto("http://localhost:3000/settings");

    await page.locator('[id="username_change_btn"]').click();

    const component = page.locator('[id="username_change_component"]');

    await component.waitFor();

    await page.locator('[id="new_username"]').fill(new_username);
    await page.locator('[id="current_password"]').fill(current_password);

    await page.locator('[id="confirm_btn"]').click();

    await expect(component).toBeHidden();

    let current_user_username = await page.locator('[id="current_user_username"]').evaluate((elm) => elm.value);
    expect(current_user_username).toBe(new_username);

  };

  await change_cancel();
  await change_err();
  await change_username(USER1_PASSWORD, USER99_USERNAME);
  await change_username(USER1_PASSWORD, USER1_USERNAME);

});
