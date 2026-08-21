import { test, expect } from "@playwright/test";
import { 
    login,
    logout,
    get_password_reset_add,
    USER1_EMAIL,
    PASSWORD,
} from "./utils";
import * as path from 'path';

test("reset_password", async ({ page }) => {
  const edit_btn = page.locator('[id="password_reset_link"]');
  const confirm_btn = page.locator('[id="confirm_btn"]');
  const close_btn = page.locator('[id="close_btn"]');

  const form = page.locator('[id="password_reset_component"]');
  const form_err = page.locator('[id="passowrd_reset_general_error"]');
  const form_section_check = page.locator('[id="pss_reset_check_component"]');
  const form_section_confirm = page.locator('[id="pss_reset_confirm_component"]');
  const form_section_finished = page.locator('[id="pss_reset_finished_component"]');
  
  const input_email = page.locator('[id="user_email"]');
  const input_new_pss = page.locator('[id="new_password"]');
  const input_new_pss_confirm = page.locator('[id="new_password_confirm"]');

  let change_err_email = async (page) => {
    await page.goto("http://localhost:3000/login");
    await edit_btn.click();
    await input_email.fill("invalid");
    await confirm_btn.click();    
    await expect(form_err).toBeVisible();
    await close_btn.click();
    await expect(form).toBeHidden();
  };

  let change_err_pss = async (page, user_email) => {
    await page.goto("http://localhost:3000/login");
    await edit_btn.click();
    await input_email.fill(user_email);

    let link = await get_password_reset_add(page, user_email);
    await page.goto(link);

    await input_new_pss.fill("invalid");

    await confirm_btn.click();    

    await expect(form_err).toBeVisible();
    
    await close_btn.click();
    await expect(form).toBeHidden();
  };

  let reset_passowrd = async (user_email, new_password) => {
    await page.goto("http://localhost:3000/login");

    await edit_btn.click();

    await form.waitFor();
    await expect(close_btn).toBeVisible();
    await expect(confirm_btn).toBeVisible();
    await input_email.fill(user_email);

    await confirm_btn.click();
    await form_section_check.waitFor();
    await expect(close_btn).toBeVisible();
    await expect(confirm_btn).toBeHidden();

    let link = await get_password_reset_add(page, user_email);
    await page.goto(link);

    await form_section_confirm.waitFor();
    await input_new_pss.fill(new_password);
    await input_new_pss_confirm.fill(new_password);
    await expect(close_btn).toBeVisible();
    await confirm_btn.click();

    await form_section_finished.waitFor();
    await expect(confirm_btn).toBeHidden();
    await close_btn.click();

    await login(page, user_email, new_password);
    await logout(page); 
  };

  await change_err_email(page);
  await change_err_pss(page, USER1_EMAIL);
  await reset_passowrd(USER1_EMAIL, "PASSWORD_RESET_A6%prime1@heyadora.com");
  await reset_passowrd(USER1_EMAIL, PASSWORD);

});
