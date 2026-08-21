import { test, expect } from "@playwright/test";

export const USER1_USERNAME = "prime1";
export const USER1_EMAIL = "prime1@heyadora.com";
export const PASSWORD = "A5%prime1@heyadora.com";

export const USER99_USERNAME = "prime99";
export const USER99_EMAIL = "prime99@heyadora.com";
// export const USER99_PASSWORD = "A6%prime1@heyadora.com";

export let login = async (page, email, password)=>{
  await page.goto("http://localhost:3000/login");

  await page.locator('[id="email"]').fill(email);
  await page.locator('[id="password"]').fill(password);
  await page.locator('[id="login_btn"]').click();

  // await page.locator('[id="gallery"] > a').first().waitFor();
  await page.locator('[id="gallery"] > a').first().waitFor();
};

export let logout = async (page)=>{
  await page.goto("http://localhost:3000/");

  // await page.locator('[id="email"]').fill(email);
  // await page.locator('[id="password"]').fill(password);
  await page.locator('[id="logout_btn"]').click();
  await page.locator('[id="login_link"]').waitFor();

  // await page.locator('[id="gallery"] > a').first().waitFor();
  // await page.locator('[id="gallery"] > a').first().waitFor();


  
};

// export let wait_for_gallery = async () => {
//     await page.locator('[id="gallery"] > a').first().waitFor();
// };

export let get_parsed_debug_state_fn = async (page) => {
  let debug = await page.evaluate(async () => wasm_bindgen.get_debug_state());
  console.log(`e2e DEBUG STATE ${JSON.stringify(debug, null, 2)}`);
  let gallery_param_limit = get_manual_data("set_gallery_param_limit", debug).map((v) => Number(v.data));

  return {
    count_interval_top: get_manual_data("gallery_interval_top_triggered", debug).length,
    count_interval_down: get_manual_data("gallery_interval_down_triggered", debug).length,
    count_mutated: get_manual_data("gallery_mutated", debug).length,
    count_init: get_manual_data("gallery_init_executed", debug).length,
    count_reset: get_manual_data("gallery_reset_executed", debug).length,
    count_anchor_selected: get_manual_data("anchor_selected", debug).length,
    count_scroll_corrected: get_manual_data("scroll_correction", debug).length,
    count_scroll_correction_reset: get_manual_data("scroll_correction_reset", debug).length,

    count_gallery_param_limit: gallery_param_limit.length,
    gallery_param_limit: gallery_param_limit,
    post_description_mutation: get_manual_data("post_description_mutation", debug),
    gallery_items: get_signal_data("gallery_api_items", debug),
    anchor_last: get_signal_data("anchor_last", debug),
    anchor_first: get_signal_data("anchor_first", debug),

  };
};

export let get_manual_data = (label, debug_state) => {
  let data = debug_state.manual_data
    .filter((v) => v.label == label)
    .map((v)=>v.data)

  console.log(
    `e2e DEBUG STATE ${label} ${JSON.stringify(data, null, 2)}`,
  );
  return data;
};

export let get_signal_data = (label, debug_state) => {
  let data = debug_state.signal_data
    .filter((v) => v.label == label)
    .map((v) => v.data.map((v)=>JSON.parse(v)) )

  console.log(
    `e2e DEBUG STATE ${label} ${JSON.stringify(data, null, 2)}`,
  );

  return data;
};

export let get_signal_data_latest = (signal_data)=>{
  let data = signal_data[signal_data.length - 1];
  data = data[data.length - 1];
  return data;
};

export let scroll_down_fn = async (
  page,
  gallery,
  offset,
  page_offset_y,
  scroll_iter_index,
) => {
  let last_item_id = await gallery.evaluate((elm) => elm.lastElementChild.id);
  let last_item_id_str = `[id="${last_item_id}"]`;
  let last_item = page.locator(last_item_id_str);

  let gallery_height = await gallery.evaluate((elm) => elm.clientHeight);
  let last_item_y = await last_item.evaluate(
    (elm) => elm.getBoundingClientRect().y,
  );

  let scroll_by = last_item_y - (page_offset_y + gallery_height + offset);

  await page.mouse.move(200, 400);

  // SCROLL 1
  await page.mouse.wheel(0, scroll_by);
  await page.locator(`[id="${last_item_id}"]`).waitFor();

  // SCROLL 2
  await page.mouse.wheel(0, offset);
  await page.locator(`[id="${last_item_id}"] + a`).waitFor();

  scroll_iter_index += 1;
};

export let get_email_change_current = async (page, email) => {
  const result = await page.evaluate(async () => {
    let result = await fetch("http://localhost:3000/api/test_backdoor_email_sent_get_all");
    return result.json();
  });
  // const response = await page.evaluate(async () => {
  //   return await fetch("http://localhost:3000/api/test_backdoor_email_sent_get_all")
  //     .then(r => r.ok ? r.json() : Promise.reject(r))
  // });
  // let result = JSON.parse(response);
  // console.log(`look at me ${result}`);
  console.log(`look at me ${JSON.stringify(result, null, 2)}`);

  let link = result["Ok"].find((v)=>(v["to_email"] == email && v["reason"] == "user_email_change_add_current")).body;
  // let link = result["Ok"][0]["body"];
  // console.log(`look at a ${JSON.stringify(a, null, 2)}`);

  return link;
};

export let get_email_change_new = async (page, email) => {
  const result = await page.evaluate(async () => {
    let result = await fetch("http://localhost:3000/api/test_backdoor_email_sent_get_all");
    return result.json();
  });
  let link = result["Ok"].find((v)=>(v["to_email"] == email && v["reason"] == "user_email_change_add_new")).body;
  return link;
};

export let get_password_change_add = async (page, email) => {
  const result = await page.evaluate(async () => {
    let result = await fetch("http://localhost:3000/api/test_backdoor_email_sent_get_all");
    return result.json();
  });
  let emails = result["Ok"];
  console.log(`searching ${emails.length} emails by to_email=${email} && reason=user_password_change_add`);
  let link = emails.find((v)=>(v["to_email"] == email && v["reason"] == "user_password_change_add")).body;
  console.log(`first result ${emails[0]}, result ${link}`);
  return link;
};

export let get_password_reset_add = async (page, email) => {
  const result = await page.evaluate(async () => {
    let result = await fetch("http://localhost:3000/api/test_backdoor_email_sent_get_all");
    return result.json();
  });
  let emails = result["Ok"];
  console.log(`searching ${emails.length} emails by to_email=${email} && reason=user_password_reset_add`);
  let link = emails.find((v)=>(v["to_email"] == email && v["reason"] == "user_password_reset_add")).body;
  console.log(`first result ${emails[0]}, result ${link}`);
  return link;
};
