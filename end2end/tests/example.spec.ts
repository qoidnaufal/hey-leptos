import { test, expect } from "@playwright/test";

test("homepage has title and links to intro page", async ({ page }) => {
  await page.goto("http://localhost:4321/");

  await expect(page).toHaveTitle("HEY!");

  await expect(page.locator("p")).toHaveText("Welcome to HEY!");
});
