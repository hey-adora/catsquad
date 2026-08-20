import { test, expect } from "@playwright/test";
import { 
    get_parsed_debug_state_fn,
    get_manual_data,
    get_signal_data,
    get_signal_data_latest,
    scroll_down_fn,
    login,
    get_email_change_current,
    get_email_change_new,
    get_password_change_add,
    USER1_USERNAME,
    USER1_EMAIL,
    PASSWORD,
    USER99_USERNAME,
    USER99_EMAIL,
} from "./utils";
import * as path from 'path';

test("change_email", async ({ page }) => {
  await login(page, USER1_EMAIL, PASSWORD);

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
  await login(page, USER1_EMAIL, PASSWORD);

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
  await change_username(PASSWORD, USER99_USERNAME);
  await change_username(PASSWORD, USER1_USERNAME);

});

test("change_password", async ({ page }) => {
  await login(page, USER1_EMAIL, PASSWORD);

  const edit_btn = page.locator('[id="password_change_btn"]');
  const confirm_btn = page.locator('[id="confirm_btn"]');
  const close_btn = page.locator('[id="close_btn"]');
  const form = page.locator('[id="password_change_component"]');
  const form_err = page.locator('[id="passowrd_change_general_error"]');
  const form_section_add = page.locator('[id="pss_add_component"]');
  const form_section_check = page.locator('[id="pss_check_component"]');
  const form_section_confirm = page.locator('[id="pss_confirm_component"]');
  const input_new_pss = page.locator('[id="new_password"]');
  const input_new_pss_confirm = page.locator('[id="new_password_confirm"]');
  const login_page = page.locator('[id="login_page"]');


  let change_cancel = async () => {
    await page.goto("http://localhost:3000/settings");

    await edit_btn.click();

    await expect(form).toBeVisible();
    await close_btn.click();
    await expect(form).toBeHidden();
  };

  let change_err = async () => {
    await page.goto("http://localhost:3000/settings");

    await edit_btn.click();

    await form.waitFor();

    await form_section_add.waitFor();
    await confirm_btn.click(); // send password change

    await form_section_check.waitFor(); // check email form

    let link = await get_password_change_add(page, USER1_EMAIL); 

    await page.goto(link);

    await form_section_confirm.waitFor();

    await input_new_pss.fill("invalid");

    await expect(form_err).toBeHidden();

    await confirm_btn.click(); // confirm new password

    await expect(form_err).toBeVisible();
  };

  let change_passowrd = async (user_email, new_password) => {
    await page.goto("http://localhost:3000/settings");

    await edit_btn.click();

    await form.waitFor();

    await form_section_add.waitFor();
    await confirm_btn.click(); // send password change

    await expect(form_section_check).toBeVisible(); // check email form
    await expect(confirm_btn).toBeHidden();
    await expect(close_btn).toBeVisible();

    let link = await get_password_change_add(page, USER1_EMAIL); 

    await page.goto(link);

    await expect(form_section_confirm).toBeVisible();
    await input_new_pss.fill(new_password);
    await input_new_pss_confirm.fill(new_password);
    await confirm_btn.click(); // confirm new password
    
    await expect(login_page).toBeVisible();
    
    await login(page, user_email, new_password);
  };

  await change_cancel();
  await change_err();
  await change_passowrd(USER1_EMAIL, "PASSWORD_CHANGE_A6%prime1@heyadora.com");
  await change_passowrd(USER1_EMAIL, PASSWORD);

});
