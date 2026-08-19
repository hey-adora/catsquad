import { test, expect } from "@playwright/test";
import { 
    gallery_search,
    get_parsed_debug_state_fn,
    get_manual_data,
    get_signal_data,
    get_signal_data_latest,
    scroll_down_fn,
    login,
    USER1_EMAIL,
    USER1_PASSWORD,
} from "./utils";

const MAX_DESCRIPTION_LENGTH = 2000;

test("post_edit_description", async ({ page }) => {
  await login(page, USER1_EMAIL, USER1_PASSWORD);
  await page.locator('[id="gallery"] > a').first().click();

  let word = "hello";
  let iter_index = 0;

  let edit_description_fn = async (word_count) => {
      let text = Array.from(Array(word_count)).map(()=>word).reduce((a, b)=> `${a} ${b}`);
      await page.locator('[id="btn_edit_description"]').click();
      await page.locator('[id="post_description_editable"]').fill(text);

      await page.waitForTimeout(1000);

      let description_length = await page.locator('[id="description_length"]').evaluate((elm) => elm.textContent);
      expect(description_length).toBe(`${text.length}`);

      await page.locator('[id="btn_save_description"]').click();
      await page.locator('[id="post_description"]').waitFor();

      await page.waitForTimeout(1000);

      let parsed_debug2 = await get_parsed_debug_state_fn(page);
      expect(parsed_debug2.post_description_mutation.length).toBe(1 + iter_index);
      expect(parsed_debug2.post_description_mutation[parsed_debug2.post_description_mutation.length - 1])
      .toBe(text);
      
      iter_index += 1;
  };

  let edit_description_err_fn = async (word_count) => {
      let text = Array.from(Array(word_count)).map(()=>word).reduce((a, b)=> `${a} ${b}`);

      await page.locator('[id="btn_edit_description"]').click();
      await page.locator('[id="post_description_editable"]').waitFor();

      let description_text_length = await page.locator('[id="post_description_editable"]').evaluate((elm) => elm.value.length);
      let description_counter_length = await page.locator('[id="description_length"]').evaluate((elm) => elm.textContent);
      expect(description_counter_length).toBe(`${description_text_length}`);

      await page.locator('[id="post_description_editable"]').fill(text);
      await page.locator('[id="btn_save_description"]').click();
      await page.locator('[id="description_errors"]').waitFor();
      
      description_counter_length = await page.locator('[id="description_length"]').evaluate((elm) => elm.textContent);
      expect(description_counter_length).toBe(`${text.length}`);

      let parsed_debug2 = await get_parsed_debug_state_fn(page);
      expect(parsed_debug2.post_description_mutation.length).toBe(1 + iter_index);
      expect(parsed_debug2.post_description_mutation[parsed_debug2.post_description_mutation.length - 1])
      .toBe(text);

      await page.locator('[id="btn_cancel_description"]').click();
      await page.waitForTimeout(1000);

      iter_index += 1;
  };
  await edit_description_fn(2);
  await edit_description_fn(10);
  await edit_description_err_fn(MAX_DESCRIPTION_LENGTH / word.length);
});
